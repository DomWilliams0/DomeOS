use crate::memory::constants::{VIRT_PHYSICAL_BASE, VIRT_PHYSICAL_SIZE};
use crate::memory::page_table::pml4;
use crate::memory::phys::frame_allocator;
use utils::memory::address::PhysicalAddress;
use utils::memory::page_table::{EntryBuilder, PageTable};
use utils::memory::{gigabytes, P3};

#[deprecated]
mod free_pages;

mod constants;
mod page_table;
mod phys;
mod virt;

pub fn init(multiboot: &'static crate::multiboot::multiboot_info) -> utils::KernelResult<()> {
    use crate::multiboot::MemoryRegion;
    use log::*;
    use phys::FrameAllocator;
    use utils::memory::address::VirtualAddress;

    let regions = MemoryRegion::iter_from_multiboot(multiboot);
    debug!("memory map from multiboot: ");
    for region in regions.clone() {
        debug!("* {:?}", region);
    }

    // init physical frame allocator
    phys::init_frame_allocator(multiboot);

    // update existing page tables for kernel mappings
    let mut p4 = pml4();

    {
        // huge physical identity mapping
        // TODO do this lazily?
        let base = VirtualAddress::new(VIRT_PHYSICAL_BASE);
        let p3_count = (VIRT_PHYSICAL_SIZE / gigabytes(512)) as u16;
        let start_idx = base.pml4t_offset();

        for i in 0..p3_count {
            let p4_offset = start_idx + i;

            // allocate new frame for p3 - will be early after the kernel image and there identity
            // mapped and writable already
            let p3_frame = frame_allocator().allocate_any()?;

            let p3_table: &mut PageTable<P3> = unsafe { p3_frame.as_mut() };

            // initialize p3 entries to each point to 1GB each
            for (p3_offset, entry) in p3_table.entries_mut().enumerate() {
                let addr = (i as u64 * gigabytes(512)) + gigabytes(p3_offset as u64);
                *entry = EntryBuilder::default()
                    .writeable()
                    .huge()
                    .address(PhysicalAddress(addr))
                    .present()
                    .build();
            }

            // point p4 entry at new p3
            let p4_entry = &mut p4[p4_offset];
            *p4_entry = EntryBuilder::default()
                .writeable()
                .present()
                .address(p3_frame.address())
                .build();
        }
    }

    debug!("{:#?}", p4);

    // init virtual memory allocator
    // r#virt::init_virtual_allocator();

    // TODO remove 0x0 identity mapping
    // TODO resize kernel mapping to fit exactly

    Ok(())
}
