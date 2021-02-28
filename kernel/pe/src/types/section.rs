use crate::address::{Address, FileOffset, VirtualAddress};
use crate::cursor::Reinterpret;
use crate::error::{PeError, PeResult};
use bitflags::bitflags;
use core::fmt::{Display, Formatter};

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
pub struct SectionDescriptor {
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

#[derive(Debug)]
pub struct MappableSection<'pe> {
    pub name: &'pe str,
    pub virtual_size: usize,
    pub virtual_address: VirtualAddress,

    /// (size of raw data, pointer to raw data). None if uninitialized
    pub raw_data: Option<(usize, FileOffset)>,

    pub flags: SectionFlags,
}

impl SectionDescriptor {
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

    pub fn as_mappable(&self) -> PeResult<Option<MappableSection>> {
        let virtual_size = match self.virtual_size {
            0 => return Ok(None),
            val => val as usize,
        };

        let virtual_address = match self.virtual_address {
            0 => return Ok(None),
            val => VirtualAddress::new(val),
        };

        let raw_data = match (self.size_of_raw_data, self.pointer_to_raw_data) {
            (0, _) | (_, 0) => None,
            (sz, ptr) => Some((sz as usize, FileOffset::new(ptr))),
        };

        Ok(Some(MappableSection {
            name: self.name()?,
            virtual_size,
            virtual_address,
            raw_data,
            flags: self.flags()?,
        }))
    }
}

// safety: raw PE type
unsafe impl Reinterpret for SectionDescriptor {}

impl Display for SectionDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Section({:?}, {:?}:{:?})",
            self.name(),
            self.flags(),
            self
        )
    }
}
