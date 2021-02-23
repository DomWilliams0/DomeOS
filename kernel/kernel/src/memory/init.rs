use crate::memory::page_table::pml4;
use crate::memory::{frame_allocator, heap, phys, FrameAllocator};
use crate::multiboot::{Multiboot, MultibootMemoryMap};
use crate::vga;
use common::*;
use enumflags2::BitFlags;
use memory::{
    gigabytes, MemoryError, PageTable, PhysicalAddress, VirtualAddress, P3, VIRT_PHYSICAL_BASE,
    VIRT_PHYSICAL_SIZE,
};

pub fn init(multiboot: Multiboot) -> Result<(), MemoryError> {
    // extract mmap from multiboot
    let memory_map = multiboot.memory_map().expect("memory map unavailable");
    debug!("memory map from multiboot: ");
    for region in memory_map.iter_regions() {
        debug!("* {:?}", region);
    }

    // init physical frame allocator
    phys::init_frame_allocator(memory_map);

    // setup physical identity mapping
    init_physical_identity_mapping()?;
    post_init_physical_identity_mapping(&memory_map)?;

    // init heap
    heap::init()?;

    Ok(())
}

/// Setup huge physical identity mapping
fn init_physical_identity_mapping() -> Result<(), MemoryError> {
    let mut p4 = pml4();
    let base = VirtualAddress::new(VIRT_PHYSICAL_BASE);
    debug!("identity mapping physical memory from {:?}", base);
    let p3_count = (VIRT_PHYSICAL_SIZE / gigabytes(512)) as u16;
    let start_idx = base.pml4t_offset();

    for i in 0..p3_count {
        let p4_offset = start_idx + i;

        // allocate new frame for p3 - will be early after the kernel image and therefore identity
        // mapped and writable already
        let p3_frame = frame_allocator().allocate(BitFlags::empty())?;

        let p3_table: &mut PageTable<P3> = unsafe { p3_frame.as_mut() };

        // initialize p3 entries to each point to 1GB each
        for (p3_offset, entry) in p3_table.entries_mut().enumerate() {
            let addr = (i as u64 * gigabytes(512)) + gigabytes(p3_offset as u64);
            entry
                .replace()
                .writeable()
                .huge()
                .address(PhysicalAddress(addr))
                .present()
                .global()
                .apply();
        }

        // point p4 entry at new p3
        let p4_entry = p4.entry_physical_mut(p4_offset);

        p4_entry
            .replace()
            .writeable()
            .present()
            .global()
            .address(p3_frame.address())
            .apply();
    }

    Ok(())
}

fn post_init_physical_identity_mapping(memory_map: &MultibootMemoryMap) -> Result<(), MemoryError> {
    // ensure frame allocator uses virtual multiboot pointer now
    frame_allocator().relocate_multiboot(unsafe {
        let phys = PhysicalAddress(memory_map.pointer() as u64);
        let virt = VirtualAddress::from_physical(phys);
        &*virt.as_ptr()
    });

    // update VGA to use new offset address
    // safety: just mapped physical identity map
    unsafe {
        let new_addr = VirtualAddress::from_physical(PhysicalAddress(0xb8000));
        trace!("moving VGA buffer to {:?}", new_addr);
        vga::get().move_vga_buffer(new_addr);
        trace!("it worked!");
    }

    // now safe to remove low identity maps from early boot
    let mut p4 = pml4();
    let mut p3 = p4.entry_mut(0).traverse()?;
    p3.entry_mut(0).replace().not_present().apply();
    p4.entry_mut(0).replace().not_present().apply();

    Ok(())
}
