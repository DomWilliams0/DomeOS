use crate::memory::address::VirtualAddress;

pub const VIRT_USERSPACE_MAX: u64 = 0x2000_0000_0000;

pub const VIRT_PHYSICAL_BASE: u64 = 0xffff_9000_0000_0000;
pub const VIRT_PHYSICAL_MAX: u64 = 0xffff_d000_0000_0000;
pub const VIRT_PHYSICAL_SIZE: u64 = VIRT_PHYSICAL_MAX - VIRT_PHYSICAL_BASE;

pub const VIRT_KERNEL_BASE: u64 = 0xffff_ffff_8000_0000;
pub const VIRT_KERNEL_MAX: u64 = 0xffff_ffff_c000_0000;
pub const VIRT_KERNEL_SIZE: u64 = VIRT_KERNEL_MAX - VIRT_KERNEL_BASE;

pub const PHYS_KERNEL_BASE: u64 = 0x10_0000;

pub const FRAME_SIZE: u64 = 4096;

#[derive(Copy, Clone)]
pub enum VirtualMapping {
    PhysicalIdentity,
    KernelIdentity,
}

#[deprecated]
pub fn mapping_for_address(addr: VirtualAddress) -> Option<VirtualMapping> {
    let ranges = [
        (
            VirtualMapping::KernelIdentity,
            VIRT_KERNEL_BASE,
            VIRT_KERNEL_MAX,
        ),
        (
            VirtualMapping::PhysicalIdentity,
            VIRT_PHYSICAL_BASE,
            VIRT_PHYSICAL_MAX,
        ),
    ];

    ranges.iter().find_map(|(ty, start, end)| {
        if ((*start)..(*end)).contains(&addr.0) {
            Some(*ty)
        } else {
            None
        }
    })
}
