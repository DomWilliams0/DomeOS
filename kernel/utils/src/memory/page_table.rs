use core::fmt::{Debug, Error as FmtError, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use c2rust_bitfields::BitfieldStruct;
use enumflags2::BitFlags;

use crate::memory::address::{PhysicalAddress, VirtualAddress};
use crate::memory::hierarchy::PageTableHierarchy;
use crate::memory::Frame;
use crate::KernelResult;

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

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
    HugePages = 1 << 7,
    // in bits field instead
    //
    // /// Only present in lowest level of hierarchy, if set
    // /// TLB wont be invalidated (requires CR4.PGE)
    // Global = 1 << 8,
    //
    // NoExecute = 1 << 63,
}

// page map level 4
// page directory pointer entry
// page directory entry
// page table entry

#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct PageTableFlags(BitFlags<PageTableFlag>);

// Copy and Clone implemented manually
/// Type param is the type of the table this POINTS to
#[repr(C)]
pub struct CommonEntry<'p, P> {
    // TODO fix flags/bits rubbish
    flags: PageTableFlags,
    bits: EntryBits,

    _phantom: PhantomData<&'p P>,
}

pub struct EntryBuilder<'e, 'p, P> {
    flags: BitFlags<PageTableFlag>,
    bits: EntryBits,
    entry: Option<&'e mut CommonEntry<'p, P>>,
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

#[derive(Clone)]
#[repr(C)]
pub struct PageTable<'p, P: PageTableHierarchy<'p>> {
    entries: [CommonEntry<'p, P>; PAGE_TABLE_ENTRY_COUNT],
    _phantom: PhantomData<&'p P>,
}

pub trait EntryIndex<'p, P: PageTableHierarchy<'p>> {
    fn index(self) -> u16;
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

impl<'p, P: PageTableHierarchy<'p>> Default for CommonEntry<'p, P> {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl<'p, P: PageTableHierarchy<'p>> Default for PageTable<'p, P> {
    fn default() -> Self {
        Self {
            entries: [CommonEntry::default(); PAGE_TABLE_ENTRY_COUNT],
            _phantom: PhantomData,
        }
    }
}

impl<'e, 'p, P> Default for EntryBuilder<'e, 'p, P> {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            bits: Default::default(),
            entry: None,
        }
    }
}

impl<'e, 'p, P> EntryBuilder<'e, 'p, P> {
    pub fn with_entry(current: &'e mut CommonEntry<'p, P>) -> Self {
        Self {
            flags: current.flags.0,
            bits: current.bits,
            entry: Some(current),
        }
    }

    pub fn with_zeroed_entry(current: &'e mut CommonEntry<'p, P>) -> Self {
        Self {
            entry: Some(current),
            ..Self::default()
        }
    }

    pub fn writeable(mut self) -> Self {
        self.flags.insert(PageTableFlag::Write);
        self
    }

    pub fn read_only(mut self) -> Self {
        self.flags.remove(PageTableFlag::Write);
        self
    }
    pub fn user(mut self) -> Self {
        self.flags.insert(PageTableFlag::User);
        self
    }

    pub fn supervisor(mut self) -> Self {
        self.flags.remove(PageTableFlag::User);
        self
    }
    pub fn writethrough(mut self) -> Self {
        self.flags.insert(PageTableFlag::WriteThrough);
        self
    }

    pub fn not_writethrough(mut self) -> Self {
        self.flags.remove(PageTableFlag::WriteThrough);
        self
    }
    pub fn cache_disable(mut self) -> Self {
        self.flags.insert(PageTableFlag::CacheDisable);
        self
    }

    pub fn not_cache_disable(mut self) -> Self {
        self.flags.remove(PageTableFlag::CacheDisable);
        self
    }

    pub fn accessed(mut self) -> Self {
        self.flags.insert(PageTableFlag::Accessed);
        self
    }

    pub fn not_accessed(mut self) -> Self {
        self.flags.remove(PageTableFlag::Accessed);
        self
    }
    pub fn dirty(mut self) -> Self {
        self.flags.insert(PageTableFlag::Dirty);
        self
    }

    pub fn not_dirty(mut self) -> Self {
        self.flags.remove(PageTableFlag::Dirty);
        self
    }

    pub fn huge(mut self) -> Self {
        self.flags.insert(PageTableFlag::HugePages);
        self
    }

    pub fn not_huge(mut self) -> Self {
        self.flags.remove(PageTableFlag::HugePages);
        self
    }

    pub fn present(mut self) -> Self {
        self.flags.insert(PageTableFlag::Present);
        self
    }

