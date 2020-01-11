use core::fmt::{Debug, Error as FmtError, Formatter};

use bitfield::BitRange;
use core::ops::Shl;
use enumflags2::BitFlags;
use modular_bitfield::{bitfield, prelude::*, FromBits};

// TODO constructor rather than pub
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "VirtualAddress({:#010x})", self.0)
    }
}

#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

impl PhysicalAddress {
    pub fn from_4096_aligned(addr: u64) -> Self {
        Self(addr.shl(ADDRESS_SHIFT))
    }
}

impl Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "PhysicalAddress({:#010x})", self.0)
    }
}

/// Bottom 12 bits should be 0 from 4096 alignment
const ADDRESS_SHIFT: u64 = 12;

fn pml4() -> PhysicalAddress {
    let value: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r" (value));
    }

    let addr: u64 = value.bit_range(51, 12);
    PhysicalAddress::from_4096_aligned(addr)
}

#[derive(BitFlags, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum PageTableFlag {
    Present = 0b00000001,
    Write = 0b00000010,
    User = 0b00000100,
    WriteThrough = 0b00001000,
    CacheDisable = 0b00010000,
    Accessed = 0b00100000,

    /// Only present in lowest level of hierarchy
    Dirty = 0b01000000,
    PageSize = 0b10000000,
}

#[repr(transparent)]
pub struct PageTableFlags(BitFlags<PageTableFlag>);

impl Specifier for PageTableFlags {
    const BITS: usize = 8;
    type Base = u8;
    type Face = u8;
}

impl Debug for PageTableFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "PageTableFlags(")?;
        if self.0.contains(PageTableFlag::Present) {
            write!(f, "PRESENT")?;
        } else {
            write!(f, "NOT_PRESENT")?;
        }

        if self.0.contains(PageTableFlag::Write) {
            write!(f, " | READ_WRITE")?;
        } else {
            write!(f, " | READ_ONLY")?;
        }

        if self.0.contains(PageTableFlag::User) {
            write!(f, " | USER")?;
        }

        if self.0.contains(PageTableFlag::WriteThrough) {
            write!(f, " | WRITE_THROUGH")?;
        }

        if self.0.contains(PageTableFlag::CacheDisable) {
            write!(f, " | CACHE_DISABLED")?;
        }

        if self.0.contains(PageTableFlag::Accessed) {
            write!(f, " | ACCESSED")?;
        }

        if self.0.contains(PageTableFlag::Dirty) {
            write!(f, " | DIRTY")?;
        }

        if self.0.contains(PageTableFlag::PageSize) {
            write!(f, " | BIG_PAGE")?;
        }

        write!(f, ")")?;
        Ok(())
    }
}

#[bitfield]
#[derive(Copy, Clone)]
pub struct PageTableEntry {
    flags: PageTableFlags,

    global: B1,

    /// Can be used by kernel
    unused1: B3,

    addr: B40,

    /// Can be used by kernel
    unused2: B11,

    no_execute: B1,
}

impl PageTableEntry {
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from_4096_aligned(self.get_addr())
    }

    pub fn flags(&self) -> PageTableFlags {
        // TODO why is this necessary!?!
        PageTableFlags(BitFlags::from_bits_truncate(self.get_flags()))
    }

    pub fn present(&self) -> bool {
        self.flags().0.contains(PageTableFlag::Present)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{:?} {:?}", self.address(), self.flags())
    }
}

const PAGE_TABLE_ENTRY_COUNT: usize = 512;

#[derive(Clone)]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
}

impl Debug for PageTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let sep = if f.alternate() { "\n    " } else { ", " };
        write!(f, "PageTable(")?;
        for (i, e) in self.entries.iter().enumerate().filter(|(_, e)| e.present()) {
            write!(f, "{}{:03} -> {:?}", sep, i, e)?;
        }

        write!(f, "{})", sep)
    }
}

impl PageTable {
    pub fn load() -> &'static Self {
        let addr = pml4().0 as *mut PageTable;
        unsafe { &*addr }
    }
}
