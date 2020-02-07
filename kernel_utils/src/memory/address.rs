use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::{Shl, Shr};

use bitfield::*;
use derive_more::*;

/// Bottom 12 bits should be 0 from 4096 alignment
const ADDRESS_SHIFT: u64 = 12;

/// 17 bits of sign extension on virtual addresses
const SIGN_EXTEND: u64 = 17;

// TODO constructor rather than pub
#[derive(Copy, Clone, Add)]
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "VirtualAddress({:#010x})", self.0)
    }
}

impl VirtualAddress {
    pub fn new(addr: u64) -> Self {
        let addr = addr >> SIGN_EXTEND;
        let addr = addr << SIGN_EXTEND;
        Self(addr)
    }

    pub fn pml4t_offset(self) -> u16 {
        self.0.bit_range(47, 39)
    }

    pub fn pdp_offset(self) -> u16 {
        self.0.bit_range(38, 30)
    }

    pub fn pd_offset(self) -> u16 {
        self.0.bit_range(29, 21)
    }

    pub fn pt_offset(self) -> u16 {
        self.0.bit_range(12, 20)
    }

    pub fn page_offset_4kb(self) -> u16 {
        self.0.bit_range(11, 0)
    }

    pub fn page_offset_2mb(self) -> u16 {
        self.0.bit_range(20, 0)
    }

    pub fn page_offset_1gb(self) -> u16 {
        self.0.bit_range(29, 0)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Add)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    pub fn from_4096_aligned(addr: u64) -> Self {
        Self(addr.shl(ADDRESS_SHIFT))
    }

    pub fn to_4096_aligned(self) -> u64 {
        self.0.shr(ADDRESS_SHIFT)
    }

    /// # Warning
    /// Null pointers casted to references are UB! It will play havoc with Options and Boxes where
    /// they use the null case for None/null detection!
    pub unsafe fn cast_mut<'a, T>(self) -> &'a mut T {
        &mut *(self.0 as *mut T)
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "PhysicalAddress({:#010x})", self.0)
    }
}
