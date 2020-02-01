use core::fmt::{Debug, Error as FmtError, Formatter};

use bitfield::BitRange;
use crate::memory::{PhysicalAddress, VirtualAddress};
use enumflags2::BitFlags;
use modular_bitfield::{bitfield, prelude::*, FromBits};

#[derive(BitFlags, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum PageTableFlag {
    Present = 1 << 0,
    Write = 1 << 1,
    User = 1 << 2,
    WriteThrough = 1 << 3,
    CacheDisable = 1 << 4,
    Accessed = 1 << 5,

    /// Only present in lowest level of hierarchy
    Dirty = 1 << 6,

    /// Used in 2 middle layers: if set it's the bottom
    /// of the hierarchy
    PageSize = 1 << 7,
    /*
        /// Only present in lowest level of hierarchy, if set
        /// TLB wont be invalidated (requires CR4.PGE)
        Global = 1<<8,

        NoExecute = 1<<63,
    */
}

// page map level 4
// page directory pointer entry
// page directory entry
// page table entry

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
struct CommonEntry {
    flags: PageTableFlags,

    global: B1,

    /// Can be used by kernel
    unused1: B3,

    addr: B40,

    /// Can be used by kernel
    unused2: B11,

    no_execute: B1,
}

impl CommonEntry {
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

    pub fn huge_pages(&self) -> bool {
        self.flags().0.contains(PageTableFlag::PageSize)
    }
}

impl Debug for CommonEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{:?} {:?}", self.address(), self.flags())
    }
}

const PAGE_TABLE_ENTRY_COUNT: usize = 512;

#[derive(Clone)]
#[repr(C)]
pub struct PageTable {
    entries: [CommonEntry; PAGE_TABLE_ENTRY_COUNT],
}

impl Debug for PageTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let sep = if f.alternate() { "\n    " } else { ", " };
        write!(f, "PageTable(")?;
        for (i, e) in self.present_entries() {
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

    pub fn present_entries(&self) -> impl Iterator<Item = (usize, &CommonEntry)> {
        self.entries.iter().enumerate().filter(|(_, e)| e.present())
    }
}

fn pml4() -> PhysicalAddress {
    let value: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r" (value));
    }

    let addr: u64 = value.bit_range(51, 12);
    PhysicalAddress::from_4096_aligned(addr)
}

pub mod hierarchy {
    use crate::memory::page_table::{pml4, CommonEntry, PageTable, PAGE_TABLE_ENTRY_COUNT};
    use crate::memory::{PhysicalAddress, VirtualAddress};
    use log::*;

    /// PML4E
    pub enum P4 {
        P3(PhysicalAddress),
    }

    /// PDPE
    pub enum P3 {
        P2(PhysicalAddress),
        Huge1GB,
    }

    /// PDE
    pub enum P2 {
        P1(PhysicalAddress),
        Huge2MB,
    }

    /// PTE
    pub enum P1 {
        Page(PhysicalAddress),
    }

    pub fn walk_active_page_hierarchy() {
        assert_eq!(core::mem::size_of::<CommonEntry>(), 8);

        // get pml4 from cr3
        let pml4: &PageTable = unsafe { pml4().cast() };

        for (i, pml4e) in pml4.present_entries() {
            info!("pml4e #{}: {:#?}", i, pml4e);

            let pdpo: &PageTable = unsafe { pml4e.address().cast() };
            for (i, pdpe) in pdpo.present_entries() {
                info!("  pdpe #{}: {:#?}", i, pdpe);

                if !pdpe.huge_pages() {
                    let pdo: &PageTable = unsafe { pdpe.address().cast() };
                    for (i, pde) in pdo.present_entries() {
                        info!("    pde #{}: {:#?}", i, pde);

                        if !pde.huge_pages() {
                            let pto: &PageTable = unsafe { pde.address().cast() };
                            for (i, pte) in pto.present_entries() {
                                info!("      pte #{}: {:#?}", i, pte);
                            }
                        }
                    }
                }
            }
        }
    }
}

const fn terabytes(n: u64) -> u64 {
    n * (1 << 40)
}

/// Start of direct physical mapping
const PHYSICAL_MAPPING_OFFSET: VirtualAddress = VirtualAddress(0xffffff00_00000000);

/// Size of direct physical mapping
const PHYSICAL_MAPPING_LENGTH: u64 = terabytes(64);

/// Start of kernel code mapping
const KERNEL_START_ADDR: VirtualAddress = VirtualAddress(0xffff0000_00000000);
