//! Kernel stack management

use memory::{
    gigabytes, kilobytes, megabytes, MapFlags, MapTarget, MemoryError, VirtualAddress, FRAME_SIZE,
};

use crate::memory::AddressSpace;
use core::marker::PhantomData;
use enumflags2::BitFlags;

pub trait StackAllocation {
    /// Bottom of allocation
    const BASE: u64;

    /// Size of total allocation in bytes
    const SIZE: u64;

    /// Max size of a single stack in bytes
    const MAX_STACK_SIZE: u64;

    /// Amount to grow a stack at a time in bytes (slab size)
    const STACK_GROWTH_INCREMENT: u64;

    const WHAT: &'static str;

    const USER_ACCESSIBLE: bool = false;

    const MAX_SLABS: u64 = Self::MAX_STACK_SIZE / Self::STACK_GROWTH_INCREMENT;
    const MAX_STACKS: u64 = Self::SIZE / Self::MAX_STACK_SIZE;
}

/// Kernel stacks for each thread in a process
pub struct ProcessKernelStacks;

/// User stacks for each thread in a process
pub struct ProcessUserStacks;

/// Kernel stacks for interrupts for each CPU
pub struct KernelInterruptStacks;

#[derive(Copy, Clone)]
pub struct StackIndex(u64);

pub struct Stacks<A: StackAllocation> {
    next_stack: u64,
    _phantom: PhantomData<A>,
}

impl<A: StackAllocation> Default for Stacks<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: StackAllocation> Stacks<A> {
    pub fn new() -> Self {
        Self {
            next_stack: 0,
            _phantom: PhantomData,
        }
    }

    pub fn new_stack(&mut self) -> Result<(VirtualAddress, StackIndex), MemoryError> {
        let idx = StackIndex(self.next_stack);
        let stack = Self::allocate_stack(idx, 0)?;

        // increment on success
        self.next_stack += 1;

        Ok((stack, idx))
    }

    /// Maps in current address space
    ///
    /// * stack: unique stack index
    /// * slab: slab index in this stack to grow. Starts at 0 and increments for each growth
    fn allocate_stack(
        StackIndex(stack): StackIndex,
        slab: u64,
    ) -> Result<VirtualAddress, MemoryError> {
        let calc_slab_bottom = || -> Option<VirtualAddress> {
            if !Self::validate(stack, slab) {
                return None;
            };

            let stack_top = A::BASE + ((stack + 1) * A::MAX_STACK_SIZE);
            let slab_bottom = stack_top.checked_sub((slab + 1) * A::STACK_GROWTH_INCREMENT)?;
            VirtualAddress::new_checked(slab_bottom)
        };

        let slab_bottom =
            calc_slab_bottom().ok_or(MemoryError::InvalidStack(stack, slab, A::WHAT))?;

        debug_assert_eq!(
            slab_bottom.address(),
            slab_bottom.round_down_to(FRAME_SIZE).address()
        );

        let mut addr_space = AddressSpace::current();

        // ensure currently unmapped
        if addr_space.get_absent_mapping(slab_bottom).is_ok() {
            return Err(MemoryError::AlreadyMapped(slab_bottom.address()));
        }

        let slab_top = slab_bottom + A::STACK_GROWTH_INCREMENT - 8;

        let map_flags = Self::map_flags();
        if map_flags.contains(MapFlags::User) {
            // stack is growable, use guard pages

            // requested slab is the new guard page
            addr_space.map_range(
                slab_top - FRAME_SIZE,
                FRAME_SIZE,
                MapTarget::Any,
                MapFlags::StackGuard,
            )?;

        // TODO actual stack mapping done in handler?
        } else {
            // stack is not growable, commit now

            addr_space.map_range(slab_bottom, A::MAX_STACK_SIZE, MapTarget::Any, map_flags)?;

            // TODO add a guard page anyway to avoid trampling a neighbouring stack?
        }

        Ok(slab_top)
    }

