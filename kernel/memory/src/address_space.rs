use crate::address::{PhysicalAddress, VirtualAddress};
use crate::custom_entry::{CustomPageEntry, DemandMapping};
use crate::{
    AnyLevel, Frame, PageTableHierarchy, PhysicalFrame, FRAME_SIZE, P4, PAGE_TABLE_ENTRY_COUNT,
};
use common::*;
use enumflags2::BitFlags;

pub trait MemoryProvider {
    fn new_frame(&mut self) -> KernelResult<PhysicalFrame>;
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
    // TODO copy on write
    // TODO global
    // TODO committed
    // TODO mapped file
}

// TODO CoW variants, recursively free pages on drop if owned

enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<'p, M: MemoryProvider> RawAddressSpace<'p, M> {
    /// # Safety
    /// Table must be mapped in and writeable already
    pub unsafe fn with_existing(pml4: P4<'p>, memory: M) -> Self {
        RawAddressSpace { pml4, memory }
    }

    // TODO constructor to allocate new possibly unmapped frame for p4, then access through id map

    #[inline]
    pub fn map_range(
        &mut self,
        start: VirtualAddress,
        size: u64,
        target: MapTarget,
        flags: impl Into<BitFlags<MapFlags>>,
    ) -> KernelResult<()> {
        self.map_range_impl(start, size, target, flags.into())
    }

    /// Actual implementation with no generic params to avoid huge code duplication
    fn map_range_impl(
        &mut self,
        start: VirtualAddress,
        size: u64,
        target: MapTarget,
        flags: BitFlags<MapFlags>,
    ) -> KernelResult<()> {
        let start = {
            let aligned = start.round_up_to(FRAME_SIZE);
            trace!("aligned base {:?} to {:?}", start, aligned);
            aligned
        };

        let limit = {
            let limit = VirtualAddress::new_checked(start.0 + size);
            let aligned = limit.round_up_to(FRAME_SIZE);
            trace!("aligned limit {:?} to {:?}", limit, aligned);
            aligned
        };

        let template_entry = {
            let mut entry = CustomPageEntry::default();
            entry.set_nx(true); // default to not executable

            for flag in flags.iter() {
                use MapFlags::*;
                match flag {
                    Writeable => entry.set_writeable(true),
                    Executable => entry.set_nx(false),
                    User => entry.set_user(true),
                    Huge2M | Huge1G => todo!("huge pages ({:?})", flag),
                }
            }

            // TODO depends on flags and target
            entry.set_on_demand(DemandMapping::Anonymous);

            entry
        };

        #[cfg(test)]
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
                        let new_entry = template_entry;

                        // safety: blatting it entirely with new custom entry, and not running any
                        // drop on probably uninitialized entry by writing through a pointer
                        unsafe {
                            let entry = p1_table.entry_mut(p1_idx).as_custom_unchecked_mut()
                                as *mut CustomPageEntry;
                            entry.write(new_entry);
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
    ) -> KernelResult<(PhysicalAddress, P::NextLevel)> {
        let entry = current.table_mut()?.entry_mut(idx);
        let phys = if entry.present() {
            // already present
            trace!("already present: {:?}", entry.address());
            entry.address()
        } else {
            // need a new frame
            let frame = memory.new_frame()?;

            // trace!(
            //     "allocated new {} at {:?}",
            //     P::NextLevel::NAME,
            //     frame.address()
            // );

            // ensure its cleared
            frame.zero();

            // link up to entry
            entry
                .replace()
                .address(frame.address())
                .writeable() // TODO depends on flags
                .present()
                .supervisor() // TODO depends on flags
                .apply();

            frame.address()
        };

        // get accessible virtual address
        let virt = VirtualAddress::from_physical(phys);

        // safety: type safety from page table types
        let next_table = P::NextLevel::with_table(unsafe { &mut *virt.as_ptr() })?;
        Ok((phys, next_table))
    }

    fn get_existing_entry_mut<P: PageTableHierarchy<'p> + 'p>(
        current: &mut P,
        idx: u16,
    ) -> KernelResult<Either<P::NextLevel, *mut CustomPageEntry>> {
        let entry = match current.table_mut() {
            Ok(table) => table.entry_mut(idx),
            Err(MemoryError::NoTableAvailable(_, addr)) => {
                return Err(MemoryError::AlreadyMapped(addr).into())
            }
            Err(err) => return Err(err.into()),
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
            Err(MemoryError::NotMapped(entry as *mut _ as u64).into())
        }
    }

    pub fn get_absent_mapping(
        &mut self,
        addr: VirtualAddress,
    ) -> KernelResult<(AnyLevel, &mut CustomPageEntry)> {
        // TODO support big absent pages
        let (p4_idx, p3_idx, p2_idx, p1_idx) = (
            addr.pml4t_offset(),
            addr.pdp_offset(),
            addr.pd_offset(),
            addr.pt_offset(),
        );

        use Either::*;
        let ptr = match Self::get_existing_entry_mut(&mut self.pml4, p4_idx)? {
            Left(mut p3) => match Self::get_existing_entry_mut(&mut p3, p3_idx)? {
                Left(mut p2) => match Self::get_existing_entry_mut(&mut p2, p2_idx)? {
                    Left(mut p1) => match Self::get_existing_entry_mut(&mut p1, p1_idx)? {
                        Left(frame) => Err(MemoryError::AlreadyMapped(frame.0.address()).into()),
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
        fn new_frame(&mut self) -> KernelResult<PhysicalFrame> {
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
                VirtualAddress::new_checked(0x5000),
                0x995, // aligned up to 0x1000
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");

        // many pages across many tables
        space
            .map_range(
                VirtualAddress::new_checked(0xaa3f0000),
                0xf0000,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");

        // stupid amount of pages
        space
            .map_range(
                VirtualAddress::new_checked(0xaa3f0000),
                0x9ae20000,
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");
    }
}
