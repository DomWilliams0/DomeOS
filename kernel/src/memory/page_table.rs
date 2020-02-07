use kernel_utils::memory::address::PhysicalAddress;
use kernel_utils::memory::page_table::hierarchy::{P3, P4};
use kernel_utils::memory::page_table::PageTable;
use kernel_utils::prelude::*;

fn cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r" (value));
    }
    value
}

pub fn pml4() -> P4<'static> {
    let addr = cr3().bit_range(51, 12);
    let ptr = PhysicalAddress::from_4096_aligned(addr);
    let table = ptr.0 as *mut PageTable<'static, P3<'static>>;
    P4::PML4T(unsafe { &mut *table })
}

pub fn set_pml4(p4: P4<'static>) {
    let P4::PML4T(table) = p4;
    let ptr = PhysicalAddress(table as *const PageTable<'static, P3<'static>> as u64);

    let mut cr3 = cr3();
    cr3.set_bit_range(51, 12, ptr.to_4096_aligned());

    unsafe {
        asm!("mov $0, %cr3" :: "r" (cr3) : "memory");
    }
}
pub fn walk_active_page_hierarchy() {
    // TODO

    //        let P4::PML4T(table) = pml4();
    //        for (i, e) in table.present_entries() {
    //            info!("pml4e #{}: {:#?}", i, e);
    //
    //            let P3::PDPT(table) = P3::traverse(e);
    //            for (i, e) in table.present_entries() {
    //                info!("  pdpe #{}: {:#?}", i, e);
    //
    //                if let P2::PDT(table) = P2::traverse(e) {
    //                    for (i, e) in table.present_entries() {
    //                        info!("    pde #{}: {:#?}", i, e);
    //
    //                        if let P1::PT(table) = P1::traverse(e) {
    //                            for (i, e) in table.present_entries() {
    //                                info!("      pte #{}: {:#?}", i, e);
    //                            }
    //                        }
    //                    }
    //                }
    //            }
    //        }
}
