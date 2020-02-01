use crate::{memory, multiboot};

use log::*;

use bitfield::Bit;
use crate::irq::enable_interrupts;
use crate::multiboot::{MemoryRegionType, MemoryRegions};
use crate::serial::LogMode;
use crate::vga::{self, Color};
use crate::{clock, idt, serial};

pub fn start(multiboot: &multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    serial::set_log_mode(LogMode::SerialAndVga);

    idt::init();
    clock::init();
    enable_interrupts();

    parse_multiboot(multiboot);

    loop {}
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

    info!("current page table is {:#?}", memory::PageTable::load());

    // register available memory regions
    memory::free_pages::init_free_pages(MemoryRegions::new(multiboot));
}

fn breakpoint() {
    unsafe { asm!("int3") }
}

fn div_by_zero() {
    unsafe { asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel") }
}
