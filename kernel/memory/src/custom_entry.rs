use crate::{CommonEntry, EntryBuilder, PageTableBits, P4};
use modular_bitfield::prelude::*;

const MARKER: u32 = 0xcc_cc_cc;

#[derive(BitfieldSpecifier, Debug, Copy, Clone)]
#[bits = 1]
pub enum DemandMapping {
    Anonymous,
    // TODO CoW
    // TODO Mapped file(fd in process)
}

/// A page table entry where the present bit is not set, so all other bits are available
#[bitfield]
#[derive(Copy, Clone, Debug)]
pub struct CustomPageEntry {
    /// Overlaps with present bit, must be 0
    #[skip(setters)]
    _reserved: B1,

    // matches up with real flags
    pub writeable: bool,
    pub user: bool,
    pub write_through: bool,
    pub cache_disable: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub huge: bool,
    pub global: bool,

    // begin custom flags
    /// Mapped but lazily allocated on access
    pub on_demand: DemandMapping,

    // end custom flags
    /// Set to a specific bit pattern to differentiate garbage pages
    marker: B24,

    #[skip]
    _unused: B29,

    // matches up with real nx bit
    pub nx: bool,
}

impl Default for CustomPageEntry {
    /// All bits unset except for nx=1 and marker bits
    fn default() -> Self {
        Self::new().with_nx(true).with_marker(MARKER)
    }
}

impl CustomPageEntry {
    pub fn is_self(entry: u64) -> bool {
        let possibly_entry: Self = unsafe { core::mem::transmute(entry) };
        possibly_entry._reserved() == 0 && possibly_entry.marker() == MARKER
    }

    pub fn as_builder(&mut self) -> EntryBuilder {
        // transfer bits without extra custom ones
        let bits = PageTableBits::new()
            .with_writeable(self.writeable())
            .with_user(self.user())
            .with_write_through(self.write_through())
            .with_cache_disable(self.cache_disable())
            .with_accessed(self.accessed())
            .with_dirty(self.dirty())
            .with_huge(self.huge())
            .with_global(self.global())
            .with_nx(self.nx());

        // safety: this maps to a common entry but when present=false. treat it as a P4 entry for
        // no reason beside it needs a type and all tables use the same entries
        let entry = unsafe { &mut *(self as *mut _ as *mut CommonEntry<P4>) };
        EntryBuilder::with_entry_and_bits(entry, bits)
    }
}
