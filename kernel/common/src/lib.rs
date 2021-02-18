#![cfg_attr(not(feature = "std"), no_std)]
#![feature(maybe_uninit_ref)]

pub use bitfield::{Bit, BitRange};
pub use derive_more::*;
pub use displaydoc::Display;
pub use log::*;

pub use error::*;
pub use globals::{InitializedGlobal, InitializedGlobalChecked};

mod error;
mod globals;
