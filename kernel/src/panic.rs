use crate::hang;
use core::panic::PanicInfo;
use log::*;

static mut PANICKED: bool = false;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    unsafe {
        PANICKED = true;
    }

    // log to serial and vga if enabled
    error!("panic occurred: {:#?}", panic_info);

    // TODO dump regs

    hang()
}

pub fn is_panicking() -> bool {
    // safety: switch flipped false->true once only
    unsafe { PANICKED }
}
