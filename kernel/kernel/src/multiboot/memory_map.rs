use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::Range;

use crate::multiboot::memory_map::MemoryRegionType::{
    Acpi, Available, Defective, PreserveOnHibernation, Reserved,
};
use crate::multiboot::{
    multiboot_info, multiboot_memory_map_t, multiboot_mmap_entry, multiboot_uint32_t,
};
use common::Bit;
use core::iter;
use core::mem;
use memory::PhysicalAddress;

#[derive(Debug)]
pub enum MemoryRegionType {
    Available,
    Acpi,
    PreserveOnHibernation,
    Defective,
    Reserved(u32),
}

pub struct MemoryRegion {
    pub base_addr: PhysicalAddress,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

#[derive(Copy, Clone)]
pub struct MultibootMemoryMap {
    start: *const multiboot_memory_map_t,
    end: *const multiboot_memory_map_t,
}

impl Debug for MemoryRegion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "MemoryRegion({:?}, len={:#x}, type={:?})",
            self.base_addr, self.length, self.region_type
        )
    }
}

impl From<u32> for MemoryRegionType {
    fn from(val: u32) -> Self {
        match val {
            1 => Available,
            3 => Acpi,
            4 => PreserveOnHibernation,
            5 => Defective,
            _ => Reserved(val),
        }
    }
}

impl MemoryRegion {
    pub fn new(mmap: &multiboot_mmap_entry) -> Self {
        Self {
            base_addr: PhysicalAddress(mmap.addr),
            length: mmap.len,
            region_type: mmap.type_.into(),
        }
    }

    pub fn range(&self) -> Range<u64> {
        self.base_addr.0..self.base_addr.0 + self.length
    }
}

impl MultibootMemoryMap {
    pub fn new(mbi: &'static multiboot_info) -> Option<Self> {
        if mbi.flags.bit(6) {
            let start = mbi.mmap_addr as *const multiboot_memory_map_t;
            let end = (mbi.mmap_addr + mbi.mmap_length) as *const multiboot_memory_map_t;
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// # Safety
    /// Original pointers + this offset must still point to a valid memory map
    pub unsafe fn add_pointer_offset(&mut self, offset: u64) {
        self.start = (self.start as u64 + offset) as *const _;
        self.end = (self.end as u64 + offset) as *const _;
    }

    pub fn pointer(&self) -> *const multiboot_memory_map_t {
        self.start
    }

    pub fn iter_regions(&self) -> impl Iterator<Item = MemoryRegion> + Clone + '_ {
        let mut current = self.start;
        iter::from_fn(move || {
            while current < self.end {
                // safety: current is < self.end
                let mmap = unsafe { &*current };
                let region = MemoryRegion::new(mmap);

                // advance by number of bytes in entry, NOT multiples of struct
                current = unsafe {
                    let bytes_offset = mmap.size as usize + mem::size_of::<multiboot_uint32_t>();
                    let ptr = current as *mut u8;
                    ptr.add(bytes_offset) as *mut multiboot_memory_map_t
                };

                // ignore region if length == 0
                if region.length != 0 {
                    return Some(region);
                }
            }

            None
        })
    }
}
