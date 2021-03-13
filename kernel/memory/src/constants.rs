use crate::{gigabytes, terabytes};
use paste::paste;

macro_rules! declare_region {
    ($base:expr => $limit:expr; $len:expr, $name:ident) => {
        paste! {
            pub const [<$name _BASE>]: u64 = $base;
            pub const [<$name _SIZE>]: u64 = $len;
            pub const [<$name _MAX>]: u64 = $limit;

            pub const [<$name _RANGE>]: core::ops::Range<u64> = $base..$limit;

            static_assertions::const_assert_eq!([<$name _MAX>], $base + $len);
        }
    };
}

// corresponds to kernel/src/memory/memory/README.md

declare_region!(0x0000_0000_0000_0000 => 0x0000_2000_0000_0000; terabytes(32), VIRT_USERSPACE);
declare_region!(0xffff_8000_0000_0000 => 0xffff_8001_0000_0000; gigabytes(4), KERNEL_ISR_STACKS);
declare_region!(0xffff_8001_0000_0000 => 0xffff_8002_0000_0000; gigabytes(4), PROCESS_KERNEL_STACKS);
declare_region!(0xffff_9000_0000_0000 => 0xffff_d000_0000_0000; terabytes(64), VIRT_PHYSICAL);
declare_region!(0xffff_d000_0000_0000 => 0xffff_d004_0000_0000; gigabytes(16), VIRT_KERNEL_HEAP);
declare_region!(0xffff_ffff_8000_0000 => 0xffff_ffff_c000_0000; gigabytes(1), VIRT_KERNEL);

pub const PHYS_KERNEL_BASE: u64 = 0x10_0000;
pub const FRAME_SIZE: u64 = 4096;

pub const ACCESSIBLE_RANGES: [core::ops::Range<u64>; 5] = [
    PROCESS_KERNEL_STACKS_RANGE,
    KERNEL_ISR_STACKS_RANGE,
    VIRT_PHYSICAL_RANGE,
    VIRT_KERNEL_HEAP_RANGE,
    VIRT_KERNEL_RANGE,
];
