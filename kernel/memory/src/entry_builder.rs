use core::marker::PhantomData;

use crate::entry::PageTableBits;
use crate::{CommonEntry, PageTableHierarchy, PhysicalAddress, VirtualAddress, VIRT_PHYSICAL_BASE};

pub struct EntryBuilder<'e, 'p, P> {
    bits: PageTableBits,
    entry: Option<&'e mut CommonEntry<'p, P>>,
}

impl<'e, 'p, P> Default for EntryBuilder<'e, 'p, P> {
    fn default() -> Self {
        Self {
            bits: Default::default(),
            entry: None,
        }
    }
}

impl<'e, 'p, P: PageTableHierarchy<'p>> EntryBuilder<'e, 'p, P> {
    pub fn with_entry(current: &'e mut CommonEntry<'p, P>) -> Self {
        Self {
            bits: **current,
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

    pub fn address(mut self, addr: PhysicalAddress) -> Self {
        if cfg!(debug_assertions) {
            self.bits
                .set_address_checked(addr.to_4096_aligned())
                .expect("failed to set address");
        } else {
            self.bits.set_address(addr.to_4096_aligned());
        }

        self
    }

    /// If [with_entry] was used then the reference is written to as well
    pub fn build(self) -> CommonEntry<'e, P> {
        let entry = CommonEntry::new(self.bits);

        if let Some(e) = self.entry {
            *e = entry;
        }

        entry
    }
}
