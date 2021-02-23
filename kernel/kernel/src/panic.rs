use crate::hang;
use common::*;
use core::fmt::{Display, Formatter};
use core::panic::PanicInfo;
use memory::kilobytes;

static mut PANICKED: bool = false;

// linker map is packed by helpers/ld-link-map then patched into this array after build by
// helpers/patcher
const PACKED_SYMBOLS_COUNT: usize = kilobytes(512) as usize / core::mem::size_of::<u32>();
const PACKED_SYMBOLS_MARKER: u32 = 0xbeef_face;
static PACKED_SYMBOLS: [u32; PACKED_SYMBOLS_COUNT] = [PACKED_SYMBOLS_MARKER; PACKED_SYMBOLS_COUNT];

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
pub struct Frame {
    idx: usize,
    ptr: *const u64,
    symbol: Option<(u64, &'static str)>,
}

pub fn backtrace(mut per_frame: impl FnMut(Frame)) {
    let symbols = {
        let first_word = unsafe {
            let ptr = &PACKED_SYMBOLS as *const u32;
            // the read here must be volatile to avoid the compiler optimising it away without
            // knowing it's been patched in to the binary post-build
            let word = ptr.read_volatile();
            trace!("packed symbols are at {:?}, first word is {:#x}", ptr, word);
            word
        };
        if first_word == PACKED_SYMBOLS_MARKER {
            warn!("packed symbols not patched in, backtrace not available");
            None
        } else {
            let packed_symbols: &[u8] = unsafe {
                let byte_len = PACKED_SYMBOLS.len() * core::mem::size_of_val(&PACKED_SYMBOLS[0]);
                let ptr = PACKED_SYMBOLS.as_ptr() as *const u8;
                core::slice::from_raw_parts(ptr, byte_len)
            };

            Some(ld_link_map::packed::iter_entries(packed_symbols))
        }
    };

    let mut rbp = {
        let rbp: u64;
        unsafe {
            llvm_asm!("mov %rbp, $0" : "=r" (rbp));

            rbp as *const u64
        }
    };

    let mut idx = 0;
    while !rbp.is_null() {
        let (addr, symbol) = {
            // deref +8 bytes to get function return address is contained in
            let ret_addr = unsafe { *rbp.offset(1) };
            let symbol = symbols
                .clone()
                .and_then(|symbols| ld_link_map::packed::resolve_entry(symbols, ret_addr))
                .map(|entry| {
                    let offset = ret_addr - entry.address;
                    (offset, entry.name)
                });
            (ret_addr, symbol)
        };

        per_frame(Frame {
            idx,
            ptr: addr as *const _,
            symbol,
        });

        // move on
        idx += 1;
        unsafe {
            rbp = *rbp as *const _;
        }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "  {:2}: [{:?}]", self.idx, self.ptr)?;
        if let Some((offset, symbol)) = self.symbol {
            write!(f, " {}+{:#x}", symbol, offset)?;
        }

        Ok(())
    }
}
