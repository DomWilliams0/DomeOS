use crate::{multiboot};

use log::*;

use bitfield::Bit;
use crate::irq::enable_interrupts;
use crate::multiboot::{MemoryRegions};
use crate::vga::{self, Color};
use crate::{clock, idt};
use crate::{print, println};

pub fn start(multiboot: &multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::DarkGray);

    idt::init();
    clock::init();
    enable_interrupts();

    parse_multiboot(multiboot);

    println!(
        "a line that is very long and most certainly wraps when it gets to the edge of the screen"
    );

    loop {}
}

fn parse_multiboot(multiboot: &multiboot::multiboot_info) {
    println!("multiboot flags: {:#b}", multiboot.flags);

    if multiboot.flags.bit(0) {
        println!(
            "mem {:#x} -> {:#x}",
            multiboot.mem_lower, multiboot.mem_upper
        );
    }

    multiboot::print_commandline(multiboot);

    for memory_region in MemoryRegions::new(multiboot) {
        info!("memory region: {:#?}", memory_region);
    }
}

fn breakpoint() {
    unsafe { asm!("int3") }
}

fn div_by_zero() {
    unsafe { asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel") }
}
