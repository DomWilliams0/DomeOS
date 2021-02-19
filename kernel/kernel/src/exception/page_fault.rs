use common::*;
use core::fmt::{Debug, Error, Formatter};

use crate::memory::{frame_allocator, AddressSpace, FrameAllocator};
use enumflags2::BitFlags;
use memory::{AbsentPageEntry, CustomPageEntry, DemandMapping, VirtualAddress};

#[derive(Debug)]
pub struct PageFaultException {
    pub flags: PageFaultFlags,
    pub addr: VirtualAddress,
}

#[derive(BitFlags, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PageFaultFlag {
    Present = 0b00001,
    Write = 0b00010,
    User = 0b00100,
    ReservedWrite = 0b01000,
    InstrFetch = 0b10000,
}

#[derive(Deref)]
pub struct PageFaultFlags(BitFlags<PageFaultFlag>);

impl PageFaultException {
    pub fn new(flags: BitFlags<PageFaultFlag>, cr2: VirtualAddress) -> Self {
        PageFaultException {
            flags: PageFaultFlags(flags),
            addr: cr2,
        }
    }

    pub fn handle(self) {
        // TODO get from current process block instead
        // TODO on error, either kill process or kernel panic

        let mut addr_space = AddressSpace::current();

        if self.flags.contains(PageFaultFlag::Present) {
            panic!("page fault on present page: {:?}", self);
        }

        // fetch mapping
        let (level, mapping) = addr_space
            .get_absent_mapping::<CustomPageEntry>(self.addr)
            .expect("nonsensical page fault");

        match mapping.on_demand() {
            DemandMapping::Anonymous => {
                // TODO reuse same physical page and CoW
                // TODO what do if frame allocation fails?
                let frame = frame_allocator()
                    .allocate(BitFlags::empty())
                    .expect("failed to allocate frame");

                // rewrite mapping
                mapping
                    .as_builder()
                    .address(frame.address())
                    .present()
                    .apply();
            }
        };
    }
}

impl Debug for PageFaultFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "(")?;
        if self.0.contains(PageFaultFlag::Present) {
            write!(f, "PAGE_PROTECTION_VIOLATION")?;
        } else {
            write!(f, "NON_PRESENT_PAGE")?;
        }

        if self.0.contains(PageFaultFlag::Write) {
            write!(f, " | CAUSED_BY_WRITE")?;
        } else {
            write!(f, " | CAUSED_BY_READ")?;
        }

        if self.0.contains(PageFaultFlag::User) {
            write!(f, " | USER")?;
        }

        if self.0.contains(PageFaultFlag::ReservedWrite) {
            write!(f, " | RESERVED_WRITE")?;
        }

        if self.0.contains(PageFaultFlag::InstrFetch) {
            write!(f, " | INSTR_FETCH")?;
        }

        write!(f, ")")?;
        Ok(())
    }
}
