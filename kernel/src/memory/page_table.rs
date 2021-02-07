use kernel_utils::memory::address::PhysicalAddress;
use kernel_utils::memory::page_table::PageTable;
use kernel_utils::memory::{P1, P2, P3, P4};
use kernel_utils::prelude::*;
use log::*;

fn cr3() -> u64 {
    let value: u64;
    unsafe {
        llvm_asm!("mov %cr3, $0" : "=r" (value));
    }
    value
}

pub fn pml4() -> P4<'static> {
    let addr = cr3().bit_range(51, 12);
    let ptr = PhysicalAddress::from_4096_aligned(addr);
    let table = ptr.0 as *mut PageTable<'static, P3<'static>>;
    P4(unsafe { &mut *table })
}

pub fn set_pml4(p4: P4<'static>) {
    let P4(table) = p4;
    let ptr = PhysicalAddress(table as *const PageTable<'static, P3<'static>> as u64);

    let mut cr3 = cr3();
    cr3.set_bit_range(51, 12, ptr.to_4096_aligned());

    unsafe {
        llvm_asm!("mov $0, %cr3" :: "r" (cr3) : "memory");
    }
}
pub fn log_active_page_hierarchy() {
    let P4(p4) = pml4();
    for (i, e) in p4.present_entries() {
        info!("pml4e {}: {:?}", i, e);

        let P3(p3) = e.traverse().unwrap();
        for (i, e) in p3.present_entries() {
            info!(" pdpe {}: {:?}", i, e);

            match e.traverse().unwrap() {
                P2::Huge1GPage(frame) => info!("  pdpe {}: {:?}", i, frame),

                P2::PDT(p2) => {
                    for (i, e) in p2.present_entries() {
                        info!("  pde {}: {:?}", i, e);

                        match e.traverse().unwrap() {
                            P1::Huge2MPage(frame) => info!("   pde {}: {:?}", i, frame),

                            P1::PT(p1) => {
                                for (i, e) in p1.present_entries() {
                                    info!("   pte {}: {:?}", i, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
