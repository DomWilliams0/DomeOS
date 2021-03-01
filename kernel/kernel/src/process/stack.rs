use crate::memory::AddressSpace;
use memory::{MapFlags, MapTarget, MemoryError, VirtualAddress, FRAME_SIZE};

/// Returns stack top.
///
/// Leaves an unmapped guard page to ensure double fault on stack overflow
pub fn allocate_kernel_stack(
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
