//! Kernel stack management

use memory::{
    MapFlags, MapTarget, MemoryError, VirtualAddress, FRAME_SIZE, KERNEL_STACKS_MAX,
    KERNEL_STACKS_START, KERNEL_STACK_MAX_SIZE, KERNEL_STACK_SIZE,
};

use crate::memory::AddressSpace;

/// [stack bottom, stack top)
fn kernel_stack_for_cpu(cpu: u64) -> (VirtualAddress, VirtualAddress) {
    assert!(cpu < KERNEL_STACKS_MAX, "cpu {} out of range", cpu);

    let stack_bottom =
        VirtualAddress::with_literal(KERNEL_STACKS_START + (cpu * KERNEL_STACK_MAX_SIZE));
    let stack_top = stack_bottom + KERNEL_STACK_MAX_SIZE - 8;

    (stack_bottom, stack_top)
}

pub fn init_kernel_stack(cpu: u64) -> Result<(), MemoryError> {
    let (_bottom, top) = kernel_stack_for_cpu(cpu);

    // TODO grow stack dynamically if needed?
    let alloc_start = top - KERNEL_STACK_SIZE;

    let mut addr_space = AddressSpace::kernel();
    let addr = allocate_kernel_stack(&mut addr_space, alloc_start, KERNEL_STACK_SIZE as usize)?;

    common::debug!("kernel stack for cpu #{} allocated at {:?}", cpu, addr);
    Ok(())
}

/// Top of stack for given CPU
pub fn kernel_stack(cpu: u64) -> VirtualAddress {
    let (_bottom, top) = kernel_stack_for_cpu(cpu);
    top
}

/// Returns stack top.
///
/// Leaves an unmapped guard page to ensure double fault on stack overflow
fn allocate_kernel_stack(
    addr_space: &mut AddressSpace,
    stack_bottom: VirtualAddress,
    frames: usize,
) -> Result<VirtualAddress, MemoryError> {
    let stack_bottom = stack_bottom.round_down_to(FRAME_SIZE);

    // +1 for guard page
    let frames = frames + 1;
    let base = addr_space.find_free_space(stack_bottom, frames)?;

    // leave guard page unmapped
    // addr_space.map_range(base, FRAME_SIZE, MapTarget::Any, MapFlags::StackGuard)?;

    // map stack just afterwards
    let stack_bottom = base + FRAME_SIZE;
    addr_space.map_range(
        stack_bottom,
        (frames as u64 - 1) * FRAME_SIZE,
        MapTarget::Any,
        MapFlags::Writeable | MapFlags::Commit,
    )?;

    // return stack top
    Ok(base + (frames as u64 * FRAME_SIZE))
}
