use displaydoc::Display;

pub type MemoryResult<T> = Result<T, MemoryError>;

#[derive(Display, Debug)]
pub enum MemoryError {
    /// No physical frames available
    NoFrame,

    /// Index {1} is invalid for page table at {0:#x}
    InvalidPageTableIndex(u64, u16),

    /// Page table of type {0} does not support table constructor
    TableNotSupported(&'static str),

    /// Page table of type {0} does not support frame constructor
    FrameNotSupported(&'static str),

    /// Page table of type {0} at {1:#x} does not have a page table available
    NoTableAvailable(&'static str, u64),
}
