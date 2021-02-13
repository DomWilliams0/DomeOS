use crate::memory::address::PhysicalAddress;

#[derive(Copy, Clone)]
pub struct PhysicalFrame(PhysicalAddress);

impl PhysicalFrame {
    pub fn new(addr: PhysicalAddress) -> Self {
        Self(addr)
    }

    pub fn address(self) -> PhysicalAddress {
        self.0
    }

    /// # Safety
    /// Physical address should be of type T
    pub unsafe fn as_mut<'a, T>(self) -> &'a mut T {
        self.0.cast_mut()
    }
}
