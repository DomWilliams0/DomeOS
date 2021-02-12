use log::*;

use utils::prelude::*;

use crate::irq::{disable_interrupts, enable_interrupts};

use crate::serial::LogMode;
use crate::vga::{self, Color};
use crate::{clock, idt, serial};
use crate::{memory, multiboot};

pub fn start(multiboot: &'static multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    serial::set_log_mode(LogMode::SerialAndVga);

    idt::init();
    clock::init();
    enable_interrupts();

    parse_multiboot(multiboot);

    info!("goodbye!");
    hang();
}

fn parse_multiboot(multiboot: &'static multiboot::multiboot_info) {
    debug!("multiboot flags: {:#b}", multiboot.flags);

    if multiboot.flags.bit(0) {
        debug!(
            "memory range: {}KiB -> {}KiB",
            multiboot.mem_lower, multiboot.mem_upper
        );
    }

    multiboot::log_command_line(multiboot);

    memory::init(multiboot);
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
