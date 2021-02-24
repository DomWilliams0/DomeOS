use crate::memory::AddressSpace;
use common::*;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ptr::{null_mut, NonNull};
use memory::{
    gigabytes, kilobytes, round_up_to, MapFlags, MapTarget, MemoryError, VirtualAddress,
    FRAME_SIZE, VIRT_KERNEL_HEAP_BASE,
};

#[global_allocator]
static mut HEAP: Heap = Heap(RefCell::new(buddy_system_allocator::Heap::empty()));

const MIN_HEAP_ALLOC: u64 = kilobytes(512);
const MAX_HEAP_ALLOC: u64 = gigabytes(1);

struct Heap(RefCell<buddy_system_allocator::Heap>);

pub fn init() -> Result<(), MemoryError> {
    assert!(MIN_HEAP_ALLOC.is_power_of_two() && MAX_HEAP_ALLOC.is_power_of_two());

    debug!("initializing kernel heap");
    grow_heap(MIN_HEAP_ALLOC)?;
    Ok(())
}

/// Parameter is rounded up to nearest number of frames
/// (start addr, length in bytes)
fn grow_heap(bytes: u64) -> Result<(VirtualAddress, u64), MemoryError> {
    let mut space = AddressSpace::current();

    let length = round_up_to(bytes, FRAME_SIZE);
    let frame_count = length / FRAME_SIZE;

    let start_addr = space.find_free_space(
        VirtualAddress::with_literal(VIRT_KERNEL_HEAP_BASE),
        frame_count as usize,
    )?;

    space.map_range(start_addr, length, MapTarget::Any, MapFlags::Writeable)?;

    let end_addr = VirtualAddress::with_literal(start_addr.address() + length);
    debug!(
        "growing kernel heap by adding chunk of {:#x} bytes at {:?}",
        length, start_addr
    );

    unsafe {
        let mut heap = HEAP.0.borrow_mut();
        heap.add_to_heap(start_addr.address() as usize, end_addr.address() as usize);
    }

    Ok((start_addr, length))
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut has_grown = false;

        loop {
            let mut heap = self.0.borrow_mut();
            return match heap.alloc(layout) {
                Ok(ptr) => {
                    #[cfg(feature = "log-heap")]
                    trace!(
                        "allocated {:#x} bytes on the heap at {:?}",
                        layout.size(),
                        ptr.as_ptr()
                    );

                    ptr.as_ptr()
                }
                Err(_) => {
                    if !has_grown {
                        let grow_by = {
                            let size = layout.size() + (1024); // a bit extra for book-keeping
                            size.clamp(MIN_HEAP_ALLOC as usize, MAX_HEAP_ALLOC as usize)
                        };

                        #[cfg(feature = "log-heap")]
                        trace!(
                            "growing heap by {:#x} bytes to try to allocate {:#x} bytes",
                            grow_by,
                            layout.size()
                        );

                        has_grown = true;
                        drop(heap); // taken again to grow heap

                        match grow_heap(grow_by as u64) {
                            Err(err) => {
                                warn!("failed to grow heap: {}", err);
                            }
                            Ok(_) => continue, // try again with bigger heap
                        };
                    }

                    warn!("failed to allocate {:#x} bytes on the heap", layout.size());
                    null_mut()
                }
            };
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        #[cfg(feature = "log-heap")]
        trace!("deallocating heap pointer {:?}", ptr);

        debug_assert!(!ptr.is_null());

        let mut heap = self.0.borrow_mut();
        heap.dealloc(NonNull::new_unchecked(ptr), layout)
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("failed to allocate: {:?}", layout)
}
