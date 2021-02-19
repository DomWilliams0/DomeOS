use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

use enumflags2::BitFlags;
use modular_bitfield::prelude::*;

use common::*;

use crate::custom_entry::AbsentPageEntry;
use crate::entry_builder::EntryBuilder;
use crate::{Frame, PageTableHierarchy, PhysicalAddress};

#[bitfield]
#[derive(Copy, Clone, Default, Deref, DerefMut)]
pub struct PageTableBits {
    pub present: bool,
    pub writeable: bool,
    pub user: bool,
    pub write_through: bool,
    pub cache_disable: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub huge: bool,
    pub global: bool,
    pub available: B3,
    pub address: B40,
    pub available2: B11,
    pub nx: bool,
}

#[derive(BitFlags, Copy, Clone, Eq, PartialEq)]
#[repr(u64)]
pub enum PageTableFlag {
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

    /// Only present in lowest level of hierarchy, if set
    /// TLB wont be invalidated (requires CR4.PGE)
    Global = 1 << 8,

    NoExecute = 1 << 63,
}

// Copy and Clone implemented manually
/// Type param is the type of the table this POINTS to
#[repr(C)]
#[derive(Deref, DerefMut)]
pub struct CommonEntry<'p, P> {
    #[deref]
    #[deref_mut]
    bits: PageTableBits,
    _phantom: PhantomData<&'p P>,
}

impl<'p, P> Copy for CommonEntry<'p, P> {}

impl<'p, P> Clone for CommonEntry<'p, P> {
    fn clone(&self) -> Self {
        Self {
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

impl<'p, P: PageTableHierarchy<'p>> CommonEntry<'p, P> {
    pub fn zeroed() -> Self {
        Self::new(PageTableBits::default())
    }

    pub fn new(bits: PageTableBits) -> Self {
        Self {
            bits,
            _phantom: Default::default(),
        }
    }

    pub fn as_custom_mut<T: AbsentPageEntry>(&mut self) -> Option<&mut T> {
        let as_u64 = u64::from_le_bytes(self.bits.into_bytes());
        if T::is_self(as_u64) {
            // safety: not present so all other bits are ours for the taking
            Some(unsafe { self.as_custom_unchecked_mut() })
        } else {
            None
        }
    }

    pub fn as_custom<T: AbsentPageEntry>(&self) -> Option<&T> {
        let as_u64 = u64::from_le_bytes(self.bits.into_bytes());
        if T::is_self(as_u64) {
            // safety: not present so all other bits are ours for the taking
            Some(unsafe { self.as_custom_unchecked() })
        } else {
            None
        }
    }

    /// # Safety
    /// Present bit must be unset
    pub unsafe fn as_custom_unchecked_mut<T>(&mut self) -> &mut T {
        assert!(core::mem::size_of::<T>() <= core::mem::size_of::<Self>());
        &mut *(self as *mut _ as *mut T)
    }

    /// # Safety
    /// Present bit must be unset
    pub unsafe fn as_custom_unchecked<T>(&self) -> &T {
        assert!(core::mem::size_of::<T>() <= core::mem::size_of::<Self>());
        &*(self as *const _ as *const T)
    }

    /// Keeps existing flags
    pub fn modify(&mut self) -> EntryBuilder {
        EntryBuilder::with_entry(self)
    }

    /// Clears all bits
    pub fn replace(&mut self) -> EntryBuilder {
        EntryBuilder::with_zeroed_entry(self)
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress::from_4096_aligned(self.bits.address())
    }

    pub fn present(&self) -> bool {
        self.bits.present()
    }

    pub fn huge_pages(&self) -> bool {
        // TODO P::supports_huge_pages to hopefully optimise this out for p4/p3
        self.bits.huge()
    }

    pub fn traverse(&self) -> MemoryResult<P> {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let this = self as *const _;
        write!(f, "[{:?} -> {:?} {:?}", this, self.address(), self.bits)?;
        if f.alternate() {
            write!(f, " ({:064b})", u64::from_le_bytes(self.bits.into_bytes()))?;
        }
        write!(f, "]")
    }
}

impl Debug for PageTableBits {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "(")?;
        if self.present() {
            write!(f, "PRESENT")?;
        } else {
            write!(f, "NOT_PRESENT")?;
        }

        if self.writeable() {
            write!(f, " | READ_WRITE")?;
        } else {
            write!(f, " | READ_ONLY")?;
        }

        if self.user() {
            write!(f, " | USER")?;
        }

        if self.write_through() {
            write!(f, " | WRITE_THROUGH")?;
        }

        if self.cache_disable() {
            write!(f, " | CACHE_DISABLED")?;
        }

        if self.accessed() {
            write!(f, " | ACCESSED")?;
        }

        if self.dirty() {
            write!(f, " | DIRTY")?;
        }

        if self.huge() {
            write!(f, " | HUGE")?;
        }

        if f.alternate() {
            write!(f, " [{:064b}]", u64::from_le_bytes(self.into_bytes()))?;
        }

        write!(f, ")")?;
        Ok(())
    }
}
