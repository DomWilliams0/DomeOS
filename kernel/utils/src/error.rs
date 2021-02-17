use crate::memory::address::PhysicalAddress;
use displaydoc::Display;

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Display, Debug)]
pub enum KernelError {
    /// No physical frames available
    NoFrame,

    /// Index {1} is invalid for page table at {0:?}
    InvalidPageTableIndex(PhysicalAddress, u16),

    /// Page table of type {0} does not support table constructor
    TableNotSupported(&'static str),

    /// Page table of type {0} does not support frame constructor
    FrameNotSupported(&'static str),

    /// Page table of type {0} at {1:?} does not have a page table available
    NoTableAvailable(&'static str, PhysicalAddress),

    /// Not implemented
    NotImplemented,
}
