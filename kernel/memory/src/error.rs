use common::Display;

pub type MemoryResult<T> = Result<T, MemoryError>;

// TODO virtualaddress struct

#[derive(Display, Debug)]
pub enum MemoryError {
    /// No physical frames available
    NoFrame,

    /// No premapped physical frames available
    NoPremappedFrame,

    /// No low physical frames available
    NoLowFrame,

    /// No contiguous region of virtual memory of {0:#x} pages available from addr {1:#?}
    NoContiguousVirtualRegion(u64, u64 /* pages */),

    /// Index {1} is invalid for page table at {0:#x}
    InvalidPageTableIndex(u64, u16),

    /// Page table of type {0} does not support table constructor
    TableNotSupported(&'static str),

    /// Page table of type {0} does not support frame constructor
    FrameNotSupported(&'static str),

    /// Page table of type {0} at {1:#x} does not have a page table available
    NoTableAvailable(&'static str, u64),

    /// Physical page {0:#x} is not mapped in
    NotMapped(u64),

    /// Physical page {0:#x} is already mapped
    AlreadyMapped(u64),

    /// Invalid stack index {0}:{1} for {2}
    InvalidStack(u64, u64, &'static str),

    /// Heap cannot grow anymore
    NoMoreHeap,

    /// Mmapped IO region cannot grow anymore
    NoMoreIo,
}
