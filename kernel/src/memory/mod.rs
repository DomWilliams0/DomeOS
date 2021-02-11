#[deprecated]
mod free_pages;
#[deprecated]
mod mapping;

mod page_table;
mod phys;
mod virt;

pub fn init(multiboot: &'static crate::multiboot::multiboot_info) {
    use crate::multiboot::MemoryRegion;
    use log::*;

    let regions = MemoryRegion::iter_from_multiboot(multiboot);
    debug!("memory map from multiboot: ");
    for region in regions.clone() {
        debug!("* {:?}", region);
    }

    // init physical frame allocator
    phys::init_frame_allocator(multiboot);
    for i in 0..8 {
        use phys::FrameAllocator;
        let frame = phys::frame_allocator().allocate_any().unwrap();
        debug!("frame {:4}: {:?}", i, frame.address());
    }

    // init virtual memory allocator
    // r#virt::init_virtual_allocator();

    // TODO remove 0x0 identity mapping
    // TODO resize kernel mapping to fit exactly
}
