#[cfg(debug_assertions)]
pub use i8259::is_in_interrupt;
pub use i8259::remap as i8259_remap;
pub use i8259::InterruptContext;
pub use init::init;

mod apic;
mod i8259;
mod init;

pub fn enable_interrupts() {
    common::debug!("enabling interrupts");
    unsafe { asm!("sti") };
}

pub fn disable_interrupts() {
    common::debug!("disabling interrupts");
    unsafe { asm!("cli") };
}
