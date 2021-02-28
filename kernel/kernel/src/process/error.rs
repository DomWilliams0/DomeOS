use common::Display;
use pe::PeError;

#[derive(Debug, Display)]
pub enum ProcessError {
    /// Error parsing PE: {0}
    Pe(pe::PeError),

    /// Address {0:#x} cannot be represented as a 48-bit virtual address
    BadVirtualAddress(u64),

    /// Relocation required but not yet supported
    RelocationUnsupported,

    /// Length mismatch, cannot copy {src:#x} bytes into slice of length {dst:#x}
    LengthMismatch { src: usize, dst: usize },

    /// Image has no entry point
    NoEntrypoint,
}

impl From<pe::PeError> for ProcessError {
    fn from(err: PeError) -> Self {
        Self::Pe(err)
    }
}
