use num_enum::TryFromPrimitive;

/// Returned as negative i64
#[repr(u64)]
#[derive(TryFromPrimitive, Debug)]
pub enum SyscallError {
    NotImplemented = 1,
    UnknownError,
    InvalidSyscall,
    InvalidArguments,
}
