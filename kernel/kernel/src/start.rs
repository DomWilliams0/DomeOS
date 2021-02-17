use log::*;

use utils::KernelResult;

use crate::irq::{disable_interrupts, enable_interrupts};
use crate::multiboot::Multiboot;
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

    let multiboot = Multiboot::new(multiboot);

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