    fn validate(stack: u64, slab: u64) -> bool {
        debug_assert!(A::MAX_SLABS > 0 && A::MAX_STACKS > 0);

        stack < A::MAX_STACKS && slab < A::MAX_SLABS
    }

    fn map_flags() -> BitFlags<MapFlags> {
        let mut flags = BitFlags::from(MapFlags::Writeable);

        let growable_stack = A::STACK_GROWTH_INCREMENT != A::MAX_STACK_SIZE;
        assert_eq!(A::USER_ACCESSIBLE, growable_stack);

        if growable_stack {
            flags.insert(MapFlags::User);
        } else {
            flags.insert(MapFlags::Commit);
        }

        flags
    }

    pub fn resolve_required_stack_growth(addr: VirtualAddress) -> Option<StackGrowth> {
        let base = addr.address().checked_sub(A::BASE)?;
        let stack_index = base / A::MAX_STACK_SIZE;

        if stack_index >= A::MAX_STACKS {
            return None;
        };

        let slab_index = {
            let stack_base = stack_index * A::MAX_STACK_SIZE;
            let slab_base = base.checked_sub(stack_base)?;
            A::MAX_SLABS - (slab_base / A::STACK_GROWTH_INCREMENT) - 1
        };

        if slab_index >= A::MAX_SLABS {
            // cant grow anymore
            return None;
        };

        Some(StackGrowth {
            stack: stack_index,
            slab: slab_index,
        })
    }

    pub fn grow_stack(&self, growth: StackGrowth) -> Result<(), MemoryError> {
        // trust input because it can only be constructed by resolve_required_stack_growth

        let stack_bottom = A::BASE + (growth.stack * A::MAX_STACK_SIZE);
        let slab_top = VirtualAddress(
            stack_bottom + ((A::MAX_SLABS - growth.slab) * A::STACK_GROWTH_INCREMENT),
        );
        let slab_bottom = slab_top - A::STACK_GROWTH_INCREMENT;

        let mut addr_space = AddressSpace::current();

        // ensure currently unmapped
        if addr_space.get_absent_mapping(slab_bottom).is_ok() {
            return Err(MemoryError::AlreadyMapped(slab_bottom.address()));
        }

        let flags = Self::map_flags();

        // commit this slab as usable stack space (blatting previous guard page)
        addr_space.map_range(
            slab_bottom,
            A::STACK_GROWTH_INCREMENT,
            MapTarget::Any,
            flags | MapFlags::Commit,
        )?;

        // map next slab as guard page, already checked as valid during construction of StackGrowth
        addr_space.map_range(
            slab_bottom - FRAME_SIZE,
            FRAME_SIZE,
            MapTarget::Any,
            flags | MapFlags::StackGuard,
        )?;

        Ok(())
    }
}

pub struct StackGrowth {
    /// Stack index
    stack: u64,

    /// Slab to allocate and map, is currently a guard page
    slab: u64,
}

impl StackAllocation for ProcessKernelStacks {
    const BASE: u64 = 0xffff_8001_0000_0000;
    const SIZE: u64 = gigabytes(4);
    const MAX_STACK_SIZE: u64 = kilobytes(128);
    const STACK_GROWTH_INCREMENT: u64 = Self::MAX_STACK_SIZE;
    const WHAT: &'static str = "process kernel stack";
}

impl StackAllocation for ProcessUserStacks {
    const BASE: u64 = 0x0000_1ff8_0000_0000;
    const SIZE: u64 = gigabytes(32);
    const MAX_STACK_SIZE: u64 = megabytes(1);
    const STACK_GROWTH_INCREMENT: u64 = kilobytes(64);
    const WHAT: &'static str = "user thread stack";
    const USER_ACCESSIBLE: bool = true;
}

impl StackAllocation for KernelInterruptStacks {
    const BASE: u64 = 0xffff_8000_0000_0000;
    const SIZE: u64 = megabytes(8);
    const MAX_STACK_SIZE: u64 = kilobytes(64);
    const STACK_GROWTH_INCREMENT: u64 = Self::MAX_STACK_SIZE;
    const WHAT: &'static str = "interrupt stack";
}
