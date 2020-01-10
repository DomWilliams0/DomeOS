#![no_std]
#![no_main]
#![feature(maybe_uninit_ref)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(panic_info_message)]

use core::ffi::c_void;
use core::fmt::Write;
use core::panic::PanicInfo;

use log::error;

use crate::irq::disable_interrupts;

mod clock;
mod exception;
mod idt;
mod io;
mod irq;
mod serial;
mod start;
mod vga;

#[no_mangle]
pub extern "C" fn kernel_main(magic: i32, multiboot: *mut c_void) -> ! {
    start::kernel_main(magic, multiboot)
}

pub fn hang() -> ! {
    unsafe {
        disable_interrupts();

        loop {
            asm!("hlt");
        }
    }
}

// TODO dump regs
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if vga::is_initialized() {
        vga::set_error_colors();
        if let Some(msg) = panic_info.message() {
            let mut screen = vga::get();
            write!(screen, "panic occurred: {}", msg).unwrap();
        } else {
            println!("panic occurred: {:?}", panic_info);
        }
    }

    // log to serial
    error!("panic occurred: {:?}", panic_info);

    hang();
}
