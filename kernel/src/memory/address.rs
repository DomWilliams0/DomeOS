use core::fmt::{Debug, Error as FmtError, Formatter};

use core::ops::Shl;
use derive_more::*;

/// Bottom 12 bits should be 0 from 4096 alignment
const ADDRESS_SHIFT: u64 = 12;

// TODO constructor rather than pub
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "VirtualAddress({:#010x})", self.0)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Add)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    pub fn from_4096_aligned(addr: u64) -> Self {
        Self(addr.shl(ADDRESS_SHIFT))
    }

    pub unsafe fn cast<'a, T>(self) -> &'a T {
        &*(self.0 as *const T)
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "PhysicalAddress({:#010x})", self.0)
    }
}
