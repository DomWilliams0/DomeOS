use crate::memory::{frame_allocator, heap, phys, AddressSpace, FrameAllocator};
use crate::multiboot::{MemoryRegionType, Multiboot, MultibootMemoryMap};
use crate::vga;
use common::*;
use memory::{
    gigabytes, iter_all_pages, megabytes, round_up_to, MemoryError, PageTableHierarchy,
    PhysicalAddress, VirtualAddress, P4, PAGE_TABLE_ENTRY_COUNT, VIRT_PHYSICAL_BASE,
    VIRT_PHYSICAL_SIZE,
};

pub fn init(multiboot: Multiboot) -> Result<(), MemoryError> {
    // extract mmap from multiboot
    let memory_map = multiboot.memory_map().expect("memory map unavailable");
    debug!("memory map from multiboot: ");
    for region in memory_map.iter_regions() {
        debug!("* {:?}", region);
    }

    // init physical frame allocator
    phys::init_frame_allocator(memory_map);

    // setup physical identity mapping
    let mut addr_space = AddressSpace::current();
    init_physical_identity_mapping(&mut *addr_space.pml4_mut(), &memory_map)?;
    post_init_physical_identity_mapping(&memory_map, &mut *addr_space.pml4_mut());

    // init heap
    heap::init()?;

    Ok(())
}

/// Setup huge physical identity mapping
///
/// TODO marking higher half pages as 'global' doesnt work in virtualbox
///  "page fault on present page: PageFaultException { flags: (PAGE_PROTECTION_VIOLATION | CAUSED_BY_READ | RESERVED_WRITE), addr: v0xffff900000000000 },"
fn init_physical_identity_mapping(
    p4: &mut P4,
    memory_map: &MultibootMemoryMap,
) -> Result<(), MemoryError> {
    // find highest physical address
    let phys_max = memory_map
        .iter_regions()
        .filter(|region| matches!(region.region_type, MemoryRegionType::Available))
        .map(|region| region.base_addr + region.length)
        .max_by_key(|addr| addr.address())
        .unwrap(); // at least 1 region expected

    // round up to nearest 2MB
    let phys_max = round_up_to(phys_max.address(), megabytes(2));
    let base = VirtualAddress::with_literal(VIRT_PHYSICAL_BASE);
    debug!(
        "identity mapping {:.2}GB ({:.2}MB) of physical memory from {:?}",
        phys_max as f32 / gigabytes(1) as f32,
        phys_max as f32 / megabytes(1) as f32,
        base
    );

    // never gonna happen i tell you
    assert!(phys_max < VIRT_PHYSICAL_SIZE, "too much physical memory?");

    // entry index into each table
    let mut tables = [
        base.pml4t_offset(),
        base.pdp_offset(),
        base.pd_offset(),
        base.pt_offset(),
    ];

    let mut pages = iter_all_pages(base, base + phys_max);

    trace!(
        "mapping {}.{}.{}.{} => {}.{}.{}.{}",
        base.pml4t_offset(),
        base.pdp_offset(),
        base.pd_offset(),
        base.pt_offset(),
        (base + phys_max).pml4t_offset(),
        (base + phys_max).pdp_offset(),
        (base + phys_max).pd_offset(),
        (base + phys_max).pt_offset(),
    );

    const KEEP_LOOPING: u16 = 4;
    let mut unroll = KEEP_LOOPING;

    let p4 = p4.table_mut();
    'outer: while unroll > 0 {
        let p4_entry = p4.entry_physical_mut(tables[0]);

        // ensure all allocated frames are already writeable with allocate_premapped
        let p3_frame = frame_allocator().allocate_premapped()?;
        unsafe {
            // safety: frame is premapped
            p3_frame.zero_in_place();
        }

        p4_entry
            .replace()
            .address(p3_frame.address())
            .higher_half()
            .apply();

        unroll = KEEP_LOOPING;
        while unroll > 1 {
            let mut p3 = p4_entry.traverse().expect("p3 was just mapped");
            let p3_table = p3.table_mut().expect("p3 is not huge");
            let p3_entry = p3_table.entry_physical_mut(tables[1]);

            let p2_frame = frame_allocator().allocate_premapped()?;
            unsafe {
                // safety: frame is premapped
                p2_frame.zero_in_place();
            }
            p3_entry
                .replace()
                .address(p2_frame.address())
                .higher_half()
                .apply();

            let mut p2 = p3_entry.traverse().expect("p2 was just mapped");
            let p2_table = p2.table_mut().expect("p2 is not huge");

            unroll = KEEP_LOOPING;
            while unroll > 2 {
                let new_tables = match pages.next() {
                    Some((n, to_unroll, new_indices)) => {
                        // multiples of 2M only
                        assert_eq!(n, PAGE_TABLE_ENTRY_COUNT as u16);

                        unroll = to_unroll;
                        new_indices
                    }
                    None => {
                        // finished
                        break 'outer;
                    }
                };

                // initialize p2 entries to each point to 2MB each
                let addr = gigabytes(tables[1] as u64) + megabytes(2 * tables[2] as u64);
                p2_table
                    .entry_physical_mut(tables[2])
                    .replace()
                    .address(PhysicalAddress(addr))
                    .huge() // 2MB
                    .higher_half()
                    .apply();

                tables = new_tables;
            }
        }
    }

    AddressSpace::invalidate_tlb();

    Ok(())
}

fn post_init_physical_identity_mapping(memory_map: &MultibootMemoryMap, p4: &mut P4) {
    // ensure frame allocator uses virtual multiboot pointer now
    frame_allocator().relocate_multiboot(unsafe {
        let phys = PhysicalAddress(memory_map.pointer() as u64);
        let virt = VirtualAddress::from_physical(phys);
        &*virt.as_ptr()
    });

    // update VGA to use new offset address
    // safety: just mapped physical identity map
    unsafe {
        let new_addr = VirtualAddress::from_physical(PhysicalAddress(0xb8000));
        trace!("moving VGA buffer to {:?}", new_addr);
        vga::get().move_vga_buffer(new_addr);
        trace!("it worked!");
    }

    // now safe to remove low identity maps from early boot
    p4.table_mut()
        .entry_physical_mut(0)
        .replace()
        .not_present()
        .apply();

    // don't bother zeroing other hardcoded p3 and p2s as we won't reference them again
}
