use crate::memory::VirtualAddress;

const fn terabytes(n: u64) -> u64 {
    n * (1 << 40)
}

/// Start of direct physical mapping
const PHYSICAL_MAPPING_OFFSET: VirtualAddress = VirtualAddress(0xffffff00_00000000);

/// Size of direct physical mapping
const PHYSICAL_MAPPING_LENGTH: u64 = terabytes(64);

/// Start of kernel code mapping
const KERNEL_START_ADDR: VirtualAddress = VirtualAddress(0xffff0000_00000000);

pub fn remap_kernel() {
    // remap to new vaddr
    // jump
    // remove old mapping
}
