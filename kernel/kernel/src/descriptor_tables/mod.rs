mod common;
mod gdt;
mod idt;

pub fn init() {
    gdt::init();
    idt::init();
}
