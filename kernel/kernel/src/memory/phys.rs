use crate::memory::phys::physical_size::{kernel_end, kernel_size};
use crate::multiboot::{multiboot_info, MemoryRegion, MemoryRegionType};
use core::mem::MaybeUninit;
use log::*;
use utils::memory::address::PhysicalAddress;
use utils::InitializedGlobal;

pub struct PhysicalFrame(PhysicalAddress);

pub trait FrameAllocator {
    /// Finds a free physical frame located after 1MB and the kernel
    fn allocate_any(&mut self) -> Option<PhysicalFrame>;

    /// Finds a free physical frame below 1MB
    fn allocate_low(&mut self) -> Option<PhysicalFrame>;

    // TODO free
}

static mut FRAME_ALLOCATOR: InitializedGlobal<DumbFrameAllocator> = InitializedGlobal::uninit();

struct DumbFrameAllocator {
    multiboot: &'static multiboot_info,
    next: usize,

    /// First frame to dish out after the kernel
    start: u64,
}
pub fn init_frame_allocator(mbi: &'static multiboot_info) {
    let size = kernel_size();
    debug!("kernel is {} ({:#x})bytes", size, size);
    assert!(
        size < 4 * 1024 * 1024,
        "kernel is bigger than 4MB, initial identity mapping is too small!"
    );

    let allocator = DumbFrameAllocator::new(mbi);
    unsafe {
        FRAME_ALLOCATOR.init(allocator);
    }
}

pub fn frame_allocator() -> &'static mut impl FrameAllocator {
    unsafe { FRAME_ALLOCATOR.get() }
}

impl DumbFrameAllocator {
    fn new(mbi: &'static multiboot_info) -> Self {
        let kernel_end = kernel_end();

        trace!("kernel ends at {:#x}", kernel_end);
        DumbFrameAllocator {
            multiboot: mbi,
            next: 0,
            start: kernel_end,
        }
    }

    fn all_frames(&self) -> impl Iterator<Item = PhysicalFrame> {
        let min = self.start;

        MemoryRegion::iter_from_multiboot(self.multiboot)
            .filter(|r| matches!(r.region_type, MemoryRegionType::Available))
            .map(|r| (r.base_addr.0)..(r.base_addr.0 + r.length))
            .flat_map(|range| range.step_by(4096))
            .filter_map(move |addr| {
                if addr > min {
                    Some(PhysicalFrame(PhysicalAddress(addr)))
                } else {
                    // overlaps with kernel
                    None
                }
            })
    }
}

impl FrameAllocator for DumbFrameAllocator {
    fn allocate_any(&mut self) -> Option<PhysicalFrame> {
        let next = self.all_frames().nth(self.next);
        self.next += 1;
        next
    }

    fn allocate_low(&mut self) -> Option<PhysicalFrame> {
        unimplemented!()
    }
}

impl PhysicalFrame {
    pub fn address(&self) -> PhysicalAddress {
        self.0
    }
}

mod physical_size {
    extern "C" {
        static KERNEL_END: usize;
        static KERNEL_VIRT: usize;
        static KERNEL_PHYS: usize;
    }

    /// Physical address of KERNEL_END symbol
    pub fn kernel_end() -> u64 {
        unsafe {
            let end = (&KERNEL_END) as *const _ as u64;
            let virt_offset = (&KERNEL_VIRT) as *const _ as u64;

            end - virt_offset
        }
    }

    pub fn kernel_size() -> u64 {
        let kernel_start = unsafe { (&KERNEL_PHYS) as *const _ as u64 };

        kernel_end() - kernel_start
    }
}
