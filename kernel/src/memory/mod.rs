#[deprecated]
mod free_pages;
#[deprecated]
mod mapping;

mod frames;
mod page_table;

pub fn init(multiboot: &'static crate::multiboot::multiboot_info) {
    use crate::multiboot::MemoryRegion;
    use log::*;

    let regions = MemoryRegion::iter_from_multiboot(multiboot);
    debug!("memory map from multiboot: ");
    for region in regions.clone() {
        debug!("* {:?}", region);
    }

    // init physical frame allocator
    frames::init_frame_allocator(multiboot);
    for i in 0..12 {
        use frames::FrameAllocator;
        let frame = frames::frame_allocator().allocate().unwrap();
        debug!("frame {:4}: {:?}", i, frame.address());
    }

    // init virtual memory allocator

    // resize 4MB identity mapping?
}
