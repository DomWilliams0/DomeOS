use buddy_alloc::buddy_alloc::BuddyAlloc;
use crate::memory::PhysicalAddress;
use crate::multiboot::{MemoryRegionType, MemoryRegions};
use log::*;

/// Arbitrary limit because we have no allocation yet, and so far I've yet to see
/// more than a 2 available memory regions
const MAX_PAGE_REGIONS: usize = 4;
static mut PAGE_REGIONS: [Option<BuddyAlloc>; MAX_PAGE_REGIONS] = [None; MAX_PAGE_REGIONS];

/// Number of page regions, will be >0
static mut PAGE_REGION_COUNT: usize = 0;

pub fn init_free_pages(regions: MemoryRegions) {
    for region in regions.filter(|r| r.region_type == MemoryRegionType::Available) {
        let idx = unsafe { PAGE_REGION_COUNT };

        if idx >= MAX_PAGE_REGIONS {
            warn!("got too many available page regions, stopping early");
            break;
        }

        // roundup of addr 0x0000 underflows...... TODO skip for now
        if region.base_addr == PhysicalAddress(0) {
            debug!("skipping memory region at 0x0");
            continue;
        }

        info!(
            "registering memory region #{} of length {} at {:?}",
            idx, region.length, region.base_addr
        );

        // TODO these regions arent mapped yet
/*
        let mut ptr = region.base_addr.0 as *mut i32;
        unsafe {
            loop {
                debug!("gonna write to {:?}", ptr);
                *ptr = 'A' as i32;
                ptr = ptr.add(2);
            }
        }
*/

        /*
        unsafe {
            let alloc = BuddyAlloc::new(
                region.base_addr.0 as *const u8,
                region.length as usize,
                4096,
            );

            PAGE_REGIONS[idx] = Some(alloc);
            PAGE_REGION_COUNT+=1;
        }
*/
    }

    // TODO result instead
    unsafe {
        assert_ne!(
            PAGE_REGION_COUNT, 0,
            "No available memory regions discovered"
        );
    }
}