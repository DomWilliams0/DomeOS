use common::*;
use memory::MemoryError;

pub type KernelResult<T> = Result<T, KernelError>;

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
