use crate::error::{PeError, PeResult};
use bitflags::bitflags;
use core::num::NonZeroU16;
use enumflags2::BitFlags;

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

bitflags! {
    pub struct SectionFlags : u32 {
        #[doc = "The section should not be padded to the next boundary. This flag is obsolete and \
        is replaced by ALIGN_1BYTES. This is valid only for object files."]
        const TYPE_NO_PAD = 0x0000_0008;

        #[doc = "The section contains executable code."]
        const CNT_CODE = 0x0000_0020;

        #[doc = "The section contains initialized data."]
        const CNT_INITIALIZED_DATA = 0x0000_0040;

        #[doc = "The section contains uninitialized data."]
        const CNT_UNINITIALIZED_DATA = 0x0000_0080;

        #[doc = "Reserved for future use."]
        const LNK_OTHER = 0x0000_0100;

        #[doc = "The section contains comments or other information. The .drectve section has this \
        type. This is valid for object files only."]
        const LNK_INFO = 0x0000_0200;

        #[doc = "The section will not become part of the image. This is valid only for object \
        files."]
        const LNK_REMOVE = 0x0000_0800;

        #[doc = "The section contains COMDAT data. For more information, see COMDAT Sections \
        (Object Only). This is valid only for object files."]
        const LNK_COMDAT = 0x0000_1000;

        #[doc = "The section contains data referenced through the global pointer (GP)."]
        const GPREL = 0x0000_8000;

        /*
        #[doc = "Reserved for future use."]
        const MEM_PURGEABLE = 0x00020000;

        #[doc = "Reserved for future use."]
        const MEM_16BIT = 0x00020000;

        #[doc = "Reserved for future use."]
        const MEM_LOCKED = 0x00040000;

        #[doc = "Reserved for future use."]
        const MEM_PRELOAD = 0x00080000;
        */

        #[doc = "Align data on a 1-byte boundary. Valid only for object files."]
        const ALIGN_1BYTES = 0x0010_0000;

        #[doc = "Align data on a 2-byte boundary. Valid only for object files."]
        const ALIGN_2BYTES = 0x0020_0000;

        #[doc = "Align data on a 4-byte boundary. Valid only for object files."]
        const ALIGN_4BYTES = 0x0030_0000;

        #[doc = "Align data on an 8-byte boundary. Valid only for object files."]
        const ALIGN_8BYTES = 0x0040_0000;

        #[doc = "Align data on a 16-byte boundary. Valid only for object files."]
        const ALIGN_16BYTES = 0x0050_0000;

        #[doc = "Align data on a 32-byte boundary. Valid only for object files."]
        const ALIGN_32BYTES = 0x0060_0000;

        #[doc = "Align data on a 64-byte boundary. Valid only for object files."]
        const ALIGN_64BYTES = 0x0070_0000;

        #[doc = "Align data on a 128-byte boundary. Valid only for object files."]
        const ALIGN_128BYTES = 0x0080_0000;

        #[doc = "Align data on a 256-byte boundary. Valid only for object files."]
        const ALIGN_256BYTES = 0x0090_0000;

        #[doc = "Align data on a 512-byte boundary. Valid only for object files."]
        const ALIGN_512BYTES = 0x00A0_0000;

        #[doc = "Align data on a 1024-byte boundary. Valid only for object files."]
        const ALIGN_1024BYTES = 0x00B0_0000;

        #[doc = "Align data on a 2048-byte boundary. Valid only for object files."]
        const ALIGN_2048BYTES = 0x00C0_0000;

        #[doc = "Align data on a 4096-byte boundary. Valid only for object files."]
        const ALIGN_4096BYTES = 0x00D0_0000;

        #[doc = "Align data on an 8192-byte boundary. Valid only for object files."]
        const ALIGN_8192BYTES = 0x00E0_0000;

        #[doc = "The section contains extended relocations."]
        const LNK_NRELOC_OVFL = 0x0100_0000;

        #[doc = "The section can be discarded as needed."]
        const MEM_DISCARDABLE = 0x0200_0000;

        #[doc = "The section cannot be cached."]
        const MEM_NOT_CACHED = 0x0400_0000;

        #[doc = "The section is not pageable."]
        const MEM_NOT_PAGED = 0x0800_0000;

        #[doc = "The section can be shared in memory."]
        const MEM_SHARED = 0x1000_0000;

        #[doc = "The section can be executed as code."]
        const MEM_EXECUTE = 0x2000_0000;

        #[doc = "The section can be read."]
        const MEM_READ = 0x4000_0000;

        #[doc = "The section can be written to."]
        const MEM_WRITE = 0x8000_0000;
    }
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

/// PE32+ only
#[derive(Debug)]
#[repr(C)]
pub struct OptionalHeader {
    magic: u16,

    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entry_point: u32,
    base_of_code: u32,

    // missing base_of_data on PE32+
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_os_version: u16,
    minor_os_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    _win32_version_value: u32,

    size_of_image: u32,
    size_of_headers: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,

    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,

    _loader_flags: u32,
    number_of_data_directories: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader {
    name: [u8; 8],
    virtual_size: u32,
    virtual_address: u32,
    size_of_raw_data: u32,
    pointer_to_raw_data: u32,
    pointer_to_relocations: u32,
    pointer_to_line_numbers: u32,
    number_of_relocations: u16,
    number_of_line_numbers: u16,
    characteristics: u32,
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

impl OptionalHeader {
    pub fn image_type(&self) -> Result<(), u16> {
        if self.magic == 0x20B {
            Ok(())
        } else {
            Err(self.magic)
        }
    }
}

impl SectionHeader {
    pub fn name(&self) -> PeResult<&str> {
        // strip null padding if present
        let string_buf = match self.name.iter().position(|&b| b == 0) {
            Some(index) => &self.name[..index],
            None => &self.name,
        };

        if string_buf.contains(&b'/') {
            return Err(PeError::LongSectionName);
        }

        core::str::from_utf8(string_buf).map_err(|error| PeError::InvalidUnicodeString {
            what: "section name",
            error,
        })
    }

    pub fn flags(&self) -> PeResult<SectionFlags> {
        SectionFlags::from_bits(self.characteristics).ok_or(PeError::InvalidBits {
            bitflag_type: "section flags",
            value: self.characteristics as u64,
        })
    }
}
