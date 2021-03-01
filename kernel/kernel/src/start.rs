use common::*;

use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::memory::AddressSpace;
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

    // finish initialization on a new stack. inner fn to ensure we can't use old local vars
    // TODO this is just really a demo of new stack allocation, a new stack isnt required
    fn post_init() -> ! {
        let do_it = || -> anyhow::Result<()> {
            // play around with exe loading
            // crate::process::spawn_process()?;

            // run a dummy function in userspace
            experiment_userspace();

            Ok(())
        };

        if let Err(err) = do_it() {
            error!("init failed: {:?}", err);
        }

        info!("goodbye!");
        hang();
    }

    // relocate stack and continue initialization there
    // relocate_stack_then_post_init(post_init)
    post_init()
}

fn breakpoint() {
    unsafe { asm!("int3") }
}

fn hang() -> ! {
    disable_interrupts();

    info!("hanging forever");
    loop {
        unsafe { asm!("hlt") }
    }
}

fn experiment_userspace() {
    let mut addr = AddressSpace::new().expect("damn");

    unsafe {
        addr.load();
        hang();
    }
}
