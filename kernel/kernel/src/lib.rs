#![no_std]
// lints
#![allow(dead_code)]
#![allow(clippy::module_inception)]
// features
#![feature(maybe_uninit_ref)]
#![feature(abi_x86_interrupt)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]
#![feature(asm)]

use core::ffi::c_void;

use log::*;

use crate::irq::disable_interrupts;

mod clock;
mod descriptor_tables;
mod error;
mod exception;
mod io;
mod irq;
mod memory;
mod multiboot;
mod panic;
mod serial;
mod spinlock;
mod start;
mod vga;

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, multiboot: *mut c_void) -> ! {
    zero_bss();
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
            llvm_asm!("hlt");
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
    let bss = unsafe {
        let start = (&mut BSS_START) as *mut _ as *mut u8;
        let end = (&mut BSS_END) as *mut _ as *mut u8;
        let len = end.offset_from(start);

        core::slice::from_raw_parts_mut(start, len as usize)
    };

    bss.fill(0);
}
