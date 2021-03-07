mod common;
mod gdt;
mod idt;
mod tss;

pub use gdt::{SEL_KERNEL_CODE, SEL_KERNEL_DATA, SEL_USER_CODE, SEL_USER_DATA};
pub use tss::tss;

pub fn init() {
    gdt::init();
    idt::init();
}