    pub fn not_present(mut self) -> Self {
        self.flags.remove(PageTableFlag::Present);
        self
    }

    pub fn executable(mut self) -> Self {
        self.bits.set_no_execute(0);
        self
    }

    pub fn not_executable(mut self) -> Self {
        self.bits.set_no_execute(1);
        self
    }

    pub fn global(mut self) -> Self {
        self.bits.set_global(1);
        self
    }

    pub fn not_global(mut self) -> Self {
        self.bits.set_global(0);
        self
    }

    pub fn address(mut self, addr: PhysicalAddress) -> Self {
        self.bits.set_addr(addr.to_4096_aligned());
        self
    }

    /// If [with_entry] was used then the reference is written to as well
    pub fn build(self) -> CommonEntry<'e, P> {
        let entry = CommonEntry {
            flags: PageTableFlags(self.flags),
            bits: self.bits,
            _phantom: PhantomData,
        };

        if let Some(e) = self.entry {
            *e = entry;
        }

        entry
    }
}

impl<'p, P: PageTableHierarchy<'p>> CommonEntry<'p, P> {
    pub fn zeroed() -> Self {
        Self {
            flags: Default::default(),
            bits: Default::default(),
            _phantom: Default::default(),
        }
    }

    /// Keeps existing flags
    pub fn modify<'e>(&'e mut self) -> EntryBuilder<'e, 'p, P> {
        EntryBuilder::with_entry(self)
    }

    /// Clears all bits
    pub fn replace<'e>(&'e mut self) -> EntryBuilder<'e, 'p, P> {
        EntryBuilder::with_zeroed_entry(self)
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from_4096_aligned(self.bits.addr())
    }

    pub fn present(&self) -> bool {
        self.flags.0.contains(PageTableFlag::Present)
    }

    pub fn huge_pages(&self) -> bool {
        // TODO P::supports_huge_pages to hopefully optimise this out for p4/p3
        self.flags.0.contains(PageTableFlag::HugePages)
    }

    pub fn traverse(&self) -> KernelResult<P> {
        assert!(self.present());

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

    #[deprecated]
    pub fn copy_to(&self, other: &mut Self) {
        self.entries()
            .zip(other.entries.iter_mut())
            .for_each(|(src, dst)| *dst = src);
    }

    pub fn entries(&self) -> impl Iterator<Item = CommonEntry<'p, P>> + '_ {
        self.entries.iter().copied()
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut CommonEntry<'p, P>> + '_ {
        self.entries.iter_mut()
    }

    pub fn entry(&self, idx: impl EntryIndex<'p, P>) -> &CommonEntry<'p, P> {
        let e = idx.index() as usize;
        debug_assert!(e < self.entries.len(), "entry index {} out of range", e);

        // safety: entry_index always returns value in range
        unsafe { self.entries.get_unchecked(e) }
    }

    pub fn entry_mut(&mut self, idx: impl EntryIndex<'p, P>) -> &mut CommonEntry<'p, P> {
        let e = idx.index() as usize;
        debug_assert!(e < self.entries.len(), "entry index {} out of range", e);

        // safety: entry_index always returns value in range
        unsafe { self.entries.get_unchecked_mut(e) }
    }
}
impl<'p, P: PageTableHierarchy<'p>> EntryIndex<'p, P> for u16 {
    fn index(self) -> u16 {
        self
    }
}

impl<'p, P: PageTableHierarchy<'p>> EntryIndex<'p, P> for VirtualAddress {
    fn index(self) -> u16 {
        P::entry_index(self)
    }
}

impl<'p, P: PageTableHierarchy<'p>, E: EntryIndex<'p, P>> Index<E> for PageTable<'p, P> {
    type Output = CommonEntry<'p, P>;

    fn index(&self, index: E) -> &Self::Output {
        self.entry(index)
    }
}

impl<'p, P: PageTableHierarchy<'p>, E: EntryIndex<'p, P>> IndexMut<E> for PageTable<'p, P> {
    fn index_mut(&mut self, index: E) -> &mut Self::Output {
        self.entry_mut(index)
    }
}

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

        if self.0.contains(PageTableFlag::HugePages) {
            write!(f, " | HUGE")?;
        }

        write!(f, ")")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::memory::hierarchy::{Frame, P1, P2, P3};
    use crate::memory::page_table::{CommonEntry, PageTable};
    use crate::memory::P4;

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
        let mut e = CommonEntry::<P4>::zeroed();
        assert_eq!(0u64, unsafe { std::mem::transmute(e.clone()) });

        e.bits.set_no_execute(true as u16);
        assert_eq!(0x8000000000000000u64, unsafe { std::mem::transmute(e) });
    }
}
