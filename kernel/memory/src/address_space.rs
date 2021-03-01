use crate::address::{PhysicalAddress, VirtualAddress};
use crate::custom_entry::{CustomPageEntry, DemandMapping};
use crate::error::MemoryResult;
use crate::{
    AnyLevel, CommonEntry, EntryBuilder, Frame, HasTable, MemoryError, PageTableBits,
    PageTableHierarchy, PhysicalFrame, FRAME_SIZE, P4, PAGE_TABLE_ENTRY_COUNT,
};
use common::*;
use core::ops::Range;
use enumflags2::BitFlags;

pub trait MemoryProvider {
    fn new_frame(&mut self) -> Result<PhysicalFrame, MemoryError>;
}

pub struct RawAddressSpace<'p, M> {
    pml4: P4<'p>,
    memory: M,
}

pub enum MapTarget {
    /// Map to any physical address
    Any,

    /// Map to this physical address
    Specific(PhysicalAddress),
}

#[derive(BitFlags, Debug, Copy, Clone)]
#[repr(u16)]
pub enum MapFlags {
    Writeable = 1 << 0,
    Executable = 1 << 1,
    User = 1 << 2,

    Huge2M = 1 << 3,
    Huge1G = 1 << 4,

    StackGuard = 1 << 5,
    Commit = 1 << 6,
    // TODO copy on write
    // TODO global
    // TODO committed
    // TODO mapped file
}

// TODO CoW variants, recursively free pages on drop if owned

/// First call to iter(): [given index..512)
/// Future calls        : [0..512)
struct OneTimeEntryRange(Range<u16>);

impl From<BitFlags<MapFlags>> for PageTableBits {
    fn from(flags: BitFlags<MapFlags>) -> Self {
        let mut bits = PageTableBits::default().with_nx(true);

        for flag in flags.iter() {
            use MapFlags::*;
            match flag {
                Writeable => bits.set_writeable(true),
                Executable => bits.set_nx(false),
                User => bits.set_user(true),
                StackGuard | Commit | Huge2M | Huge1G => {}
            }
        }

        bits
    }
}

impl<'p, M: MemoryProvider> RawAddressSpace<'p, M> {
    /// # Safety
    /// Table must be mapped in and writeable already
    pub unsafe fn with_existing(pml4: P4<'p>, memory: M) -> Self {
        RawAddressSpace { pml4, memory }
    }

    // TODO constructor to allocate new possibly unmapped frame for p4, then access through id map

    pub fn map_range(
        &mut self,
        start: VirtualAddress,
        size: u64,
        target: MapTarget,
        flags: impl Into<BitFlags<MapFlags>>,
    ) -> MemoryResult<()> {
        self.map_range_impl(start, size, target, flags.into())
    }

