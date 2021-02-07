use core::fmt::{Debug, Error as FmtError, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use c2rust_bitfields::BitfieldStruct;
use enumflags2::BitFlags;

use crate::memory::address::PhysicalAddress;
use crate::memory::hierarchy::PageTableHierarchy;
use crate::memory::Frame;

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
        write!(f, "(")?;
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

// Copy and Clone implemented manually
/// Type param is the type of the table this POINTS to
#[repr(C)]
#[derive(Default)]
pub struct CommonEntry<'p, P> {
    flags: PageTableFlags,
    bits: EntryBits,

    _phantom: PhantomData<&'p P>,
}

#[repr(transparent)]
#[derive(BitfieldStruct, Copy, Clone, Default)]
pub struct EntryBits {
    #[bitfield(name = "global", ty = "u8", bits = "0..=0")]
    #[bitfield(name = "unused1", ty = "u8", bits = "1..=3")]
    #[bitfield(name = "addr", ty = "u64", bits = "4..=43")]
    #[bitfield(name = "unused2", ty = "u16", bits = "44..=54")]
    #[bitfield(name = "no_execute", ty = "u16", bits = "55..=55")]
    global_unused1_addr_unused2_nx: [u8; 7],
}

impl<'p, P> Copy for CommonEntry<'p, P> {}

impl<'p, P> Clone for CommonEntry<'p, P> {
    fn clone(&self) -> Self {
        Self {
            flags: self.flags,
            bits: self.bits,
            _phantom: PhantomData,
        }
    }
}

impl<'p, P: PageTableHierarchy<'p>> CommonEntry<'p, P> {
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from_4096_aligned(self.bits.addr())
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

    pub fn init(
        &mut self,
        address: PhysicalAddress,
        w: Writeable,
        x: Executable,
        overwrite: Overwrite,
    ) {
        if let Overwrite::MustNotExist = overwrite {
            assert!(
                !self.present(),
                "not expected to be present already: {:?}",
                self
            );
        }

        let flags = match w {
            Writeable::Read => PageTableFlag::Present.into(),
            Writeable::Write => PageTableFlag::Present | PageTableFlag::Write,
        };

        let nx = match x {
            Executable::Executable => false,
            Executable::NotExecutable => true,
        };
        self.bits.set_no_execute(nx as u16);

        self.flags.0.insert(flags);
        self.bits.set_addr(address.to_4096_aligned());
    }

    pub fn traverse(&self) -> Option<P> {
        debug_assert!(self.present());

        let address = self.address();

        // safety: P type param ensures target table is correct type
        if self.huge_pages() {
            P::with_frame(Frame(address))
        } else {
            let table = unsafe { self.address().cast_mut() };

            P::with_table(table)
        }
    }
}

impl<'p, P: PageTableHierarchy<'p>> Debug for CommonEntry<'p, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let this = PhysicalAddress(&self as *const _ as u64);
        write!(f, "[{:?} -> {:?} {:?}]", this, self.address(), self.flags)
    }
}

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

#[derive(Clone)]
#[repr(C)]
pub struct PageTable<'p, P: PageTableHierarchy<'p>> {
    entries: [CommonEntry<'p, P>; PAGE_TABLE_ENTRY_COUNT],
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
    pub fn present_entries(&self) -> impl Iterator<Item = (usize, CommonEntry<'p, P>)> + '_ {
        self.entries().enumerate().filter(|(_, e)| e.present())
    }

    pub fn copy_to(&self, other: &mut Self) {
        self.entries()
            .zip(other.entries.iter_mut())
            .for_each(|(src, dst)| *dst = src.clone());
    }

    fn entries(&self) -> impl Iterator<Item = CommonEntry<'p, P>> + '_ {
        self.entries.iter().copied()
    }
}

impl<'p, P: PageTableHierarchy<'p>> Index<u16> for PageTable<'p, P> {
    type Output = CommonEntry<'p, P>;

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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::memory::hierarchy::{Frame, P1, P2, P3};
    use crate::memory::page_table::{CommonEntry, PageTable};

    #[test]
    fn assert_sizes() {
        assert_eq!(size_of::<CommonEntry<()>>(), 8);
        assert_eq!(size_of::<PageTable<P3>>(), 4096);
        assert_eq!(size_of::<PageTable<P2>>(), 4096);
        assert_eq!(size_of::<PageTable<P1>>(), 4096);
        assert_eq!(size_of::<PageTable<Frame>>(), 4096);
    }

    #[test]
    fn nx() {
        let mut e = CommonEntry::<()>::default();
        assert_eq!(0u64, unsafe { std::mem::transmute(e.clone()) });

        e.bits.set_no_execute(true as u16);
        assert_eq!(0x8000000000000000u64, unsafe { std::mem::transmute(e) });
    }
}
