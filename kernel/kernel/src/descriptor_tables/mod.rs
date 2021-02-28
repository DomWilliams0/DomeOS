mod common;
mod gdt;
mod idt;
mod tss;

pub fn init() {
    gdt::init();
    idt::init();
}
