use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::page_table::PAGE_TABLE_ENTRY_COUNT;
use crate::memory::{PageTableHierarchy, PhysicalFrame, FRAME_SIZE, P2, P3, P4};
use crate::KernelResult;
use enumflags2::BitFlags;
use log::*;

pub trait MemoryProvider {
    fn new_frame(&self) -> KernelResult<PhysicalFrame>;
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
    Single(u16, u16),

    /// [start..511]
    UntilEndFrom(u16),

    /// [0..end]
    FromStartUntil(u16),
}

impl IterationRange {
    fn from_range<'p, P: PageTableHierarchy<'p>>(
        start: VirtualAddress,
        end: VirtualAddress,
    ) -> impl Iterator<Item = Self> {
        let this_start = P::entry_index(start);
        let this_end = P::entry_index(end);
        assert!(this_end >= this_start);

        let next_start = P::NextLevel::entry_index(start);
        let next_end = P::NextLevel::entry_index(end);
        assert!(next_end >= next_start);

        let mut cursor = this_start;
        core::iter::from_fn(move || {
            if cursor > this_end {
                // finished
                return None;
            }

            // cursor=0, p4
            let ret = if cursor == this_end {
                // final iteration
                if this_start == this_end {
                    // special case, all within single table
                    Self::Single(next_start, next_end)
                } else {
                    Self::FromStartUntil(next_end)
                }
            } else if cursor == this_start {
                // at the start when a!=b, at least 1 pages to cover
                Self::UntilEndFrom(next_start)
            } else {
                // cursor is in the middle, neither a or b, so must be a full page inbetween
                Self::UntilEndFrom(0)
            };

            cursor += 1;
            Some(ret)
        })
    }

    fn iter(self) -> impl Iterator<Item = u16> {
        match self {
            Self::Single(a, b) => a..(b + 1),
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
            trace!("aligned {:?} to {:?}", start, aligned);
            aligned
        };

        let limit = {
            let limit = VirtualAddress::new_checked(start.0 + size);
            let aligned = limit.round_up_to(FRAME_SIZE);
            trace!("aligned {:?} to {:?}", limit, aligned);
            limit
        };

        let mut total_count = 0_u64;
        let p4_range = start.pml4t_offset()..=limit.pml4t_offset();
        let mut p3_range = IterationRange::from_range::<P4>(start, limit);
        let mut p2_range = IterationRange::from_range::<P3>(start, limit);
        let mut p1_range = IterationRange::from_range::<P2>(start, limit);

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

    struct Memory {
        pages: [u8; 512 * FRAME_SIZE as usize],
        next: usize,
    }

    impl MemoryProvider for Memory {
        fn new_frame(&self) -> KernelResult<PhysicalFrame> {
            todo!();
            unsafe { Ok(PhysicalFrame::new(PhysicalAddress(0))) }
        }
    }

    #[test]
    fn mapping() {
        let mut p4 = PageTable::default();
        let memory: Memory = unsafe { std::mem::zeroed() };

        let mut space =
            unsafe { RawAddressSpace::with_existing(P4::with_initialized(&mut p4), memory) };

        space
            .map_range(
                VirtualAddress::new_checked(0x5000),
                0x995, // aligned up to 0x1000
                MapTarget::Any,
                MapFlags::Writeable | MapFlags::User,
            )
            .expect("mapping failed");
    }
}
