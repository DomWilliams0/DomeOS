#![no_std]
// lints
#![allow(dead_code)]
#![allow(clippy::module_inception)]
// features
#![feature(maybe_uninit_ref)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(asm)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(const_ptr_offset_from, const_maybe_uninit_as_ptr, const_raw_ptr_deref)]
#![feature(naked_functions)]

extern crate alloc;

use core::ffi::c_void;

use common::*;

use crate::irq::disable_interrupts;

mod clock;
mod cpu;
mod descriptor_tables;
mod exception;
mod io;
mod irq;
mod logging;
mod memory;
mod multiboot;
mod panic;
mod process;
mod serial;
mod spinlock;
mod start;
mod syscall;
mod vga;

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, multiboot: *mut c_void) -> ! {
    zero_bss();
    logging::init(LevelFilter::Trace); // TODO configure this

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
    unsafe {
        disable_interrupts();
        warn!("hanging forever");

        loop {
            asm!("hlt");
        }
    }
}

extern "C" {
    #[link_name = "_bss"]
    static mut BSS_START: usize;

    #[link_name = "_bss_end"]
    static mut BSS_END: usize;
}

fn zero_bss() {
    unsafe {
        let start = (&mut BSS_START) as *mut _ as *mut u8;
        let end = (&mut BSS_END) as *mut _ as *mut u8;
        let len = end.offset_from(start);

        core::intrinsics::volatile_set_memory(start, 0, len as usize);
    }
}