    /// Actual implementation with no generic params to avoid huge code duplication
    #[allow(clippy::let_and_return)]
    fn map_range_impl(
        &mut self,
        start: VirtualAddress,
        size: u64,
        target: MapTarget,
        flags: BitFlags<MapFlags>,
    ) -> MemoryResult<()> {
        let start = {
            let aligned = start.round_up_to(FRAME_SIZE);
            #[cfg(feature = "log-paging")]
            trace!("aligned base {:?} to {:?}", start, aligned);
            aligned
        };

        let limit = {
            let limit = VirtualAddress::with_literal(start.0 + size);
            let aligned = limit.round_up_to(FRAME_SIZE);
            #[cfg(feature = "log-paging")]
            trace!("aligned limit {:?} to {:?}", limit, aligned);
            aligned
        };

        enum NewEntry {
            Absent(CustomPageEntry),
            Committed(PageTableBits),
        }

        let new_entry = {
            let mut bits = PageTableBits::default().with_nx(true);

            let mut commit = false;
            let mut demand = DemandMapping::Anonymous;

            for flag in flags.iter() {
                use MapFlags::*;
                match flag {
                    Writeable => bits.set_writeable(true),
                    Executable => bits.set_nx(false),
                    User => bits.set_user(true),
                    StackGuard => demand = DemandMapping::StackGuard,
                    Commit => commit = true,
                    Huge2M | Huge1G => todo!("huge pages ({:?})", flag),
                }
            }

            if commit {
                NewEntry::Committed(bits)
            } else {
                NewEntry::Absent(CustomPageEntry::from_bits(bits).with_on_demand(demand))
            }
        };

        #[cfg(feature = "log-paging")]
        trace!(
            "mapping {}.{}.{}.{} => {}.{}.{}.{}",
            start.pml4t_offset(),
            start.pdp_offset(),
            start.pd_offset(),
            start.pt_offset(),
            limit.pml4t_offset(),
            limit.pdp_offset(),
            limit.pd_offset(),
            limit.pt_offset(),
        );

        const KEEP_LOOPING: u16 = 4;

        let mut unroll = KEEP_LOOPING;

        // entry index into each table
        let mut tables = [
            start.pml4t_offset(),
            start.pdp_offset(),
            start.pd_offset(),
            start.pt_offset(),
        ];

        // TODO allocate a single CoW frame for template entry and point everything at it

        let mut total_count = 0;
        let mut things = iter_all_pages(start, limit);

        'outer: while unroll > 0 {
            let (_, mut p3) =
                Self::get_or_create_entry_mut(&mut self.pml4, tables[0], flags, &mut self.memory)?;

            unroll = KEEP_LOOPING;
            while unroll > 1 {
                let (_, mut p2) =
                    Self::get_or_create_entry_mut(&mut p3, tables[1], flags, &mut self.memory)?;

                unroll = KEEP_LOOPING;
                while unroll > 2 {
                    let (_, mut p1) =
                        Self::get_or_create_entry_mut(&mut p2, tables[2], flags, &mut self.memory)?;
                    let p1_table = p1.table_mut()?;

                    let (pages_to_do, new_tables) = match things.next() {
                        Some((n, to_unroll, new_indices)) => {
                            unroll = to_unroll;
                            (n, new_indices)
                        }
                        None => {
                            // finished
                            break 'outer;
                        }
                    };
                    #[cfg(test)]
                    trace!(
                        "mapping {} {}s from {}.{}.{}.{}",
                        pages_to_do,
                        crate::Frame::NAME, // TODO get from generic parameter instead
                        tables[0],
                        tables[1],
                        tables[2],
                        tables[3],
                    );

                    for p1_idx in tables[3]..tables[3] + pages_to_do {
                        let entry = p1_table.entry_mut(p1_idx);

                        match new_entry {
                            NewEntry::Absent(custom) => {
                                // safety: blatting it entirely with new custom entry, and not running any
                                // drop on probably uninitialized entry by writing through a pointer
                                unsafe {
                                    (entry.as_custom_unchecked_mut() as *mut CustomPageEntry)
                                        .write(custom);
                                }
                            }
                            NewEntry::Committed(bits) => {
                                // allocate new physical
                                // TODO check for previous mapping?
                                Self::create_entry(entry, bits, &mut self.memory)?;
                            }
                        }
                    }

                    total_count += pages_to_do as u64;
                    tables = new_tables;
                }
            }
        }

        // ensure we mapped the exact amount of pages
        let expected_page_count = (limit.0 - start.0) / FRAME_SIZE;
        assert_eq!(expected_page_count, total_count);

        // TODO consider huge pages

