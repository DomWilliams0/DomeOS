use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::{Add, AddAssign, Shl, Shr};

use crate::{VIRT_KERNEL_BASE, VIRT_PHYSICAL_BASE, VIRT_PHYSICAL_MAX};
use common::*;

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
    pub const fn new(addr: u64) -> Self {
        let sign_extend_mask = 1 << (SIGN_EXTEND - 1);
        let addr = (addr.wrapping_mul(sign_extend_mask) as i64 / sign_extend_mask as i64) as u64;
        Self(addr)
    }

    /// Panics if value changes from 48-bit sign extension
    pub fn new_checked(addr: u64) -> Self {
        let virt = Self::new(addr);
        assert_eq!(
            virt.0, addr,
            "virtual address {:#x} is not representable (becomes {:?})",
            addr, virt
        );
        virt
    }

    pub fn from_indices(p4: u16, p3: u16, p2: u16, p1: u16) -> Self {
        let p4_masked = p4 as u64 & OFFSET_MASK;
        let p3_masked = p3 as u64 & OFFSET_MASK;
        let p2_masked = p2 as u64 & OFFSET_MASK;
        let p1_masked = p1 as u64 & OFFSET_MASK;

        if cfg!(debug_assertions) {
            assert_eq!(p4 as u64, p4_masked);
            assert_eq!(p3 as u64, p3_masked);
            assert_eq!(p2 as u64, p2_masked);
            assert_eq!(p1 as u64, p1_masked);
        }

        VirtualAddress::new(
            (p4_masked << 39) | (p3_masked << 30) | (p2_masked << 21) | (p1_masked << 12),
        )
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

    pub fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Adds physical identity base
    pub fn from_physical(addr: PhysicalAddress) -> VirtualAddress {
        if cfg!(test) {
            return Self::new_checked(addr.0);
        }

        let addr = addr.0.checked_add(VIRT_PHYSICAL_BASE).unwrap_or_else(|| {
            panic!(
                "overflow calculating identity mapped address for {:?}",
                addr
            )
        });
        Self::new_checked(addr)
    }

    /// Adds kernel higher half base
    pub fn from_kernel_code<T>(addr: *const T) -> *const T {
        (addr as u64)
            .checked_add(VIRT_KERNEL_BASE)
            .unwrap_or_else(|| panic!("overflow adding virtual kernel base offset to {:?}", addr))
            as *mut T
    }

    /// Subtracts physical identity base
    pub fn to_physical(self) -> PhysicalAddress {
        if cfg!(test) {
            return PhysicalAddress(self.0);
        }

        let addr = self
            .0
            .checked_sub(VIRT_PHYSICAL_BASE)
            .unwrap_or_else(|| panic!("overflow calculating physical address for {:?}", self));
        PhysicalAddress(addr)
    }

    /// True if in the physical identity mapped range
    pub fn is_identity_mapped_physical<T>(thing: &T) -> bool {
        let addr = thing as *const T as u64;
        (VIRT_PHYSICAL_BASE..VIRT_PHYSICAL_MAX).contains(&addr)
    }

    pub fn round_up_to(self, multiple: u64) -> Self {
        Self::new_checked(round_up_to(self.0, multiple))
    }

    pub fn round_down_to(self, multiple: u64) -> Self {
        Self::new_checked(round_down_to(self.0, multiple))
    }

    pub fn log_all_offsets(self) {
        debug!(
            "{:?} -> {}, {}, {}, {}",
            self,
            self.pml4t_offset(),
            self.pdp_offset(),
            self.pd_offset(),
            self.pt_offset()
        );
    }

    pub fn address(self) -> u64 {
        self.0
    }
}

pub fn round_up_to(value: u64, multiple: u64) -> u64 {
    assert!(multiple.is_power_of_two());
    let val = (value + multiple - 1) as i64 & -(multiple as i64);
    val as u64
}

pub fn round_down_to(value: u64, multiple: u64) -> u64 {
    assert!(multiple.is_power_of_two());
    value & !(multiple - 1)
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

    pub fn address(self) -> u64 {
        self.0
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
    use crate::address::{VirtualAddress, ADDRESS_SHIFT, OFFSET_MASK, OFFSET_SHIFT};
    use crate::FRAME_SIZE;

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

        let addr = addr.round_down_to(FRAME_SIZE);
        let from_indices = VirtualAddress::from_indices(
            addr.pml4t_offset(),
            addr.pdp_offset(),
            addr.pd_offset(),
            addr.pt_offset(),
        );
        assert_eq!(
            from_indices.address(),
            addr.address(),
            "expected={:#x}, actual={:#x}",
            addr.address(),
            from_indices.address()
        );
    }

    #[test]
    fn round_up() {
        assert_eq!(
            VirtualAddress::new_checked(0xeff00).round_up_to(0x1000).0,
            0xf0000
        );
        assert_eq!(
            VirtualAddress::new_checked(0x1000).round_up_to(0x1000).0,
            0x1000
        ); // no change
        assert_eq!(
            VirtualAddress::new_checked(0x5).round_up_to(0x1000).0,
            0x1000
        );
    }
}
