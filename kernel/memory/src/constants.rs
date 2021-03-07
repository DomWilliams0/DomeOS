use crate::{kilobytes, megabytes};

pub const VIRT_USERSPACE_MAX: u64 = 0x2000_0000_0000;

pub const VIRT_PHYSICAL_BASE: u64 = 0xffff_9000_0000_0000;
pub const VIRT_PHYSICAL_MAX: u64 = 0xffff_d000_0000_0000;
pub const VIRT_PHYSICAL_SIZE: u64 = VIRT_PHYSICAL_MAX - VIRT_PHYSICAL_BASE;

pub const VIRT_KERNEL_BASE: u64 = 0xffff_ffff_8000_0000;
pub const VIRT_KERNEL_MAX: u64 = 0xffff_ffff_c000_0000;
pub const VIRT_KERNEL_SIZE: u64 = VIRT_KERNEL_MAX - VIRT_KERNEL_BASE;

pub const VIRT_KERNEL_HEAP_BASE: u64 = 0xffff_d000_0000_0000;

pub const PHYS_KERNEL_BASE: u64 = 0x10_0000;

pub const FRAME_SIZE: u64 = 4096;

pub const KERNEL_STACKS_START: u64 = 0xffff_8000_0000_0000;
pub const KERNEL_STACK_MAX_SIZE: u64 = megabytes(8);
pub const KERNEL_STACK_SIZE: u64 = kilobytes(64) / FRAME_SIZE;
pub const KERNEL_STACKS_MAX: u64 = 512;

pub const ACCESSIBLE_RANGES: [core::ops::Range<u64>; 3] = [
    (KERNEL_STACKS_START..KERNEL_STACKS_START + (KERNEL_STACKS_MAX * KERNEL_STACK_MAX_SIZE)),
    (VIRT_PHYSICAL_BASE..VIRT_PHYSICAL_MAX),
    (VIRT_KERNEL_BASE..VIRT_KERNEL_MAX),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gigabytes, terabytes};

    #[test]
    fn sizes_sanity_check() {
        assert_eq!(VIRT_PHYSICAL_SIZE, terabytes(64));
        assert_eq!(VIRT_KERNEL_SIZE, gigabytes(1));
    }
}