        Ok(())
    }

    /// Allocates a new physical frame if not already present
    fn get_or_create_entry_mut<'pt, P: PageTableHierarchy<'pt> + 'pt>(
        current: &mut P,
        idx: u16,
        flags: BitFlags<MapFlags>,
        memory: &mut M,
    ) -> MemoryResult<(PhysicalAddress, P::NextLevel)> {
        let entry = current.table_mut()?.entry_mut(idx);
        let phys = if entry.present() {
            // already present
            #[cfg(feature = "log-paging")]
            trace!("already present: {:?}", entry.address());
            entry.address()
        } else {
            // need a new frame
            Self::create_entry(entry, flags.into(), memory)?
        };

        // get accessible virtual address
        let virt = VirtualAddress::from_physical(phys);

        // safety: type safety from page table types
        let next_table = P::NextLevel::with_table(unsafe { &mut *virt.as_ptr() })?;
        Ok((phys, next_table))
    }

    fn create_entry<'pt, P: PageTableHierarchy<'pt> + 'pt>(
        entry: &mut CommonEntry<'pt, P>,
        bits: PageTableBits,
        memory: &mut M,
    ) -> MemoryResult<PhysicalAddress> {
        let frame = memory.new_frame()?;

        #[cfg(feature = "log-paging")]
        trace!(
            "allocated new {} at {:?}",
            P::NextLevel::NAME,
            frame.address()
        );

        // ensure its cleared
        frame.zero();

        // link up to entry
        EntryBuilder::with_entry_and_bits(entry, bits)
            .address(frame.address())
            .present()
            .apply();

        Ok(frame.address())
    }

    /// Returns mutable reference to entry but makes no changes.
    /// Errors:
    ///     * AlreadyMapped
    ///     * NotMapped
    fn get_existing_entry<P: PageTableHierarchy<'p> + 'p>(
        current: &mut P,
        idx: u16,
    ) -> MemoryResult<Either<P::NextLevel, *mut CustomPageEntry>> {
        let entry = match current.table_mut() {
            Ok(table) => table.entry_mut(idx),
            Err(MemoryError::NoTableAvailable(_, addr)) => {
                return Err(MemoryError::AlreadyMapped(addr))
            }
            Err(err) => return Err(err),
        };

        if entry.present() {
            let next_level = if entry.huge_pages() {
                P::NextLevel::with_frame(Frame(entry.address()))?
            } else {
                // get accessible virtual address for table
                let virt = VirtualAddress::from_physical(entry.address());
                P::NextLevel::with_table(unsafe { &mut *virt.as_ptr() })?
            };

            Ok(Either::Left(next_level))
        } else if let Some(custom) = entry.as_custom_mut() {
            // great, custom
            Ok(Either::Right(custom as *mut _))
        } else {
            // not mapped
            Err(MemoryError::NotMapped(entry as *mut _ as u64))
        }
    }

    pub fn get_absent_mapping(
        &mut self,
        addr: VirtualAddress,
    ) -> MemoryResult<(AnyLevel, &mut CustomPageEntry)> {
        // TODO support big absent pages
        let (p4_idx, p3_idx, p2_idx, p1_idx) = (
            addr.pml4t_offset(),
            addr.pdp_offset(),
            addr.pd_offset(),
            addr.pt_offset(),
        );

        use Either::*;
        let ptr = match Self::get_existing_entry(&mut self.pml4, p4_idx)? {
            Left(mut p3) => match Self::get_existing_entry(&mut p3, p3_idx)? {
                Left(mut p2) => match Self::get_existing_entry(&mut p2, p2_idx)? {
                    Left(mut p1) => match Self::get_existing_entry(&mut p1, p1_idx)? {
                        Left(frame) => Err(MemoryError::AlreadyMapped(frame.0.address())),
                        Right(mapping) => Ok((AnyLevel::Frame, mapping)),
                    },
                    Right(mapping) => Ok((AnyLevel::P1, mapping)),
                },
                Right(mapping) => Ok((AnyLevel::P2, mapping)),
            },
            Right(mapping) => Ok((AnyLevel::P3, mapping)),
        };

        // safety: cant return reference from "borrowed" page tables, but everything lives in
        // physical memory and so is actually present still
        ptr.map(|(level, ptr)| (level, unsafe { &mut *ptr }))
    }

    /// Unmapped page returned as `Ok(Either::Right())`
    fn get_unmapped_entry<P, N>(table: &mut P, idx: u16) -> MemoryResult<Either<N, ()>>
    where
        P: PageTableHierarchy<'p, NextLevel = N> + 'p,
        N: HasTable<'p>,
    {
        match Self::get_existing_entry(table, idx) {
            Ok(Either::Left(next)) => {
                // already mapped as a table, traverse to next level
                Ok(Either::Left(next))
            }
            Ok(Either::Right(ptr)) => {
                // already mapped but absent
                Err(MemoryError::AlreadyMapped(ptr as u64))
            }
            Err(MemoryError::NotMapped(_)) => {
                // not mapped, success
                Ok(Either::Right(()))
            }
            Err(other) => {
                // any other error
                Err(other)
            }
        }
    }

    fn iter_unmapped_foreach(
        p4: &mut P4<'p>,
        start_addr: VirtualAddress,
        mut todo: impl FnMut(AnyLevel, VirtualAddress) -> bool,
    ) {
        const ENTRY_MAX: u16 = PAGE_TABLE_ENTRY_COUNT as u16;

        let mut p3_range = OneTimeEntryRange::new_from(start_addr.pdp_offset());
        let mut p2_range = OneTimeEntryRange::new_from(start_addr.pd_offset());
        let mut p1_range = OneTimeEntryRange::new_from(start_addr.pt_offset());

        // im so sorry... maybe eventually we'll have a non-generic page table
        // representation and could iterate instead
        for p4_idx in start_addr.pml4t_offset()..ENTRY_MAX {
            match Self::get_unmapped_entry(p4, p4_idx) {
                Ok(Either::Right(_)) => {
                    if !todo(
                        AnyLevel::P4,
                        VirtualAddress::from_indices(
                            p4_idx,
                            p3_range.start(),
                            p2_range.start(),
                            p1_range.start(),
                        ),
                    ) {
                        return;
                    }
                }
                Ok(Either::Left(mut p3)) => {
                    for p3_idx in p3_range.iter() {
                        match Self::get_unmapped_entry(&mut p3, p3_idx) {
                            Ok(Either::Right(_)) => {
                                if !todo(
                                    AnyLevel::P3,
                                    VirtualAddress::from_indices(
                                        p4_idx,
                                        p3_idx,
                                        p2_range.start(),
                                        p1_range.start(),
                                    ),
                                ) {
                                    return;
                                }
                            }
                            Ok(Either::Left(mut p2)) => {
                                for p2_idx in p2_range.iter() {
                                    match Self::get_unmapped_entry(&mut p2, p2_idx) {
                                        Ok(Either::Right(_)) => {
                                            if !todo(
                                                AnyLevel::P2,
                                                VirtualAddress::from_indices(
                                                    p4_idx,
                                                    p3_idx,
                                                    p2_idx,
                                                    p1_range.start(),
                                                ),
                                            ) {
                                                return;
                                            }
                                        }
                                        Ok(Either::Left(mut p1)) => {
                                            let p1_table = match p1.table_mut() {
                                                Ok(p1) => p1,
                                                Err(_) => {
                                                    // not a table
                                                    continue;
                                                }
                                            };
                                            for p1_idx in p1_range.iter() {
                                                let entry = p1_table.entry_mut(p1_idx);
                                                if entry.present()
                                                    || entry.as_custom_mut().is_some()
                                                {
                                                    // mapped
                                                    continue;
                                                }

                                                if !todo(
                                                    AnyLevel::P1,
                                                    VirtualAddress::from_indices(
                                                        p4_idx, p3_idx, p2_idx, p1_idx,
                                                    ),
                                                ) {
                                                    return;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
        }
    }

    #[allow(clippy::let_and_return)]
    pub fn find_free_space(
        &mut self,
        start: VirtualAddress,
        n_contiguous: usize,
    ) -> MemoryResult<VirtualAddress> {
        let n_to_find = n_contiguous as u64;

        let mut last_addr = None;
        let mut contiguous_start = None;

        // for checking no mappings exist, safe because table walking functions modify nothing
        // even though they're mutable (to return a mutable reference)
        #[cfg(debug_assertions)]
        let other_self = unsafe { &mut *(self as *mut Self) };

        Self::iter_unmapped_foreach(&mut self.pml4, start, |level, addr| {
            let is_consecutive = {
                last_addr
                    .map(|last: VirtualAddress| {
                        let consecutive = are_consecutive(last, addr, level);

                        #[cfg(feature = "log-paging")]
                        trace!(
                            " considering {:?}, level={:?}, consecutive={:?} (last={:?})",
                            addr,
                            level,
                            consecutive,
                            last
                        );

                        consecutive
                    })
                    .unwrap_or(true)
            };

            last_addr = Some(addr);

            if !is_consecutive || contiguous_start.is_none() {
                // restart search from here
                #[cfg(feature = "log-paging")]
                trace!(" restarting search from {:?}", addr);
                contiguous_start = Some(addr);
            }

            #[cfg(debug_assertions)]
            if let Ok((_, entry)) = other_self.get_absent_mapping(addr) {
                panic!("\"unmapped\" page is actually mapped: {:?}", entry)
            }

            // calculate number of consecutive pages we have
            let contiguous_start = contiguous_start.unwrap(); // unconditionally initialized above
            let contiguous_count = (addr.address() - contiguous_start.address()) / FRAME_SIZE;

            // keep going only if not done yet
            contiguous_count < n_to_find
        });

        contiguous_start
            .ok_or_else(|| MemoryError::NoContiguousVirtualRegion(start.address(), n_to_find))
    }
}

fn are_consecutive(a: VirtualAddress, b: VirtualAddress, level: AnyLevel) -> bool {
    const ENTRY_COUNT: u64 = PAGE_TABLE_ENTRY_COUNT as u64;

    const DIFF_P1: u64 = FRAME_SIZE;
    const DIFF_P2: u64 = FRAME_SIZE * ENTRY_COUNT;
    const DIFF_P3: u64 = FRAME_SIZE * ENTRY_COUNT.pow(2);
    const DIFF_P4: u64 = FRAME_SIZE * ENTRY_COUNT.pow(3);

    let diff = b.address() - a.address();
    match diff {
        DIFF_P1 => true,
        DIFF_P2 if level == AnyLevel::P2 || level == AnyLevel::P3 => true,
        DIFF_P3 if level == AnyLevel::P3 || level == AnyLevel::P4 => true,
        DIFF_P4 if level == AnyLevel::P4 => true,
        _ => false,
    }
}

impl OneTimeEntryRange {
    fn new_from(start: u16) -> Self {
        Self(start..PAGE_TABLE_ENTRY_COUNT as u16)
    }

    fn start(&self) -> u16 {
        self.0.start
    }

    fn iter(&mut self) -> impl Iterator<Item = u16> {
        let new_range = 0..PAGE_TABLE_ENTRY_COUNT as u16;
        core::mem::replace(&mut self.0, new_range)
    }
}

/// (number of pt entries to do, new unwind value, new tables values)
fn iter_all_pages(
    start: VirtualAddress,
    end: VirtualAddress,
) -> impl Iterator<Item = (u16, u16, [u16; 4])> {
    const LIMIT: u16 = PAGE_TABLE_ENTRY_COUNT as u16;

    // number of pages to map
    let real_count = (end.0 - start.0) / FRAME_SIZE;

    let mut tables = [
        start.pml4t_offset(),
        start.pdp_offset(),
        start.pd_offset(),
        start.pt_offset(),
    ];

    let mut remaining = real_count;

    core::iter::from_fn(move || {
        let pages = remaining.min((LIMIT - tables[3]) as u64);

        if pages == 0 {
            // all done
            return None;
        }

        remaining -= pages;

        let mut unwind = 0;

        if remaining > 0 {
            // only try to increment indices if there's another iteration after this

            unwind = 3;

            // wrap digits around
            tables[3] = 0;
            tables[2] += 1;

            if tables[2] == LIMIT {
                unwind -= 1;

                tables[2] = 0;
                tables[1] += 1;
                if tables[1] == LIMIT {
                    unwind -= 1;

                    tables[1] = 0;
                    tables[0] += 1;
                    assert_ne!(tables[0], LIMIT); // p4 can't wrap around
                }
            }
        }

        Some((pages as u16, unwind, tables))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::{PhysicalAddress, VirtualAddress};
    use crate::{PageTable, PhysicalFrame, FRAME_SIZE, P4};

    const FRAME_COUNT: usize = 4096;
    struct Memory {
        pages: Box<[u8]>,
        next: usize,
    }

    impl Memory {
        fn new() -> Self {
            Self {
                pages: vec![0u8; FRAME_COUNT * FRAME_SIZE as usize].into_boxed_slice(),
                next: 0,
            }
        }
    }

    impl MemoryProvider for Memory {
        fn new_frame(&mut self) -> Result<PhysicalFrame, MemoryError> {
            let idx = self.next;
            assert!(idx < FRAME_COUNT, "all gone");
            self.next += 1;

            let frame = &self.pages[idx * FRAME_SIZE as usize..];
            unsafe { Ok(PhysicalFrame::new(PhysicalAddress(frame.as_ptr() as u64))) }
        }
    }

    #[test]
    fn mapping() {
        // main testing is done by the asserts in map_range e.g. exact number of pages is mapped

        env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .init();

        let mut p4 = PageTable::default();
        let memory = Memory::new();

        let mut space =
            unsafe { RawAddressSpace::with_existing(P4::with_initialized(&mut p4), memory) };

        // a single page
        space
            .map_range(
                VirtualAddress::with_literal(0x5000),
                0x995, // aligned up to 0x1000
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");

        // many pages across many tables
        space
            .map_range(
                VirtualAddress::with_literal(0xaa3f0000),
                0xf0000,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");

        // stupid amount of pages
        space
            .map_range(
                VirtualAddress::with_literal(0xaa3f0000),
                0x9ae20000,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");
    }
}
