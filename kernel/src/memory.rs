use core::fmt::{Debug, Error, Formatter};

// TODO constructor rather than pub
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtualAddress({:#x})", self.0)
    }
}

#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "PhysicalAddress({:#x})", self.0)
    }
}
