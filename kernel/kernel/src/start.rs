use log::*;

use utils::prelude::*;
use utils::KernelResult;

use crate::irq::{disable_interrupts, enable_interrupts};

use crate::serial::LogMode;
use crate::vga::{self, Color};
use crate::{clock, descriptor_tables, serial};
use crate::{memory, multiboot};

// TODO guard page to detect and handle stack overflow
pub fn start(multiboot: &'static multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    serial::set_log_mode(LogMode::SerialAndVga);

    descriptor_tables::init();
    clock::init();

    parse_multiboot(multiboot);

    let init_result = (|| -> KernelResult<()> {
        // set up page tables for desired mapping
        memory::init(multiboot)?;

        // finally enable interrupts now that the higher half mappings are in place, so the isrs are
        // actually mapped
        enable_interrupts();

        Ok(())
    })();

    if let Err(err) = init_result {
        error!("setup failed: {}", err);
    }

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
