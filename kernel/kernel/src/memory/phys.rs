use crate::memory::phys::dumb::DumbFrameAllocator;
use crate::memory::phys::physical_size::kernel_size;
use crate::multiboot::{multiboot_memory_map_t, MultibootMemoryMap};
use common::{InitializedGlobal, KernelResult};
use enumflags2::BitFlags;
use log::*;
use memory::PhysicalFrame;

#[derive(BitFlags, Debug, Copy, Clone)]
#[repr(u16)]
pub enum FrameFlags {
    /// Will come from below the first 1MB
    Low = 1 << 0,
    // /// Will come from the first identity mapped 1GB after the kernel
    // PreMapped = 1 << 1,
}

/// Allocates physical pages
pub trait FrameAllocator {
    fn allocate(&mut self, flags: BitFlags<FrameFlags>) -> KernelResult<PhysicalFrame>;

    fn free(&mut self, frame: PhysicalFrame) -> KernelResult<()>;

    fn relocate_multiboot(&mut self, mbi: &'static multiboot_memory_map_t);
}

static mut FRAME_ALLOCATOR: InitializedGlobal<DumbFrameAllocator> = InitializedGlobal::uninit();

pub fn init_frame_allocator(mmap: MultibootMemoryMap) {
    let size = kernel_size();
    debug!("kernel is {} ({:#x}) bytes", size, size);
    assert!(
        size < 4 * 1024 * 1024,
        "kernel is bigger than 4MB, initial identity mapping is too small!"
    );

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
    use crate::memory::phys::{FrameAllocator, FrameFlags};
    use crate::multiboot::{multiboot_memory_map_t, MemoryRegionType, MultibootMemoryMap};
    use common::{KernelError, KernelResult, MemoryError};
    use enumflags2::BitFlags;
    use log::*;
    use memory::{PhysicalAddress, PhysicalFrame};

    pub struct DumbFrameAllocator {
        multiboot_mmap: MultibootMemoryMap,
        next: usize,

        /// First frame to dish out after the kernel
        start: u64,
    }

    impl DumbFrameAllocator {
        pub fn new(mmap: MultibootMemoryMap) -> Self {
            let kernel_end = kernel_end();

            trace!("kernel ends at {:#x}", kernel_end);
            DumbFrameAllocator {
                multiboot_mmap: mmap,
                next: 0,
                start: kernel_end,
            }
        }

        fn all_frames(&self) -> impl Iterator<Item = PhysicalFrame> + '_ {
            let min = self.start;

            self.multiboot_mmap
                .iter_regions()
                .filter(|r| matches!(r.region_type, MemoryRegionType::Available))
                .map(|r| (r.base_addr.0)..(r.base_addr.0 + r.length))
                .flat_map(|range| range.step_by(4096))
                .filter_map(move |addr| {
                    if addr > min {
                        // safety: physical addr calculated from multiboot
                        let frame = unsafe { PhysicalFrame::new(PhysicalAddress(addr)) };
                        Some(frame)
                    } else {
                        // overlaps with kernel
                        None
                    }
                })
        }
    }

    impl FrameAllocator for DumbFrameAllocator {
        fn allocate(&mut self, flags: BitFlags<FrameFlags>) -> KernelResult<PhysicalFrame> {
            // TODO separate allocator for low memory
            if flags.contains(FrameFlags::Low) {
                return Err(KernelError::NotImplemented);
            }

            let next = self.all_frames().nth(self.next);
            self.next += 1;
            Ok(next.ok_or(MemoryError::NoFrame)?)
        }

        fn free(&mut self, _frame: PhysicalFrame) -> KernelResult<()> {
            unimplemented!()
        }

        fn relocate_multiboot(&mut self, mmap: &'static multiboot_memory_map_t) {
            let old_ptr = self.multiboot_mmap.pointer();
            let new_ptr = mmap as *const multiboot_memory_map_t;
            // panics if new ptr is not higher, which it never is
            let offset = (new_ptr as u64) - (old_ptr as u64);
            unsafe {
                self.multiboot_mmap.add_pointer_offset(offset);
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
