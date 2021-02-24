use crate::types::DataDirectoryType;
use byte::Error;
use common::Display;
use core::char::DecodeUtf16Error;
use core::str::Utf8Error;

pub type PeResult<T> = Result<T, PeError>;

#[derive(Debug, Display)]
pub enum PeError {
    /// Failed to read bytes: {0:?}
    Reading(byte::Error),

    /// Incorrect magic value for {magic:?}, expected {expected:#x} but got {value:#x}
    IncorrectMagic {
        magic: &'static str,
        expected: u64,
        value: u64,
    },

    /// Invalid enum value for {enum_type:?} ({value:#x})
    InvalidEnumValue { enum_type: &'static str, value: u64 },

    /// Invalid bits for {bitflag_type:?}: {value:#x}
    InvalidBits {
        bitflag_type: &'static str,
        value: u64,
    },

    /// Invalid RVA count {0}
    InvalidRvaCount(u32),

    /// Invalid unicode string for {what:?}: {error}
    InvalidUnicodeString {
        what: &'static str,
        error: Utf8Error,
    },

    /// String is missing null terminator
    MissingStringNullByte,

    /// String is invalid UTF8: {0}
    InvalidString(Utf8Error),

    /// String is invalid UTF16: {0}
    InvalidStringUtf16(DecodeUtf16Error),

    /// Big endian not supported
    UnsupportedEndianness,

    /// Machine type {0:#x} not supported
    UnsupportedMachineType(u16),

    /// No optional header
    NoOptionalHeader,

    /// Unsupported image type {0:#x}
    UnsupportedImage(u16),

    /// Long section names not implemented
    LongSectionName,

    /// Data directory {0:?} not present
    MissingDataDirectory(DataDirectoryType),

    /// Value {value:#x} is not a multiple of {expected:#x}
    Unaligned { expected: u64, value: u64 },
}

impl From<byte::Error> for PeError {
    fn from(e: Error) -> Self {
        PeError::Reading(e)
    }
}
