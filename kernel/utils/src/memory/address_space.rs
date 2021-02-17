use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::page_table::PAGE_TABLE_ENTRY_COUNT;
use crate::memory::{HasTable, PageTableHierarchy, PhysicalFrame, FRAME_SIZE, P1, P2, P3, P4};
use crate::KernelResult;
use enumflags2::BitFlags;
use log::*;

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

#[derive(Copy, Clone, Debug)]
enum IterationRange {
    /// [start..end] in single table
    SingleInclusive(u16, u16),

    /// [start..end) in single table
    SingleExclusive(u16, u16),

    /// [start..511]
    UntilEndFrom(u16),

    /// [0..end]
    FromStartUntil(u16),
}

impl IterationRange {
    fn from_range<'p, P, N>(
        start: VirtualAddress,
        end: VirtualAddress,
    ) -> impl Iterator<Item = Self>
    where
        P: PageTableHierarchy<'p, NextLevel = N> + HasTable<'p>,
        N: PageTableHierarchy<'p> + HasTable<'p>,
    {
        let this_start = P::entry_index(start);
        let this_end = P::entry_index(end);

        // last level is off by one if it's inclusive?
        let exclusive = P::NextLevel::NAME == P1::NAME;

        let next_start = P::NextLevel::entry_index(start);
        let next_end = P::NextLevel::entry_index(end);

        let mut cursor = this_start;
        core::iter::from_fn(move || {
            if cursor > this_end {
                // finished
                return None;
            }

            let ret = if cursor == this_end {
                // final iteration
                if this_start == this_end {
                    // special case, all within single table
                    if exclusive {
                        Self::SingleExclusive(next_start, next_end)
                    } else {
                        Self::SingleInclusive(next_start, next_end)
                    }
                } else {
                    // fill final table up to final index
                    Self::FromStartUntil(next_end)
                }
            } else if cursor == this_start {
                // at the start when a!=b, at least 1 page to cover
                Self::UntilEndFrom(next_start)
            } else {
                // cursor is in the middle, neither a or b, so must be a full page inbetween
                Self::UntilEndFrom(0)
            };

            if cfg!(test) {
                trace!("{}: cursor={}, range={:?}", P::NAME, cursor, ret);
            }

            cursor += 1;
            Some(ret)
        })
    }

    fn iter(self) -> impl Iterator<Item = u16> {
        match self {
            Self::SingleInclusive(a, b) => a..(b + 1),
            Self::SingleExclusive(a, b) => a..b,
            Self::UntilEndFrom(a) => a..(PAGE_TABLE_ENTRY_COUNT as u16),
            Self::FromStartUntil(b) => 0..b,
        }
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

        let mut total_count = 0_u64;
        let p4_range = start.pml4t_offset()..=limit.pml4t_offset();
        let mut p3_range = IterationRange::from_range::<P4, _>(start, limit);
        let mut p2_range = IterationRange::from_range::<P3, _>(start, limit);
        let mut p1_range = IterationRange::from_range::<P2, _>(start, limit);

        fn iter_once(
            range: &mut impl Iterator<Item = IterationRange>,
        ) -> impl Iterator<Item = u16> {
            range.next().map(|iter| iter.iter()).into_iter().flatten()
        }

        for p4_idx in p4_range {
            let (_, mut p3) = Self::get_entry(&mut self.pml4, p4_idx, flags, &mut self.memory)?;
            for p3_idx in iter_once(&mut p3_range) {
                let (_, mut p2) = Self::get_entry(&mut p3, p3_idx, flags, &mut self.memory)?;

                for p2_idx in iter_once(&mut p2_range) {
                    let (_, mut p1) = Self::get_entry(&mut p2, p2_idx, flags, &mut self.memory)?;
                    let p1_table = p1.table_mut()?;

                    for p1_idx in iter_once(&mut p1_range) {
                        let mut entry = p1_table[p1_idx].replace();

                        for flag in flags.iter() {
                            use MapFlags::*;
                            entry = match flag {
                                Writeable => entry.writeable(),
                                Executable => entry.executable(),
                                User => entry.user(),
                                Huge2M | Huge1G => todo!("huge pages ({:?})", flag),
                            }
                        }

                        #[cfg(test)]
                        trace!("mapping {}.{}.{}.{}", p4_idx, p3_idx, p2_idx, p1_idx);

                        // for qemu debugging only
                        // TODO mark as on-demand
                        entry = entry.present();

                        entry.build();

                        total_count += 1;
                    }
                }
            }
        }

        let expected_page_count = (limit.0 - start.0) / FRAME_SIZE;
        assert_eq!(expected_page_count, total_count);

        // TODO consider huge pages

        Ok(())
    }

    fn get_entry<'pt, P: PageTableHierarchy<'pt> + 'pt>(
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
            trace!(
                "allocated new {} at {:?}",
                P::NextLevel::NAME,
                frame.address()
            );

            // ensure its cleared
            frame.zero();

            // link up to entry
            entry
                .replace()
                .address(frame.address())
                .writeable()
                .present()
                .supervisor()
                .build();

            frame.address()
        };

        // get accessible virtual address
        let virt = VirtualAddress::from_physical(phys);

        // safety: type safety from page table types
        let next_table = P::NextLevel::with_table(unsafe { &mut *virt.as_ptr() })?;
        Ok((phys, next_table))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::page_table::PageTable;

    const FRAME_COUNT: usize = 512;
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
    }
}
