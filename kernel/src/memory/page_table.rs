use core::fmt::{Debug, Error as FmtError, Formatter};

use bitfield::BitRange;
use core::marker::PhantomData;
use crate::memory::page_table::hierarchy::{P3, P4, PageTableHierarchy};
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
pub struct PageTable<'p, P: PageTableHierarchy<'p>> {
    entries: [CommonEntry; PAGE_TABLE_ENTRY_COUNT],
    _phantom: &'p PhantomData<P>,
}

impl<'p, P: PageTableHierarchy<'p>> Debug for PageTable<'p, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let sep = if f.alternate() { "\n    " } else { ", " };
        write!(f, "PageTable(")?;
        for (i, e) in self.present_entries() {
            write!(f, "{}{:03} -> {:?}", sep, i, e)?;
        }
        write!(f, "{})", sep)
    }
}

impl<'p, P: PageTableHierarchy<'p>> PageTable<'p, P> {
    pub fn present_entries(&self) -> impl Iterator<Item = (usize, &CommonEntry)> {
        self.entries.iter().enumerate().filter(|(_, e)| e.present())
    }
}

pub fn load_current() -> P4<'static> {
    let addr = pml4().0 as *mut PageTable<'static, P3<'static>>;
    let table = unsafe { &*addr };
    P4::PML4T(table)
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
    use crate::memory::page_table::{load_current, CommonEntry, PageTable, PAGE_TABLE_ENTRY_COUNT};
    use crate::memory::PhysicalAddress;
    use log::*;

    pub trait PageTableHierarchy<'p> {
        fn entry(e: &CommonEntry) -> Self;
    }

    pub enum P4<'p> {
        PML4T(&'p PageTable<'p, P3<'p>>),
    }

    pub enum P3<'p> {
        PDPT(&'p PageTable<'p, P2<'p>>),
    }

    pub enum P2<'p> {
        PDT(&'p PageTable<'p, P1<'p>>),
        Huge1GPage(Frame),
    }

    pub enum P1<'p> {
        PT(&'p PageTable<'p, Frame>),
        Huge2MPage(Frame),
    }

    pub struct Frame(PhysicalAddress);

    impl<'p> PageTableHierarchy<'p> for P4<'p> {
        fn entry(e: &CommonEntry) -> Self {
            let table = unsafe { e.address().cast() };
            Self::PML4T(table)
        }
    }

    impl<'p> PageTableHierarchy<'p> for P3<'p> {
        fn entry(e: &CommonEntry) -> Self {
            let table = unsafe { e.address().cast() };
            Self::PDPT(table)
        }
    }

    impl<'p> PageTableHierarchy<'p> for P2<'p> {
        fn entry(e: &CommonEntry) -> Self {
            if e.huge_pages() {
                Self::Huge1GPage(Frame(e.address()))
            } else {
                let table = unsafe { e.address().cast() };
                Self::PDT(table)
            }
        }
    }

    impl<'p> PageTableHierarchy<'p> for P1<'p> {
        fn entry(e: &CommonEntry) -> Self {
            if e.huge_pages() {
                Self::Huge2MPage(Frame(e.address()))
            } else {
                let table = unsafe { e.address().cast() };
                Self::PT(table)
            }
        }
    }

    impl<'p> PageTableHierarchy<'p> for Frame {
        fn entry(e: &CommonEntry) -> Self {
            Self(e.address())
        }
    }

    pub fn walk_active_page_hierarchy() {
        assert_eq!(core::mem::size_of::<CommonEntry>(), 8);

        let P4::PML4T(table) = load_current();
        for (i, e) in table.present_entries() {
            info!("pml4e #{}: {:#?}", i, e);

            let P3::PDPT(table) = P3::entry(e);
            for (i, e) in table.present_entries() {
                info!("  pdpe #{}: {:#?}", i, e);

                if let P2::PDT(table) = P2::entry(e) {
                    for (i, e) in table.present_entries() {
                        info!("    pde #{}: {:#?}", i, e);

                        if let P1::PT(table) = P1::entry(e) {
                            for (i, e) in table.present_entries() {
                                info!("      pte #{}: {:#?}", i, e);
                            }
                        }
                    }
                }
            }
        }
    }
}
