#![cfg_attr(not(feature = "std"), no_std)]

#![feature(maybe_uninit_ref)]
#![feature(const_fn)]

mod error;
mod globals;
pub mod memory;

pub use error::{KernelError, KernelResult};
pub use globals::InitializedGlobal;

pub mod prelude {
    pub use bitfield::{Bit, BitRange};
    pub use c2rust_bitfields::BitfieldStruct;
    pub use derive_more::*;
    pub use enumflags2::{self, BitFlags};
}
