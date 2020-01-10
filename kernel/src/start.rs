use core::ffi::c_void;

use log::*;

use crate::irq::enable_interrupts;
use crate::vga::{self, Color};
use crate::{clock, idt, serial};
use crate::{print, println};

pub fn kernel_main(_magic: i32, _multiboot: *mut c_void) -> ! {
    serial::init(LevelFilter::Trace); // TODO configure this
    vga::init(Color::LightGreen, Color::DarkGray);

    idt::init();
    clock::init();
    enable_interrupts();

    println!(
        "a line that is very long and most certainly wraps when it gets to the edge of the screen"
    );

    loop {}
}

fn breakpoint() {
    unsafe { asm!("int3") }
}

fn div_by_zero() {
    unsafe { asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel") }
}
