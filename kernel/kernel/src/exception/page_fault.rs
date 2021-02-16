use core::fmt::{Debug, Error, Formatter};

use log::*;
use utils::memory::address::VirtualAddress;
use utils::prelude::BitFlags;

#[derive(Debug)]
pub struct PageFaultException(pub PageFaultFlags, pub VirtualAddress);

#[derive(BitFlags, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PageFaultFlag {
    Present = 0b00001,
    Write = 0b00010,
    User = 0b00100,
    ReservedWrite = 0b01000,
    InstrFetch = 0b10000,
}

pub struct PageFaultFlags(BitFlags<PageFaultFlag>);

impl PageFaultException {
    pub fn new(flags: BitFlags<PageFaultFlag>, cr2: VirtualAddress) -> Self {
        PageFaultException(PageFaultFlags(flags), cr2)
    }

    // TODO pub fn handle(self) { }
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
