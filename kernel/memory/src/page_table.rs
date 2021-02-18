use core::fmt::{Debug, Error as FmtError, Formatter};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use enumflags2::BitFlags;

use common::*;

use crate::address::{PhysicalAddress, VirtualAddress};
use crate::entry::{CommonEntry, PageTableBits, PageTableFlag};
use crate::{Frame, HasTable, PageTableHierarchy};

pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

#[derive(Clone)]
#[repr(C)]
pub struct PageTable<'p, P: PageTableHierarchy<'p>> {
    entries: [CommonEntry<'p, P>; PAGE_TABLE_ENTRY_COUNT],
    _phantom: PhantomData<&'p P>,
}

pub trait EntryIndex<'p, P: PageTableHierarchy<'p>> {
    fn index(self) -> u16;
}

impl<'p, P: PageTableHierarchy<'p>> Default for PageTable<'p, P> {
    fn default() -> Self {
        Self {
            entries: [CommonEntry::default(); PAGE_TABLE_ENTRY_COUNT],
            _phantom: PhantomData,
        }
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

impl<'p, P: HasTable<'p>> EntryIndex<'p, P> for VirtualAddress {
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::{CommonEntry, PageTable};
    use crate::{Frame, P1, P2, P3, P4};

    #[test]
    fn ensure_sizes() {
        assert_eq!(size_of::<CommonEntry<()>>(), 8);
        assert_eq!(size_of::<PageTable<P4>>(), 4096);
        assert_eq!(size_of::<PageTable<P3>>(), 4096);
        assert_eq!(size_of::<PageTable<P2>>(), 4096);
        assert_eq!(size_of::<PageTable<P1>>(), 4096);
        assert_eq!(size_of::<PageTable<Frame>>(), 4096);
    }

    #[test]
    fn nx() {
        let mut e = CommonEntry::<P4>::zeroed();
        assert_eq!(0u64, unsafe { std::mem::transmute(e.clone()) });

        e.set_nx(true);
        assert_eq!(0x8000000000000000u64, unsafe { std::mem::transmute(e) });
    }
}
