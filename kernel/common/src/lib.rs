#![cfg_attr(not(feature = "std"), no_std)]
#![feature(maybe_uninit_ref)]

pub use anyhow::{self, Context, Error, Result};
pub use bitfield::{Bit, BitRange};
pub use derive_more::*;
pub use displaydoc::Display;
pub use log::{self, *};

pub use globals::{InitializedGlobal, InitializedGlobalChecked};

mod globals;
