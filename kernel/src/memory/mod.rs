use crate::memory::frames::{DumbFrameAllocator, FrameAllocator};
use crate::memory::page_table::log_active_page_hierarchy;

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
    let mut alloc = DumbFrameAllocator::new(multiboot);
    for (i, frame) in core::iter::from_fn(|| alloc.allocate())
        .enumerate()
        .take(12)
    {
        debug!("frame {:4}: {:?}", i, frame.address());
    }

    // init virtual memory allocator

    // resize 4MB identity mapping?
}
