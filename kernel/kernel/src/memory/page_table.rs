use bitfield::BitRange;
use common::*;
use memory::{PageTable, PhysicalAddress, P1, P2, P3, P4};

fn cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {0}, cr3", out(reg) value);
    }
    value
}

pub fn pml4<'p>() -> P4<'p> {
    let addr = cr3().bit_range(51, 12);
    let ptr = PhysicalAddress::from_4096_aligned(addr);
    let table = ptr.0 as *mut PageTable<'p, P3<'p>>;
    P4::with_initialized(unsafe { &mut *table })
}

pub fn set_pml4(p4: &P4<'static>) {
    let ptr = PhysicalAddress((&***p4) as *const PageTable<'static, P3<'static>> as u64);

    let mut cr3 = cr3();
    cr3.set_bit_range(51, 12, ptr.to_4096_aligned());

    unsafe {
        asm!("mov cr3, {0}", in(reg) cr3);
    }
}
pub fn log_active_page_hierarchy() {
    let p4 = pml4();
    for (i, e) in p4.present_entries() {
        info!("pml4 {}: {:?}", i, e);

        let p3 = e.traverse().unwrap();
        for (i, e) in p3.present_entries() {
            info!(" pdp {}: {:?}", i, e);

            if let P2::PDT(p2) = e.traverse().unwrap() {
                for (i, e) in p2.present_entries() {
                    info!("  pde {}: {:?}", i, e);

                    if let P1::PT(p1) = e.traverse().unwrap() {
                        for (i, e) in p1.present_entries() {
                            info!("   pte {}: {:?}", i, e);
                        }
                    }
                }
            }
        }
    }
}
