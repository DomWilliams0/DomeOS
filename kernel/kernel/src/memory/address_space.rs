use crate::memory::page_table::pml4;
use crate::memory::phys::{frame_allocator, FrameAllocator};
use common::KernelResult;
use core::ops::{Deref, DerefMut};
use enumflags2::BitFlags;
use memory::{MemoryProvider, PhysicalFrame, RawAddressSpace};

pub struct FrameProvider;

pub struct AddressSpace<'p>(RawAddressSpace<'p, FrameProvider>);

impl MemoryProvider for FrameProvider {
    fn new_frame(&mut self) -> KernelResult<PhysicalFrame> {
        frame_allocator().allocate(BitFlags::empty())
    }
}

impl<'p> AddressSpace<'p> {
    pub fn current() -> Self {
        // safety: valid pml4 is in cr3
        let address_space = unsafe { RawAddressSpace::with_existing(pml4(), FrameProvider) };
        Self(address_space)
    }
}

impl<'p> Deref for AddressSpace<'p> {
    type Target = RawAddressSpace<'p, FrameProvider>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'p> DerefMut for AddressSpace<'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
