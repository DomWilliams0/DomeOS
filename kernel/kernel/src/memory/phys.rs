use crate::memory::phys::dumb::DumbFrameAllocator;
use crate::memory::phys::physical_size::kernel_size;
use crate::memory::KERNEL_IDENTITY_MAPPING;
use crate::multiboot::{multiboot_memory_map_t, MultibootMemoryMap};
use common::InitializedGlobal;
use common::*;
use memory::{MemoryError, PhysicalFrame, VIRT_KERNEL_SIZE};

/// Allocates physical pages
pub trait FrameAllocator {
    fn allocate_any(&mut self) -> Result<PhysicalFrame, MemoryError>;

    /// Must already be writeable upon allocation, i.e. from the kernel identity map
    fn allocate_premapped(&mut self) -> Result<PhysicalFrame, MemoryError>;

    /// Must come from below the first 4GB
    fn allocate_low(&mut self) -> Result<PhysicalFrame, MemoryError>;

    fn free(&mut self, frame: PhysicalFrame) -> Result<(), MemoryError>;

    fn relocate_multiboot(&mut self, mbi: &'static multiboot_memory_map_t);
}

static mut FRAME_ALLOCATOR: InitializedGlobal<DumbFrameAllocator> = InitializedGlobal::uninit();

pub fn init_frame_allocator(mmap: MultibootMemoryMap) {
    let size = kernel_size();
    debug!("kernel is {} ({:#x}) bytes", size, size);
    assert!(
        size < KERNEL_IDENTITY_MAPPING,
        "kernel is bigger than its identity mapping! bump KERNEL_IDENTITY_MAPPING"
    );

    #[allow(clippy::assertions_on_constants)]
    {
        assert!(
            KERNEL_IDENTITY_MAPPING < VIRT_KERNEL_SIZE,
            "kernel identity map is bigger than upper limit!"
        );
    }

    let allocator = DumbFrameAllocator::new(mmap);
    unsafe {
        FRAME_ALLOCATOR.init(allocator);
    }
}

pub fn frame_allocator() -> &'static mut impl FrameAllocator {
    unsafe { FRAME_ALLOCATOR.get() }
}

mod dumb {
    use crate::memory::phys::physical_size::kernel_end;
    use crate::memory::phys::FrameAllocator;
    use crate::memory::KERNEL_IDENTITY_MAPPING;
    use crate::multiboot::{multiboot_memory_map_t, MemoryRegionType, MultibootMemoryMap};
    use common::*;
    use memory::{MemoryError, PhysicalAddress, PhysicalFrame, FRAME_SIZE};

    pub struct DumbFrameAllocator {
        frames: Frames,
    }

    #[derive(Clone)]
    struct Frames {
        region_idx: usize,
        frame_idx: u64,

        multiboot_mmap: MultibootMemoryMap,

        /// First frame to dish out after the kernel
        start: u64,
    }

    impl Iterator for Frames {
        type Item = PhysicalFrame;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let region = self
                    .multiboot_mmap
                    .iter_regions()
                    .filter(|r| matches!(r.region_type, MemoryRegionType::Available))
                    .nth(self.region_idx)?;

                let frame = region.base_addr.address() + (self.frame_idx * FRAME_SIZE);
                if frame >= (region.base_addr.address() + region.length) {
                    // next region
                    self.region_idx += 1;
                    self.frame_idx = 0;
                    continue;
                }

                self.frame_idx += 1;

                if frame <= self.start {
                    // overlaps with kernel
                    continue;
                }

                // safety: physical addr calculated from multiboot
                return Some(unsafe { PhysicalFrame::new(PhysicalAddress(frame)) });
            }
        }
    }

    impl DumbFrameAllocator {
        pub fn new(mmap: MultibootMemoryMap) -> Self {
            let kernel_end = kernel_end();

            trace!("kernel ends at {:#x}", kernel_end);
            DumbFrameAllocator {
                frames: Frames {
                    multiboot_mmap: mmap,
                    region_idx: 0,
                    frame_idx: 0,
                    start: kernel_end,
                },
            }
        }
    }

    impl FrameAllocator for DumbFrameAllocator {
        fn allocate_any(&mut self) -> Result<PhysicalFrame, MemoryError> {
            let frame = self.frames.next();
            frame.ok_or(MemoryError::NoFrame)
        }

        fn allocate_premapped(&mut self) -> Result<PhysicalFrame, MemoryError> {
            // need to peek ahead to see if next frame is premapped

            // clone iter temporarily
            let mut peeked = self.frames.clone();

            match peeked.next() {
                Some(frame) if frame.address().address() >= KERNEL_IDENTITY_MAPPING => {
                    // frame not available, don't advance allocator iterator
                    Err(MemoryError::NoPremappedFrame)
                }
                result => {
                    // some other result, advance allocator iterator
                    debug_assert!(!core::mem::needs_drop::<Frames>());
                    self.frames = peeked;
                    result.ok_or(MemoryError::NoFrame)
                }
            }
        }

        fn allocate_low(&mut self) -> Result<PhysicalFrame, MemoryError> {
            unimplemented!()
        }

        fn free(&mut self, _frame: PhysicalFrame) -> Result<(), MemoryError> {
            unimplemented!()
        }

        fn relocate_multiboot(&mut self, mmap: &'static multiboot_memory_map_t) {
            let old_ptr = self.frames.multiboot_mmap.pointer();
            let new_ptr = mmap as *const multiboot_memory_map_t;
            // panics if new ptr is not higher, which it never is
            let offset = (new_ptr as u64) - (old_ptr as u64);
            unsafe {
                self.frames.multiboot_mmap.add_pointer_offset(offset);
            }
        }
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
