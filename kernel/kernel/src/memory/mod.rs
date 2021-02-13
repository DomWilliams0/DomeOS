use utils::memory::*;

use crate::memory::page_table::pml4;
use crate::memory::phys::frame_allocator;
use utils::memory::address::PhysicalAddress;
use utils::memory::page_table::{EntryBuilder, PageTable};

#[deprecated]
mod free_pages;

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
        debug!("identity mapping physical memory from {:?}", base);
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
                entry
                    .builder()
                    .writeable()
                    .huge()
                    .address(PhysicalAddress(addr))
                    .present()
                    .global()
                    .build();
            }

            // point p4 entry at new p3
            let p4_entry = &mut p4[p4_offset];
            p4_entry
                .builder()
                .writeable()
                .present()
                .global()
                .address(p3_frame.address())
                .build();
        }
    }

    // update VGA to use new offset address
    // safety: just mapped physical identity map
    unsafe {
        let new_addr = VirtualAddress::from_physical(PhysicalAddress(0xb8000));
        trace!("moving VGA buffer to {:?}", new_addr);
        crate::vga::move_vga_buffer(new_addr);
        trace!("it worked!");
    }

    // now safe to remove 1MB identity map
    p4[0].builder().not_present().build();

    // TODO remove the other mapping from boot too?

    // init virtual memory allocator
    // r#virt::init_virtual_allocator();

    Ok(())
}
