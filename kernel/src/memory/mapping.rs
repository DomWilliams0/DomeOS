use log::*;

use kernel_utils::memory::{kilobytes, terabytes};
use kernel_utils::memory::address::VirtualAddress;
use kernel_utils::memory::page_table::hierarchy::PageTableHierarchy;

use crate::memory::page_table::pml4;

/// Kernel virtual addresses: 128TiB -> 192TiB
/// Physical mapping:         192TiB -> 256TiB

/// Start of kernel code mapping at 128TiB
const KERNEL_START_ADDR: u64 = terabytes(128);

const KERNEL_MAPPING_LENGTH: u64 = terabytes(64);

/// Start of direct physical mapping at 128TiB
const PHYSICAL_MAPPING_OFFSET: u64 = terabytes(192);

/// Size of direct physical mapping
const PHYSICAL_MAPPING_LENGTH: u64 = terabytes(64);

extern "C" {
    static KERNEL_START: u64;
    static KERNEL_END: u64;
}
fn kernel_range() -> (u64, u64) {
    unsafe {
        (
            &KERNEL_START as *const _ as u64,
            &KERNEL_END as *const _ as u64,
        )
    }
}

pub fn remap_kernel() {
    let (old_start, old_end) = kernel_range();
    debug!(
        "kernel code ranges from {:#x?} to {:#x?}",
        old_start, old_end
    );

    let n_pages = {
        let size = old_end - old_start;
        let page_size = kilobytes(4); // TODO huge 2MiB
        let page_count = size / page_size;
        page_count + 1
    };

    let new_start = VirtualAddress::new(KERNEL_START_ADDR + old_start);
    let new_end = VirtualAddress::new(KERNEL_START_ADDR + old_end);

    debug!("mapping {} pages starting at {:?}", n_pages, new_start);

    let pml4 = pml4();
    let (p3, p3_entry) = pml4.traverse(new_start).unwrap_next_level();

    debug!("p3: {:?}, entry: {:?}", p3, p3_entry);
    let (p2, p2_entry) = p3.traverse(new_start).unwrap_next_level();

    debug!("p2: {:?}, entry: {:?}", p2, p2_entry);

    // remap to new vaddr
    // jump
    // remove old mapping
}
