#![cfg_attr(not(feature = "std"), no_std)]

mod map;
mod output;

pub use map::parse_text;
pub use output::{Sink, StdoutSink, PackedFormatSink};