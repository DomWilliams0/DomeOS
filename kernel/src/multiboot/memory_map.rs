use crate::memory::PhysicalAddress;
use crate::multiboot::memory_map::MemoryRegionType::{Acpi, Available, Defective,
                                                     PreserveOnHibernation, Reserved};
use crate::multiboot::{multiboot_info, multiboot_memory_map_t, multiboot_mmap_entry};
use log::*;
use bitfield::Bit;

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

#[derive(Debug)]
pub struct MemoryRegion {
    pub base_addr: PhysicalAddress,
    pub length: u64,
    pub region_type: MemoryRegionType,
}

impl From<&multiboot_memory_map_t> for MemoryRegion {
    fn from(mmap: &multiboot_mmap_entry) -> Self {
        Self {
            base_addr: PhysicalAddress(mmap.addr),
            length: mmap.len,
            region_type: mmap.type_.into(),
        }
    }
}

pub struct MemoryRegions {
    current: *mut multiboot_memory_map_t,
    end: *mut multiboot_memory_map_t,
}

impl MemoryRegions {
    pub fn new(mbi: &multiboot_info) -> Self {
        assert!(mbi.flags.bit(6), "memory map isn't available");

        let start = mbi.mmap_addr as *mut multiboot_memory_map_t;
        let end = (mbi.mmap_addr + mbi.mmap_length) as *mut multiboot_memory_map_t;
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for MemoryRegions {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current >= self.end {
                break None;
            } else {
                // Safety: current is < self.end
                let mmap = unsafe { &*self.current };
                let region = MemoryRegion::from(mmap);

                // Safety: move on to next in bytes, not multiples of multiboot_memory_map_t
                self.current = unsafe {
                    let bytes_offset = mmap.size as usize + core::mem::size_of_val(&mmap.size);
                    let ptr = self.current as *mut u8;
                    let ptr = ptr.add(bytes_offset);
                    ptr as *mut multiboot_memory_map_t
                };

                // Ignore length == 0
                if region.length == 0 {
                    continue;
                } else {
                    break Some(region);
                }
            }
        }
    }
}
