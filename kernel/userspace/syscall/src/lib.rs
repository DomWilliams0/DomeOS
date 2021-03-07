#![cfg_attr(not(test), no_std)]
#![feature(const_fn_transmute)]

mod error;
mod result;

pub use error::SyscallError;
pub use result::SyscallResult;
