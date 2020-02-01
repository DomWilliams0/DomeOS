use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::{Index, IndexMut};

use bitfield::BitRange;
use core::marker::PhantomData;
use crate::memory::page_table::hierarchy::{P3, P4, PageTableHierarchy};
use crate::memory::PhysicalAddress;
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
#[derive(Clone)]
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

    pub fn set_present(&mut self, present: bool) {
        todo!()
    }
}

impl Debug for CommonEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let this = PhysicalAddress(&self as *const _ as u64);
        write!(f, "[{:?} -> {:?} {:?}]", this, self.address(), self.flags())
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

impl<'p, P: PageTableHierarchy<'p>> Index<u16> for PageTable<'p, P> {
    type Output = CommonEntry;

    fn index(&self, index: u16) -> &Self::Output {
        debug_assert!(
            index < 512,
            "Table index out of range, must be < 512 but is {}",
            index
        );
        &self.entries[index as usize]
    }
}

impl<'p, P: PageTableHierarchy<'p>> IndexMut<u16> for PageTable<'p, P> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        debug_assert!(
            index < 512,
            "Table index out of range, must be < 512 but is {}",
            index
        );
        &mut self.entries[index as usize]
    }
}

fn cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r" (value));
    }
    value
}

pub fn pml4() -> P4<'static> {
    let addr = cr3().bit_range(51, 12);
    let ptr = PhysicalAddress::from_4096_aligned(addr);
    let table = ptr.0 as *mut PageTable<'static, P3<'static>>;
    P4::PML4T(unsafe { &mut *table })
}

pub fn set_pml4(p4: P4<'static>) {
    let P4::PML4T(table) = p4;
    let ptr = PhysicalAddress(table as *const PageTable<'static, P3<'static>> as u64);

    let mut cr3 = cr3();
    cr3.set_bit_range(51, 12, ptr.to_4096_aligned());

    unsafe {
        asm!("mov $0, %cr3" :: "r" (cr3) : "memory");
    }
}

pub mod hierarchy {
    use core::marker::PhantomData;
    use crate::memory::page_table::{CommonEntry, PageTable};
    use crate::memory::{PhysicalAddress, VirtualAddress};

    #[derive(Debug)]
    pub enum ResolveResult<'p, P: PageTableHierarchy<'p>> {
        AnotherLevel {
            next_level: P,
            src_entry: &'p mut CommonEntry,
            _phantom: &'p PhantomData<P>,
        },
        PhysicalFrame(Frame),
    }

    /// Helper to construct `AnotherFrame` variant without faffing around with PhantomData
    impl<'p, P: PageTableHierarchy<'p>> ResolveResult<'p, P> {
        fn another_level(entry: &'p mut CommonEntry) -> Self {
            let table_ptr: &mut PageTable<'p, P::NextLevel> = unsafe { entry.address().cast_mut() };

            ResolveResult::AnotherLevel {
                next_level: P::with_table(table_ptr),
                src_entry: entry,
                _phantom: &PhantomData,
            }
        }

