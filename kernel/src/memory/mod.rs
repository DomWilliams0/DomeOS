mod address;
pub mod free_pages;
mod mapping;
mod page_table;

pub use address::{PhysicalAddress, VirtualAddress};
pub use mapping::remap_kernel;
pub use page_table::hierarchy::walk_active_page_hierarchy;
pub use page_table::PageTable;

const fn terabytes(n: u64) -> u64 {
    n * (1 << 40)
}

const fn gigabytes(n: u64) -> u64 {
    n * (1 << 30)
}

const fn megabytes(n: u64) -> u64 {
    n * (1 << 20)
}

const fn kilobytes(n: u64) -> u64 {
    n * (1 << 10)
}
