#![feature(maybe_uninit_ref)]
#![cfg_attr(not(feature = "std"), no_std)]

mod globals;
pub mod memory;

pub use globals::InitializedGlobal;

pub mod prelude {
    pub use bitfield::{Bit, BitRange};
    pub use c2rust_bitfields::BitfieldStruct;
    pub use derive_more::*;
    pub use enumflags2::{self, BitFlags};
}
