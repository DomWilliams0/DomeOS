use std::num::NonZeroU16;

use enumflags2::BitFlags;

use crate::cursor::Reinterpret;
use crate::error::{PeError, PeResult};

#[derive(BitFlags, Copy, Clone, Debug)]
#[repr(u16)]
pub enum Characteristics {
    /// Image only, Windows CE, and Microsoft Windows NT and later. This indicates that the
    /// file does not contain base relocations and must therefore be loaded at its preferred base
    /// address. If the base address is not available, the loader reports an error. The default behavior
    /// of the linker is to strip base relocations from executable (EXE) files.
    RelocsStripped = 0x0001,

    /// Image only. This indicates that the image file is valid and can be run. If this flag
    /// is not set, it indicates a linker error.
    ExecutableImage = 0x0002,

    /// COFF line numbers have been removed. This flag is deprecated and should be zero.
    LineNumsStripped = 0x0004,

    /// COFF symbol table entries for local symbols have been removed. This flag is deprecated
    /// and should be zero.
    LocalSymsStripped = 0x0008,

    /// Obsolete. Aggressively trim working set. This flag is deprecated for Windows 2000 and
    /// later and must be zero.
    AggressiveWsTrim = 0x0010,

    /// Application can handle > 2-GB addresses.
    LargeAddressAware = 0x0020,

    /// This flag is reserved for future use.
    _Reserved = 0x0040,

    /// Little endian: the least significant bit (LSB) precedes the most significant bit (MSB)
    /// in memory. This flag is deprecated and should be zero.
    BytesReversedLo = 0x0080,

    /// Machine is based on a 32-bit-word architecture.
    ThirtyTwoBitMachine = 0x0100,

    /// Debugging information is removed from the image file.
    DebugStripped = 0x0200,

    /// If the image is on removable media, fully load it and copy it to the swap file.
    RemovableRunFromSwap = 0x0400,

    /// If the image is on network media, fully load it and copy it to the swap file.
    NetRunFromSwap = 0x0800,

    /// The image file is a system file, not a user program.
    System = 0x1000,

    /// The image file is a dynamic-link library (DLL). Such files are considered executable
    /// files for almost all purposes, although they cannot be directly run.
    Dll = 0x2000,

    /// The file should be run only on a uniprocessor machine.
    UpSystemOnly = 0x4000,

    /// Big endian: the MSB precedes the LSB in memory. This flag is deprecated and should be
    /// zero.
    BytesReversedHi = 0x8000,
}

#[derive(Debug)]
#[repr(C)]
pub struct CoffHeader {
    machine: u16,
    pub num_sections: u16,
    timestamp: u32,
    symbol_table_offset: u32,
    symbol_count: u32,
    optional_header_sz: u16,
    characteristics: u16,
}

impl CoffHeader {
    pub fn machine_type(&self) -> Result<(), u16> {
        match self.machine {
            0 | 0x8664 => Ok(()),
            x => Err(x),
        }
    }

    pub fn characteristics(&self) -> PeResult<BitFlags<Characteristics>> {
        BitFlags::from_bits(self.characteristics).map_err(|_| PeError::InvalidBits {
            bitflag_type: "characteristics",
            value: self.characteristics as u64,
        })
    }

    pub fn optional_header_size(&self) -> Option<NonZeroU16> {
        if self.optional_header_sz != 0 {
            Some(unsafe { NonZeroU16::new_unchecked(self.optional_header_sz) })
        } else {
            None
        }
    }
}

// safety: raw PE type
unsafe impl Reinterpret for CoffHeader {}
