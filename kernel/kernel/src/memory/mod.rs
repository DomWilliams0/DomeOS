mod address_space;
mod heap;
mod init;
mod phys;
mod stack;

pub use address_space::{AddressSpace, AddressSpaceRef};
pub use init::init;
use memory::megabytes;
pub use phys::{frame_allocator, FrameAllocator, FrameFlags};
pub use stack::{
    KernelInterruptStacks, ProcessKernelStacks, ProcessUserStacks, StackGrowth, Stacks,
};

/// Matches boot/long_mode.asm
const KERNEL_IDENTITY_MAPPING: u64 = megabytes(32);
