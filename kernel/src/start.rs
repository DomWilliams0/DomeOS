use crate::multiboot;

use log::*;

use crate::irq::enable_interrupts;
use crate::vga::{self, Color};
use crate::{clock, idt};
use crate::{print, println};

pub fn start(multiboot: &multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::DarkGray);

    // TODO parse multiboot

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
