#![no_std]
#![no_main]
#![feature(maybe_uninit_ref)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(const_in_array_repeat_expressions)]
use core::ffi::c_void;
use core::panic::PanicInfo;

use log::*;

use crate::irq::disable_interrupts;

mod clock;
mod exception;
mod idt;
mod io;
mod irq;
mod memory;
mod multiboot;
mod serial;
mod start;
mod vga;

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, multiboot: *mut c_void) -> ! {
    serial::init(LevelFilter::Trace); // TODO configure this

    match multiboot::parse(magic, multiboot) {
        Ok(multiboot) => {
            // Safety: parse succeeded
            let multiboot = unsafe { &*multiboot };
            start::start(multiboot)
        }
        Err(e) => panic!("failed to boot: {}", e),
    }
}

pub fn hang() -> ! {
    warn!("hanging forever");
    unsafe {
        disable_interrupts();

        loop {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    // log to serial and vga if enabled
    error!("panic occurred: {:?}", panic_info);

    // TODO dump regs

    hang();
}
