use displaydoc::Display;

pub type KernelResult<T> = Result<T, KernelError>;

// it's not ideal having all error types in this crate, but until we have allocation to box a dyn
// trait they need to be here to be accessible

pub use memory::{MemoryError, MemoryResult};

mod memory;

#[derive(Display, Debug)]
pub enum KernelError {
    /// Memory error: {0}
    Memory(MemoryError),

    /// Not implemented
    NotImplemented,
}

impl From<MemoryError> for KernelError {
    fn from(err: MemoryError) -> Self {
        Self::Memory(err)
    }
}
