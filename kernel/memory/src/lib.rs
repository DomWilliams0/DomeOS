#![cfg_attr(not(feature = "std"), no_std)]
#![feature(core_intrinsics)]

pub use address::{round_down_to, round_up_to, PhysicalAddress, VirtualAddress};
pub use address_space::{
    iter_all_pages, MapFlags, MapTarget, MappedSlice, MemoryProvider, Pml4Guard, RawAddressSpace,
};
pub use constants::*;
pub use custom_entry::{CustomPageEntry, DemandMapping};
pub use entry::{CommonEntry, PageTableBits, PageTableFlag};
pub use entry_builder::EntryBuilder;
pub use error::MemoryError;
pub use frame::PhysicalFrame;
pub use hierarchy::*;
pub use page_table::{EntryIndex, PageTable, PAGE_TABLE_ENTRY_COUNT};

mod address;
mod address_space;
mod constants;
mod custom_entry;
mod entry;
mod entry_builder;
mod error;
mod frame;
mod hierarchy;
mod page_table;

pub const fn terabytes(n: u64) -> u64 {
    n * (1 << 40)
}

pub const fn gigabytes(n: u64) -> u64 {
    n * (1 << 30)
}

pub const fn megabytes(n: u64) -> u64 {
    n * (1 << 20)
}

pub const fn kilobytes(n: u64) -> u64 {
    n * (1 << 10)
}
