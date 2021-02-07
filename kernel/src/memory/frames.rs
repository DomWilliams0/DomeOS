use crate::multiboot::{multiboot_info, MemoryRegion, MemoryRegionType};
use kernel_utils::memory::address::PhysicalAddress;

pub struct PhysicalFrame(PhysicalAddress);

pub struct DumbFrameAllocator {
    multiboot: &'static multiboot_info,
    next: usize,
}

pub trait FrameAllocator {
    fn allocate(&mut self) -> Option<PhysicalFrame>;

    // TODO free
}

impl DumbFrameAllocator {
    pub fn new(mbi: &'static multiboot_info) -> Self {
        DumbFrameAllocator {
            multiboot: mbi,
            next: 0,
        }
    }

    fn all_frames(&self) -> impl Iterator<Item = PhysicalFrame> {
        MemoryRegion::iter_from_multiboot(self.multiboot)
            .filter(|r| matches!(r.region_type, MemoryRegionType::Available))
            .map(|r| (r.base_addr.0)..(r.base_addr.0 + r.length))
            .flat_map(|range| range.step_by(4096))
            .map(|addr| PhysicalFrame(PhysicalAddress::from_4096_aligned(addr)))
    }
}

impl FrameAllocator for DumbFrameAllocator {
    fn allocate(&mut self) -> Option<PhysicalFrame> {
        let next = self.all_frames().nth(self.next);
        self.next += 1;
        next
    }
}

impl PhysicalFrame {
    pub fn address(&self) -> PhysicalAddress {
        self.0
    }
}
