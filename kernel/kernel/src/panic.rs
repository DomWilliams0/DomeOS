use crate::hang;
use core::fmt::{Display, Formatter};
use core::panic::PanicInfo;
use log::*;

static mut PANICKED: bool = false;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    if is_panicking() {
        error!("nested panic: {:#?}", panic_info);
        hang();
    }

    unsafe {
        PANICKED = true;
    }

    // log to serial and vga if enabled
    error!("panic occurred: {:#?}", panic_info);

    error!("generating stack trace");
    backtrace(|frame| {
        error!("{}", frame);
    });

    // TODO dump regs

    hang()
}

pub fn is_panicking() -> bool {
    // safety: switch flipped false->true once only
    unsafe { PANICKED }
}

#[derive(Debug)]
struct Frame {
    idx: usize,
    ptr: *const u64,
}

fn backtrace(mut per_frame: impl FnMut(Frame)) {
    let mut addr = {
        let rbp: u64;
        unsafe {
            llvm_asm!("mov %rbp, $0" : "=r" (rbp));

            rbp as *const u64
        }
    };

    let mut idx = 0;
    while !addr.is_null() {
        per_frame(Frame::resolve(addr, idx));

        // move on
        idx += 1;
        unsafe {
            addr = *addr as *const _;
        }
    }
}

impl Frame {
    fn resolve(rbp: *const u64, idx: usize) -> Self {
        // TODO resolve symbol
        Self { idx, ptr: rbp }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "  {:2}: [{:?}]", self.idx, self.ptr)
    }
}
