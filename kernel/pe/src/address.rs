use core::fmt::Debug;
use core::ops::Add;

use common::*;

use core::ops::RangeInclusive;

pub trait Address: Sized + Copy {
    fn with_u64(addr: u64) -> Self;
    fn into_u64(self) -> u64;

    // ----
    fn new<A: Into<u64>>(addr: A) -> Self {
        Self::with_u64(addr.into())
    }
    fn with_usize(addr: usize) -> Self {
        Self::with_u64(addr as u64)
    }

    fn range(self, other: Self) -> RangeInclusive<u64> {
        self.into_u64()..=other.into_u64()
    }

    fn into_usize(self) -> usize {
        self.into_u64() as usize
    }
}

/// Address relative to image base
#[derive(Copy, Clone, Add, Sub, Eq, PartialEq)]
pub struct VirtualAddress(u64);

/// Absolute pointer into the image file
#[derive(Copy, Clone, Add, Sub)]
pub struct FileOffset(u64);

macro_rules! impl_addr {
    ($addr:ty) => {
        impl Address for $addr {
            fn with_u64(addr: u64) -> Self {
                Self(addr)
            }

            fn into_u64(self) -> u64 {
                self.0
            }
        }

        impl Add<usize> for $addr {
            type Output = Self;

            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs as u64)
            }
        }
        impl Debug for $addr {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({:#x})", stringify!($addr), self.0)
            }
        }
    };
}

impl_addr!(VirtualAddress);
impl_addr!(FileOffset);

impl VirtualAddress {
    pub fn to_file_offset_in_section(
        self,
        section_addr: VirtualAddress,
        section_raw_data: FileOffset,
    ) -> FileOffset {
        let va = self - section_addr;
        let offset = va.into_usize() + section_raw_data.into_usize();
        FileOffset::with_usize(offset)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Add<u64> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}
