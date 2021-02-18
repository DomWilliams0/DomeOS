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
    /// Physical address should be of type T
    pub unsafe fn as_mut<'a, T>(self) -> &'a mut T {
        self.0.cast_mut()
    }

    pub fn zero(&self) {
        let ptr = if cfg!(test) {
            (self.0).0 as *mut u8
        } else {
            let virt = VirtualAddress::from_physical(self.address());
            virt.as_ptr()
        };

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, FRAME_SIZE as usize) };

        slice.fill(0);
    }
}
