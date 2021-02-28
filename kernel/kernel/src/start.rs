use common::*;

use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::memory::AddressSpace;
use crate::multiboot;
use crate::multiboot::Multiboot;
use crate::vga::{self, Color};
use crate::{clock, descriptor_tables, logging};
use memory::{kilobytes, megabytes, VirtualAddress, FRAME_SIZE};

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

    // relocate stack to include guard page
    let init_result = relocate_stack_then_post_init();

    if let Err(err) = init_result {
        error!("init failed: {:?}", err);
    }

    info!("goodbye!");
    hang();
}

fn post_init() -> anyhow::Result<()> {
    // play around with exe loading
    crate::process::spawn_process()?;

    Ok(())
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

fn relocate_stack_then_post_init() -> anyhow::Result<()> {
    // TODO move to constants
    const KERNEL_STACKS_START: u64 = 0xffff_8000_0000_0000;
    const KERNEL_STACK_MAX_SIZE: u64 = megabytes(8);
    const KERNEL_STACK_SIZE: u64 = kilobytes(64) / FRAME_SIZE;
    const KERNEL_STACKS_MAX: u64 = 512;

    let core_id = 0; // TODO actually multicore
    assert!(core_id < KERNEL_STACKS_MAX, "core={}", core_id);

    let stack_start =
        VirtualAddress::with_literal(KERNEL_STACKS_START + (core_id * KERNEL_STACK_MAX_SIZE));

    let new_stack = crate::process::allocate_kernel_stack(
        &mut AddressSpace::current(),
        stack_start,
        KERNEL_STACK_SIZE as usize,
    )
    .map_err(Error::msg)?;

    debug!("new stack allocated at {:?}", new_stack);

    unsafe {
        asm!(
        "mov rsp, {stack_top}",
        stack_top = in(reg) new_stack.address(),
        )
    };

    // on new stack
    post_init()
}
