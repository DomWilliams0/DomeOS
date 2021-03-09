use crate::address::{PhysicalAddress, VirtualAddress};
use crate::FRAME_SIZE;

#[derive(Copy, Clone)]
pub struct PhysicalFrame(PhysicalAddress);

impl PhysicalFrame {
    /// # Safety
    /// Physical address must be valid and accessible through the identity mapped region
    pub unsafe fn new(addr: PhysicalAddress) -> Self {
        Self(addr)
    }

    pub const fn address(self) -> PhysicalAddress {
        self.0
    }

    /// # Safety
    /// Physical address should be of type T. Returned value is a **physical** reference
    pub unsafe fn as_mut<'a, T>(self) -> &'a mut T {
        self.0.cast_mut()
    }

    /// Converts physical address to accessible virtual first
    pub fn zero(&self) {
        let ptr = if cfg!(test) {
            (self.0).0 as *mut u8
        } else {
            let virt = VirtualAddress::from_physical(self.address());
            virt.as_ptr()
        };

        unsafe {
            core::intrinsics::volatile_set_memory(ptr, 0, FRAME_SIZE as usize);
        }
    }
    /// # Safety
    /// Must ensure it is writeable
    pub unsafe fn zero_in_place(&self) {
        let ptr = (self.0).0 as *mut u8;

        core::intrinsics::volatile_set_memory(ptr, 0, FRAME_SIZE as usize);
    }
}
