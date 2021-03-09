pub use gdt::{SEL_KERNEL_CODE, SEL_KERNEL_DATA, SEL_USER_CODE, SEL_USER_DATA};
pub use tss::tss;

mod gdt;
mod idt;
mod tss;

pub fn init() {
    gdt::init();
    idt::init();
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}
