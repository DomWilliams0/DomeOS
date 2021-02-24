use common::*;

use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::multiboot;
use crate::multiboot::Multiboot;
use crate::vga::{self, Color};
use crate::{clock, descriptor_tables, logging};

// TODO guard page to detect and handle stack overflow
pub fn start(multiboot: &'static multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    logging::set_log_mode(LogMode::SerialAndVga);

    descriptor_tables::init();
    clock::init();

    let multiboot = Multiboot::new(multiboot);

    // init memory and get ourselves a heap
    if let Err(err) = crate::memory::init(multiboot) {
        error!("early memory setup failed: {}", err);
        hang();
    }

    // now we have a heap we can start using boxed error types

    // finally enable interrupts now that the higher half mappings are in place, so the isrs are
    // actually mapped
    enable_interrupts();

    // other kernel init
    let init_result = (|| -> anyhow::Result<()> {
        // TODO

        // play around with exe loading
        crate::process::spawn_process()?;

        Ok(())
    })();

    if let Err(err) = init_result {
        error!("init failed: {:?}", err);
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