        pub fn unwrap_next_level(self) -> (P, &'p mut CommonEntry) {
            match self {
                ResolveResult::AnotherLevel {
                    next_level,
                    src_entry,
                    ..
                } => (next_level, src_entry),
                _ => panic!("expected AnotherLevel variant"),
            }
        }
    }

    pub trait PageTableHierarchy<'p> {
        type NextLevel: PageTableHierarchy<'p>;
        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel>;
        fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Self;

        fn traverse(self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel>;
    }

    #[derive(Debug)]
    pub enum P4<'p> {
        PML4T(&'p mut PageTable<'p, P3<'p>>),
    }

    #[derive(Debug)]
    pub enum P3<'p> {
        PDPT(&'p mut PageTable<'p, P2<'p>>),
    }

    #[derive(Debug)]
    pub enum P2<'p> {
        PDT(&'p mut PageTable<'p, P1<'p>>),
        Huge1GPage(Frame),
    }

    #[derive(Debug)]
    pub enum P1<'p> {
        PT(&'p mut PageTable<'p, Frame>),
        Huge2MPage(Frame),
    }

    #[derive(Debug)]
    pub struct Frame(PhysicalAddress);

    impl<'p> PageTableHierarchy<'p> for P4<'p> {
        type NextLevel = P3<'p>;

        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel> {
            ResolveResult::another_level(e)
        }

        fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Self {
            P4::PML4T(table)
        }

        fn traverse(self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel> {
            let P4::PML4T(table) = self;
            let entry = &mut table[addr.pml4t_offset()];
            Self::current(entry)
        }
    }

    impl<'p> PageTableHierarchy<'p> for P3<'p> {
        type NextLevel = P2<'p>;

        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel> {
            ResolveResult::another_level(e)
        }

        fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Self {
            P3::PDPT(table)
        }

        fn traverse(self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel> {
            let P3::PDPT(table) = self;
            let entry = &mut table[addr.pdp_offset()];
            Self::current(entry)
        }
    }

    impl<'p> PageTableHierarchy<'p> for P2<'p> {
        type NextLevel = P1<'p>;

        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel> {
            if e.huge_pages() {
                ResolveResult::PhysicalFrame(Frame(e.address()))
            } else {
                ResolveResult::another_level(e)
            }
        }

        fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Self {
            P2::PDT(table)
        }

        fn traverse(self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel> {
            match self {
                P2::PDT(table) => {
                    let entry = &mut table[addr.pd_offset()];
                    Self::current(entry)
                }
                P2::Huge1GPage(frame) => ResolveResult::PhysicalFrame(frame),
            }
        }
    }

    impl<'p> PageTableHierarchy<'p> for P1<'p> {
        type NextLevel = Frame;

        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel> {
            if e.huge_pages() {
                ResolveResult::PhysicalFrame(Frame(e.address()))
            } else {
                ResolveResult::another_level(e)
            }
        }

        fn with_table(table: &'p mut PageTable<'p, Self::NextLevel>) -> Self {
            P1::PT(table)
        }

        fn traverse(self, addr: VirtualAddress) -> ResolveResult<'p, Self::NextLevel> {
            match self {
                P1::PT(table) => {
                    let entry = &mut table[addr.pt_offset()];
                    Self::current(entry)
                }
                P1::Huge2MPage(frame) => ResolveResult::PhysicalFrame(frame),
            }
        }
    }

    impl<'p> PageTableHierarchy<'p> for Frame {
        type NextLevel = Self;

        fn current(e: &'p mut CommonEntry) -> ResolveResult<'p, Self::NextLevel> {
            ResolveResult::PhysicalFrame(Frame(e.address()))
        }

        fn with_table(_: &'p mut PageTable<'p, Self::NextLevel>) -> Self {
            // makes no logical sense
            unreachable!("Already reached the bottom of the hierarchy")
        }

        fn traverse(self, _: VirtualAddress) -> ResolveResult<'p, Self::NextLevel> {
            // TODO correct?
            ResolveResult::PhysicalFrame(self)
        }
    }

    pub fn walk_active_page_hierarchy() {
        assert_eq!(core::mem::size_of::<CommonEntry>(), 8);
        // TODO

        //        let P4::PML4T(table) = pml4();
        //        for (i, e) in table.present_entries() {
        //            info!("pml4e #{}: {:#?}", i, e);
        //
        //            let P3::PDPT(table) = P3::traverse(e);
        //            for (i, e) in table.present_entries() {
        //                info!("  pdpe #{}: {:#?}", i, e);
        //
        //                if let P2::PDT(table) = P2::traverse(e) {
        //                    for (i, e) in table.present_entries() {
        //                        info!("    pde #{}: {:#?}", i, e);
        //
        //                        if let P1::PT(table) = P1::traverse(e) {
        //                            for (i, e) in table.present_entries() {
        //                                info!("      pte #{}: {:#?}", i, e);
        //                            }
        //                        }
        //                    }
        //                }
        //            }
        //        }
    }
}
