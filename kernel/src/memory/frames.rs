use crate::multiboot::{multiboot_info, MemoryRegion, MemoryRegionType};
use core::mem::MaybeUninit;
use utils::memory::address::PhysicalAddress;
use log::*;

pub struct PhysicalFrame(PhysicalAddress);

pub trait FrameAllocator {
    fn allocate(&mut self) -> Option<PhysicalFrame>;

    // TODO free
}

static mut FRAME_ALLOCATOR: MaybeUninit<DumbFrameAllocator> = MaybeUninit::uninit();

#[cfg(debug_assertions)]
static mut FRAME_ALLOCATOR_INIT: bool = false;

struct DumbFrameAllocator {
    multiboot: &'static multiboot_info,
    next: usize,

    /// First frame to dish out after the kernel
    start: u64,
}

extern "C" {
    static KERNEL_END: usize;
    static KERNEL_VIRT: usize;
}

pub fn init_frame_allocator(mbi: &'static multiboot_info) {
    debug_assert!(unsafe { !FRAME_ALLOCATOR_INIT });

    unsafe {
        FRAME_ALLOCATOR = MaybeUninit::new(DumbFrameAllocator::new(mbi));

        #[cfg(debug_assertions)]
        {
            FRAME_ALLOCATOR_INIT = true;
        }
    }
}

pub fn frame_allocator() -> &'static mut impl FrameAllocator {
    debug_assert!(unsafe { FRAME_ALLOCATOR_INIT });

    // safety: asserted initialized
    unsafe { FRAME_ALLOCATOR.assume_init_mut() }
}

impl DumbFrameAllocator {
    fn new(mbi: &'static multiboot_info) -> Self {
        let kernel_end = unsafe {
            let end = (&KERNEL_END) as *const _ as u64;
            let virt_offset = (&KERNEL_VIRT) as *const _ as u64;

            end - virt_offset
        };

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
