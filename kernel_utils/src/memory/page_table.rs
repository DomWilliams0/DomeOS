use core::fmt::{Debug, Error as FmtError, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use c2rust_bitfields::BitfieldStruct;
use enumflags2::BitFlags;

use crate::memory::address::PhysicalAddress;
use crate::memory::page_table::hierarchy::PageTableHierarchy;

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

#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct PageTableFlags(BitFlags<PageTableFlag>);

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

#[derive(Copy, Clone)]
pub enum Writeable {
    Read,
    Write,
}

#[derive(Copy, Clone)]
pub enum Executable {
    Executable,
    NotExecutable,
}

#[derive(Copy, Clone)]
pub enum Overwrite {
    OverwriteExisting,
    MustNotExist,
}

#[repr(C)]
#[derive(BitfieldStruct, Clone, Default)]
pub struct CommonEntry {
    flags: PageTableFlags,

    #[bitfield(name = "global", ty = "u8", bits = "0..=0")]
    #[bitfield(name = "unused1", ty = "u8", bits = "1..=3")]
    #[bitfield(name = "addr", ty = "u64", bits = "4..=43")]
    #[bitfield(name = "unused2", ty = "u16", bits = "44..=54")]
    #[bitfield(name = "no_execute", ty = "u16", bits = "55..=55")]
    global_unused1_addr_unused2_nx: [u8; 7],
}

impl CommonEntry {
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from_4096_aligned(self.addr())
    }
    pub fn present(&self) -> bool {
        self.flags.0.contains(PageTableFlag::Present)
    }

    pub fn huge_pages(&self) -> bool {
        self.flags.0.contains(PageTableFlag::PageSize)
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.flags.0.insert(PageTableFlag::Present)
        } else {
            self.flags.0.remove(PageTableFlag::Present)
        }
    }

    pub fn init(&mut self, address: PhysicalAddress, w: Writeable, x: Executable, overwrite: Overwrite) {
        if let Overwrite::MustNotExist = overwrite {
            assert!(!self.present(), "not expected to be present already: {:?}", self);
        }

        let flags = match w {
            Writeable::Read => PageTableFlag::Present.into(),
            Writeable::Write => PageTableFlag::Present | PageTableFlag::Write,
        };

        let nx = match x {
            Executable::Executable => false,
            Executable::NotExecutable => true,
        };
        self.set_no_execute(nx as u16);

        self.flags.0.insert(flags);
        self.set_addr(address.to_4096_aligned());
    }
}

impl Debug for CommonEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let this = PhysicalAddress(&self as *const _ as u64);
        write!(f, "[{:?} -> {:?} {:?}]", this, self.address(), self.flags)
    }
}

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

#[derive(Clone)]
#[repr(C)]
pub struct PageTable<'p, P: PageTableHierarchy<'p>> {
    entries: [CommonEntry; PAGE_TABLE_ENTRY_COUNT],
    _phantom: PhantomData<&'p P>,
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

    pub fn copy_to(&self, other: &mut Self) {
        self.entries
            .iter()
            .zip(other.entries.iter_mut())
            .for_each(|(src, dst)| *dst = src.clone());
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

pub mod hierarchy {
    use core::marker::PhantomData;

    use crate::memory::address::{PhysicalAddress, VirtualAddress};
    use crate::memory::page_table::{CommonEntry, PageTable};

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
}
#[cfg(test)]
mod tests {
    use crate::memory::page_table::hierarchy::{Frame, P1, P2, P3};
    use crate::memory::page_table::{CommonEntry, PageTable};
    use std::mem::size_of;

    #[test]
    fn assert_sizes() {
        assert_eq!(size_of::<CommonEntry>(), 8);
        assert_eq!(size_of::<PageTable<P3>>(), 4096);
        assert_eq!(size_of::<PageTable<P2>>(), 4096);
        assert_eq!(size_of::<PageTable<P1>>(), 4096);
        assert_eq!(size_of::<PageTable<Frame>>(), 4096);
    }

    #[test]
    fn nx() {
        let mut e = CommonEntry::default();
        assert_eq!(0u64, unsafe { std::mem::transmute(e.clone()) });

        e.set_no_execute(true as u16);
        assert_eq!(0x8000000000000000u64, unsafe { std::mem::transmute(e) });
    }
}
