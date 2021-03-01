use core::marker::PhantomData;

use crate::entry::PageTableBits;
use crate::{CommonEntry, PageTableHierarchy, PhysicalAddress, VirtualAddress, P4};

pub struct EntryBuilder<'e> {
    bits: PageTableBits,
    /// &mut CommonEntry
    entry: Option<VirtualAddress>,

    _phantom: PhantomData<&'e ()>,
}

impl<'e> EntryBuilder<'e> {
    pub fn with_entry_and_bits<'p, P: PageTableHierarchy<'p>>(
        current: &'e mut CommonEntry<'p, P>,
        bits: PageTableBits,
    ) -> Self {
        Self {
            bits,
            entry: Some(VirtualAddress::new(current as *mut _ as u64)),
            _phantom: PhantomData,
        }
    }

    pub fn with_entry<'p, P: PageTableHierarchy<'p>>(current: &'e mut CommonEntry<'p, P>) -> Self {
        Self::with_entry_and_bits(current, **current)
    }

    pub fn with_zeroed_entry<'p, P: PageTableHierarchy<'p>>(
        current: &'e mut CommonEntry<'p, P>,
    ) -> Self {
        Self::with_entry_and_bits(current, PageTableBits::default())
    }

    pub fn writeable(mut self) -> Self {
        self.bits.set_writeable(true);
        self
    }

    pub fn read_only(mut self) -> Self {
        self.bits.set_writeable(false);
        self
    }
    pub fn user(mut self) -> Self {
        self.bits.set_user(true);
        self
    }

    pub fn supervisor(mut self) -> Self {
        self.bits.set_user(false);
        self
    }
    pub fn write_through(mut self) -> Self {
        self.bits.set_write_through(true);
        self
    }

    pub fn not_write_through(mut self) -> Self {
        self.bits.set_write_through(false);
        self
    }
    pub fn cache_disable(mut self) -> Self {
        self.bits.set_cache_disable(true);
        self
    }

    pub fn not_cache_disable(mut self) -> Self {
        self.bits.set_cache_disable(false);
        self
    }

    pub fn accessed(mut self) -> Self {
        self.bits.set_accessed(true);
        self
    }

    pub fn not_accessed(mut self) -> Self {
        self.bits.set_accessed(false);
        self
    }
    pub fn dirty(mut self) -> Self {
        self.bits.set_dirty(true);
        self
    }

    pub fn not_dirty(mut self) -> Self {
        self.bits.set_dirty(false);
        self
    }

    pub fn huge(mut self) -> Self {
        self.bits.set_huge(true);
        self
    }

    pub fn not_huge(mut self) -> Self {
        self.bits.set_huge(false);
        self
    }

    pub fn present(mut self) -> Self {
        self.bits.set_present(true);
        self
    }

    pub fn not_present(mut self) -> Self {
        self.bits.set_present(false);
        self
    }

    pub fn executable(mut self) -> Self {
        self.bits.set_nx(false);
        self
    }

    pub fn not_executable(mut self) -> Self {
        self.bits.set_nx(true);
        self
    }

    pub fn global(mut self) -> Self {
        self.bits.set_global(true);
        self
    }

    pub fn not_global(mut self) -> Self {
        self.bits.set_global(false);
        self
    }

    /// Shorthand for global, writeable, present, supervisor
    pub fn higher_half(self) -> Self {
        self.global().writeable().present().supervisor()
    }

    pub fn address(mut self, addr: PhysicalAddress) -> Self {
        if cfg!(debug_assertions) {
            self.bits
                .set_address_checked(addr.to_4096_aligned())
                .unwrap_or_else(|e| panic!("failed to set address to {:?}: {}", addr, e));
        } else {
            self.bits.set_address(addr.to_4096_aligned());
        }

        self
    }

    /// Writes to given entry reference as if it was a CommonEntry<P4>
    pub fn apply<'p>(self) -> CommonEntry<'p, P4<'p>> {
        let entry = {
            let addr = self.entry.expect("no reference provided");
            // safety: pointer was a 'e reference passed in the constructor, and so is still valid
            unsafe { &mut *addr.as_ptr() }
        };

        let new_entry = CommonEntry::<P4>::new(self.bits);
        *entry = new_entry;
        new_entry
    }
}
