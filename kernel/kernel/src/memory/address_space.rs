use crate::memory::phys::{frame_allocator, FrameAllocator};

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use enumflags2::BitFlags;
pub use memory::{
    MemoryError, MemoryProvider, PageTableHierarchy, PhysicalFrame, RawAddressSpace, P1, P2, P4,
};

#[derive(Clone)]
pub struct FrameProvider;

pub struct AddressSpace<'p>(RawAddressSpace<'p, FrameProvider>);

/// * 'p: physical PML4 page
/// * 'r: this reference
#[repr(transparent)]
pub struct AddressSpaceRef<'p: 'r, 'r>(AddressSpace<'p>, PhantomData<&'r ()>);

impl MemoryProvider for FrameProvider {
    fn new_frame(&mut self) -> Result<PhysicalFrame, MemoryError> {
        frame_allocator().allocate(BitFlags::empty())
    }
}

impl<'p> AddressSpace<'p> {
    pub fn current() -> Self {
        // safety: valid pml4 must be in cr3
        let address_space = unsafe { RawAddressSpace::with_existing(cr3::get(), FrameProvider) };
        Self(address_space)
    }

    pub fn kernel() -> AddressSpace<'static> {
        extern "C" {
            #[link_name = "init_pml4"]
            static KERNEL_P4: usize;
        }

        unsafe {
            AddressSpace({
                let p4 = (&KERNEL_P4) as *const _ as u64 as *mut () as *mut _;
                RawAddressSpace::with_existing(P4::with_initialized(&mut *p4), FrameProvider)
            })
        }
    }

    pub fn is_current(&self) -> bool {
        let current = cr3::get();
        self.0 == current
    }

    /// New totally empty address space
    fn new_empty() -> Result<Self, MemoryError> {
        FrameProvider.new_frame().map(|frame| unsafe {
            let p4 = P4::new(frame);
            Self(RawAddressSpace::with_existing(p4, FrameProvider))
        })
    }

    /// New empty address space with kernel mapped in higher half
    pub fn new() -> Result<Self, MemoryError> {
        let mut addr_space = Self::new_empty()?;
        let mut new_p4 = addr_space.pml4_mut();
        new_p4.ensure_accessible();

        // copying from this p4 only
        let mut kernel_space = Self::kernel();
        let mut kernel_p4 = kernel_space.pml4_mut();
        let kernel_p4 = kernel_p4.table_mut();

        for idx in 256..512 {
            let src = kernel_p4.entry_physical_mut(idx);
            // can use entry_physical_mut because of ensure_accessible() above
            let dst = new_p4.entry_physical_mut(idx);

            // safety: page table entries in 'static kernel p4 outlive this function, so erase
            // lifetimes by converting through a u64
            unsafe {
                let src: u64 = core::mem::transmute(*src);
                let dst: &mut u64 = &mut *(dst as *mut _ as *mut u64);
                *dst = src;
            }
        }

        Ok(addr_space)
    }

    pub fn borrow<'space>(&self) -> AddressSpaceRef<'p, 'space> {
        // safety: lifetime is still restricted
        let addr_space = unsafe { self.0.clone() };
        AddressSpaceRef(AddressSpace(addr_space), PhantomData)
    }

    pub unsafe fn load_unconditionally(&mut self) {
        let pml4 = self.pml4_mut();
        cr3::set(&*pml4)
    }

    pub unsafe fn load_if_not_current(&mut self) {
        if !self.is_current() {
            self.load_unconditionally();
        }
    }

    pub fn log_hierarchy(&self) {
        let p4 = self.pml4();
        for (i, e) in p4.present_entries() {
            common::info!("pml4 {}: {:?}", i, e);

            let p3 = e.traverse().unwrap();
            for (i, e) in p3.present_entries() {
                common::info!(" pdp {}: {:?}", i, e);

                if let P2::PDT(p2) = e.traverse().unwrap() {
                    for (i, e) in p2.present_entries() {
                        common::info!("  pde {}: {:?}", i, e);

                        if let P1::PT(p1) = e.traverse().unwrap() {
                            for (i, e) in p1.present_entries() {
                                common::info!("   pte {}: {:?}", i, e);
                            }
                        }
                    }
                }
            }
        }
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
impl<'p, 'r> Deref for AddressSpaceRef<'p, 'r> {
    type Target = AddressSpace<'p>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'p, 'r> DerefMut for AddressSpaceRef<'p, 'r> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

mod cr3 {
    use common::BitRange;
    use memory::{PhysicalAddress, P4};

    fn cr3() -> u64 {
        let value: u64;
        unsafe {
            asm!("mov {0}, cr3", out(reg) value);
        }
        value
    }

    pub fn get<'p>() -> P4<'p> {
        let addr = cr3().bit_range(51, 12);
        let ptr = PhysicalAddress::from_4096_aligned(addr);
        let table = ptr.0 as *mut _;
        P4::with_initialized(unsafe { &mut *table })
    }

    pub fn set(p4: &P4) {
        let ptr = PhysicalAddress((&***p4) as *const _ as u64);
        common::trace!("setting cr3 to {:?}", ptr);

        let mut cr3 = cr3();
        cr3.set_bit_range(51, 12, ptr.to_4096_aligned());

        unsafe {
            asm!("mov cr3, {0}", in(reg) cr3);
        }
    }
}
