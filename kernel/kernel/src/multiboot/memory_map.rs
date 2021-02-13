use core::fmt::{Debug, Error as FmtError, Formatter};
use core::ops::Range;

use utils::memory::address::PhysicalAddress;
use utils::prelude::Bit;

use crate::multiboot::memory_map::MemoryRegionType::{
    Acpi, Available, Defective, PreserveOnHibernation, Reserved,
};
use crate::multiboot::{multiboot_info, multiboot_memory_map_t, multiboot_mmap_entry};
use core::iter;
use core::mem;

#[derive(Debug)]
pub enum MemoryRegionType {
    Available,
    Acpi,
    PreserveOnHibernation,
    Defective,
    Reserved(u32),
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

pub struct MemoryRegion {
    pub base_addr: PhysicalAddress,
    pub length: u64,
    pub region_type: MemoryRegionType,
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

    pub fn iter_from_multiboot(mbi: &multiboot_info) -> impl Iterator<Item = Self> + Clone {
        assert!(mbi.flags.bit(6), "memory map isn't available");

        let start = mbi.mmap_addr as *mut multiboot_memory_map_t;
        let end = (mbi.mmap_addr + mbi.mmap_length) as *mut multiboot_memory_map_t;

        let mut current = start;
        iter::from_fn(move || {
            while current < end {
                // safety: current is < self.end
                let mmap = unsafe { &*current };
                let region = MemoryRegion::new(mmap);

                // advance by number of bytes in entry, NOT multiples of struct
                current = unsafe {
                    let bytes_offset = mmap.size as usize + mem::size_of_val(&mmap.size);
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
