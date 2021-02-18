use crate::address::{PhysicalAddress, VirtualAddress};
use crate::{PageTableHierarchy, PhysicalFrame, FRAME_SIZE, P4, PAGE_TABLE_ENTRY_COUNT};
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

        const KEEP_LOOPING: u16 = 4;

        let mut unroll = KEEP_LOOPING;

        // entry index into each table
        let mut tables = [
            start.pml4t_offset(),
            start.pdp_offset(),
            start.pd_offset(),
            start.pt_offset(),
        ];

        let mut total_count = 0;
        let mut things = iter_all_pages(start, limit);

        'outer: while unroll > 0 {
            let (_, mut p3) = Self::get_entry(&mut self.pml4, tables[0], flags, &mut self.memory)?;

            unroll = KEEP_LOOPING;
            while unroll > 1 {
                let (_, mut p2) = Self::get_entry(&mut p3, tables[1], flags, &mut self.memory)?;

                unroll = KEEP_LOOPING;
                while unroll > 2 {
                    let (_, mut p1) = Self::get_entry(&mut p2, tables[2], flags, &mut self.memory)?;
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
                        let mut entry = p1_table.entry_mut(p1_idx).replace();

                        // TODO calc flags once and copy each time
                        for flag in flags.iter() {
                            use MapFlags::*;
                            entry = match flag {
                                Writeable => entry.writeable(),
                                Executable => entry.executable(),
                                User => entry.user(),
                                Huge2M | Huge1G => todo!("huge pages ({:?})", flag),
                            }
                        }

                        // TODO mark as on-demand
                        // for qemu debugging only
                        // entry = entry.present();

                        entry.build();
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
