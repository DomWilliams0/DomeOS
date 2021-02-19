use core::fmt::{Debug, Error as FmtError, Formatter};
use core::marker::PhantomData;

use crate::address::{PhysicalAddress, VirtualAddress};
use crate::entry::CommonEntry;
use crate::{HasTable, PageTableHierarchy};

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

    pub fn entries(&self) -> impl Iterator<Item = CommonEntry<'p, P>> + '_ {
        self.entries.iter().copied()
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut CommonEntry<'p, P>> + '_ {
        self.entries.iter_mut()
    }

    pub fn entry(&self, idx: impl EntryIndex<'p, P>) -> &CommonEntry<'p, P> {
        let e = idx.index() as usize;
        debug_assert!(e < self.entries.len(), "entry index {} out of range", e);

        let identity_mapped_physical = VirtualAddress::is_identity_mapped_physical(self);

        // safety: not dereferencing physical memory
        let entry = unsafe { self.entries.get_unchecked(e) };

        if !identity_mapped_physical {
            // is not currently accessible, needs to be offset into identity mapped region
            let entry_virt =
                VirtualAddress::from_physical(PhysicalAddress(entry as *const _ as u64));
            unsafe { &*entry_virt.as_ptr() }
        } else {
            // already accessible
            entry
        }
    }

    /// If this table is in physical memory, convert the entry address into accessible identity
    /// mapped virtual memory. Otherwise returns it as-is
    pub fn entry_mut(&mut self, idx: impl EntryIndex<'p, P>) -> &mut CommonEntry<'p, P> {
        let identity_mapped_physical = VirtualAddress::is_identity_mapped_physical(self);

        // entry may be an inaccessible physical reference but we're not dereferencing it
        let entry = self.entry_physical_mut(idx);

        if !identity_mapped_physical {
            // is not currently accessible, needs to be offset into identity mapped region
            let entry_virt = VirtualAddress::from_physical(PhysicalAddress(entry as *mut _ as u64));
            unsafe { &mut *entry_virt.as_ptr() }
        } else {
            // already accessible
            entry
        }
    }

    pub fn entry_physical_mut(&mut self, idx: impl EntryIndex<'p, P>) -> &mut CommonEntry<'p, P> {
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
