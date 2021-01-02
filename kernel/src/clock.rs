use core::ops::{BitAnd, Shr};
use core::time::Duration;

use modular_bitfield::prelude::*;

use crate::io::Port;
use crate::irq;

const PIT_CHANNEL0_DATA: Port = Port(0x40);
// const PIT_CHANNEL2_DATA: Port = Port(0x42);
const PIT_COMMAND: Port = Port(0x43);

const TICKS_PER_SECOND: u64 = 120;

/// Total number of ticks since boot
static mut TICKS: u64 = 0;

#[bitfield]
struct PitCommand {
    /// 0: 16 bit binary, 1: 0000-9999
    bcd_or_binary: B1,

    /// Operating mode
    /// e.g. 011: square wave
    mode: B3,

    // Access mode
    // lobyte only, hibyte only, lobyte + hibyte
    rw: B2,

    // Channel 0 - 2
    channel: B2,
}

fn set_interval(hz: u64) {
    let divisor = 1193180 / hz;
    assert_ne!(divisor, 0);

    let cmd = {
        let mut cmd = PitCommand::new();
        cmd.set_bcd_or_binary(0); // binary
        cmd.set_mode(3); // square wave
        cmd.set_rw(3); // lo and hi bytes
        cmd.set_channel(0); // channel 0
        cmd
    };

    unsafe {
        // write command
        PIT_COMMAND.write_u8(core::mem::transmute(cmd));

        // write divisor
        PIT_CHANNEL0_DATA.write_u8(divisor.bitand(0xFF) as u8); // lo
        PIT_CHANNEL0_DATA.write_u8(divisor.shr(8) as u8); // hi
    }
}
extern "C" fn on_clock(_ctx: *const irq::InterruptContext) {
    unsafe {
        TICKS += 1;
    };
}

pub fn init() {
    set_interval(TICKS_PER_SECOND);
    irq::register_handler(irq::Irq::Clock, on_clock);
}

pub fn since_boot() -> Duration {
    let seconds: f64 = unsafe { TICKS as f64 / TICKS_PER_SECOND as f64 };
    Duration::from_secs_f64(seconds)
}
