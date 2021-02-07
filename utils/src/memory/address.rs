use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::{Add, AddAssign, Shl, Shr};

use derive_more::*;

/// Bottom 12 bits should be 0 from 4096 alignment
const ADDRESS_SHIFT: u64 = 12;

/// 17 bits of sign extension on virtual addresses
const SIGN_EXTEND: u64 = 17;

/// Each table index is 9 bits
const OFFSET_SHIFT: u64 = 9;

/// Mask to extract a 9 bit offset from virtual addresses
const OFFSET_MASK: u64 = (1 << OFFSET_SHIFT) - 1;

// TODO constructor rather than pub
#[derive(Copy, Clone, Add, AddAssign)]
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "v{:#010x}", self.0)
    }
}

impl VirtualAddress {
    pub fn new(addr: u64) -> Self {
        let sign_extend_mask = 1 << (SIGN_EXTEND - 1);
        let addr = (addr.wrapping_mul(sign_extend_mask) as i64 / sign_extend_mask as i64) as u64;
        Self(addr)
    }

    /// P4
    pub fn pml4t_offset(self) -> u16 {
        ((self.0 >> 39) & OFFSET_MASK) as u16
    }

    /// P3
    pub fn pdp_offset(self) -> u16 {
        ((self.0 >> 30) & OFFSET_MASK) as u16
    }

    /// P2
    pub fn pd_offset(self) -> u16 {
        ((self.0 >> 21) & OFFSET_MASK) as u16
    }

    /// P1
    pub fn pt_offset(self) -> u16 {
        ((self.0 >> 12) & OFFSET_MASK) as u16
    }

    /// P1
    pub fn page_offset_4kb(self) -> u16 {
        let mask = (1 << ADDRESS_SHIFT) - 1;
        (self.0 & mask) as u16
    }

    /// P2
    pub fn page_offset_2mb(self) -> u32 {
        let mask = (1 << (ADDRESS_SHIFT + OFFSET_SHIFT)) - 1;
        (self.0 & mask) as u32
    }

    /// P3
    pub fn page_offset_1gb(self) -> u64 {
        let mask = (1 << (ADDRESS_SHIFT + OFFSET_SHIFT + OFFSET_SHIFT)) - 1;
        (self.0 & mask) as u64
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Add)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    /// Shifts left 12 bytes
    pub fn from_4096_aligned(addr: u64) -> Self {
        Self(addr.shl(ADDRESS_SHIFT))
    }

    pub fn to_4096_aligned(self) -> u64 {
        self.0.shr(ADDRESS_SHIFT)
    }

    /// # Safety
    /// Null pointers casted to references are UB! It will play havoc with Options and Boxes where
    /// they use the null case for None/null detection!
    pub unsafe fn cast_mut<'a, T>(self) -> &'a mut T {
        &mut *(self.0 as *mut T)
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "p{:#010x}", self.0)
    }
}

impl Add<u64> for PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for PhysicalAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl<T> From<&T> for PhysicalAddress {
    fn from(t: &T) -> Self {
        Self(t as *const _ as u64)
    }
}

impl Add<u64> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<u64> for VirtualAddress {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::address::VirtualAddress;

    #[test]
    fn virtaddr_offsets() {
        let addr = VirtualAddress::new(
            0b1010_1010_1111_0011_1011_1100_1111_1100_0001_0010_1100_1010_1010_0011_1011_1011,
        );

        // sign extension
        assert_eq!(
            addr.0,
            0b1111_1111_1111_1111_1011_1100_1111_1100_0001_0010_1100_1010_1010_0011_1011_1011
        );

        assert_eq!(addr.pml4t_offset(), 0b1011_1100_1);
        assert_eq!(addr.pdp_offset(), 0b111_1100_00);
        assert_eq!(addr.pd_offset(), 0b01_0010_110);
        assert_eq!(addr.pt_offset(), 0b0_1010_1010);
        assert_eq!(addr.page_offset_4kb(), 0b0011_1011_1011);
        assert_eq!(addr.page_offset_2mb(), 0b0_1010_1010_0011_1011_1011);
        assert_eq!(
            addr.page_offset_1gb(),
            0b01_0010_1100_1010_1010_0011_1011_1011
        );
    }
}
