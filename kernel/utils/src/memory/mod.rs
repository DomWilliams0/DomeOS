pub use constants::*;
pub use frame::PhysicalFrame;
pub use hierarchy::*;

pub mod address;
mod frame;
pub mod page_table;

mod constants;
mod hierarchy;

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
