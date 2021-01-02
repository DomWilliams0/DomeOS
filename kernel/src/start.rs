use log::*;

use kernel_utils::prelude::*;

use crate::irq::{disable_interrupts, enable_interrupts};
use crate::multiboot::{MemoryRegion, MemoryRegionType};
use crate::serial::LogMode;
use crate::vga::{self, Color};
use crate::{clock, idt, serial};
use crate::{memory, multiboot};
use kernel_utils::memory::address::PhysicalAddress;

pub fn start(multiboot: &multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    serial::set_log_mode(LogMode::SerialAndVga);

    idt::init();
    clock::init();
    enable_interrupts();

    parse_multiboot(multiboot);

    hang();
}

fn parse_multiboot(multiboot: &multiboot::multiboot_info) {
    debug!("multiboot flags: {:#b}", multiboot.flags);

    if multiboot.flags.bit(0) {
        debug!(
            "memory range: {}KiB -> {}KiB",
            multiboot.mem_lower, multiboot.mem_upper
        );
    }

    multiboot::print_commandline(multiboot);

    // memory::walk_active_page_hierarchy();
    memory::init(multiboot);
    //
    //    memory::remap_kernel();
    //
    //    // register available memory regions
    //    memory::free_pages::init_free_pages(MemoryRegions::new(multiboot));
}

fn breakpoint() {
    unsafe { llvm_asm!("int3") }
}

fn div_by_zero() {
    unsafe { llvm_asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel") }
}

fn hang() -> ! {
    disable_interrupts();

    info!("hanging forever");
    loop {
        unsafe { llvm_asm!("hlt") }
    }
}
