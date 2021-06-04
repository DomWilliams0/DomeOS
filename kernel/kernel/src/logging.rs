use core::fmt::Write;
use core::mem::MaybeUninit;

use common::{Level, LevelFilter, Log, Metadata, Record};

use crate::io::Port;
use crate::panic::is_panicking;
use crate::serial::{SerialPort, SerialRegister};
use crate::spinlock::SpinLock;
use crate::vga::Color;
use crate::{clock, vga};

static mut COM1: SerialPort = SerialPort::new(Port::new(0x3F8), false);

static mut SERIAL_LOGGER: MaybeUninit<LockedSerialLogger> = MaybeUninit::uninit();

pub enum LogMode {
    SerialOnly,
    SerialAndVga,
}

struct LockedSerialLogger(SpinLock<()>, LevelFilter, LogMode);

pub fn init(log_level: LevelFilter) {
    unsafe {
        // disable interrupts
        COM1.write(SerialRegister::InterruptEnable, 0x00);

        // enable DLAB by setting high bit
        COM1.write(SerialRegister::LineControl, 0x80);

        // set lo and hi bytes of baud rate
        // default (115200) / 6 = 19200
        COM1.write(SerialRegister::DivisorLsb, 0x03);
        COM1.write(SerialRegister::DivisorMsb, 0x00);

        // 8 bits       11
        // no parity    000
        // 1 stop bit   0
        // disable DLAB 0
        COM1.write(SerialRegister::LineControl, 0x03);

        // enable fifo
        // threshold of 14 bytes
        COM1.write(SerialRegister::FifoControl, 0xC7);

        // enable interrupts for;
        // data available    1
        // transmitter empty 1
        // not break/error   0
        // status change     1
        COM1.write(SerialRegister::ModemControl, 0x0B);

        // enable interrupts
        COM1.write(SerialRegister::InterruptEnable, 0x01);

        // ensure working
        // set in loopback mode, test the serial chip
        COM1.write(SerialRegister::ModemControl, 0x1E);

        // test serial chip (send byte 0xAE and check if serial returns same byte)
        COM1.write(SerialRegister::Data, 0xAE);

        // check if serial is faulty (i.e: not same byte as sent)
        let resp = COM1.recv();
        assert_eq!(resp, 0xAE); // invisible failure because we can't log anything at this point

        // if serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        COM1.write(SerialRegister::ModemControl, 0x0F);

        // init logger
        let logger = SERIAL_LOGGER.assume_init_mut();
        *logger = LockedSerialLogger(SpinLock::new(()), log_level, LogMode::SerialOnly);

        // safety: interrupts are disabled at this point, so can use racy variant
        use common::log;
        log::set_logger_racy(logger).unwrap();
        log::set_max_level(log_level);
    }
}

impl Log for LockedSerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // no locking needed
        metadata.level() <= self.1
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // we can ignore taking the serial port mutex if we're panicking and logging an error
            let _guard = {
                let first_attempt = self.0.try_lock();

                match first_attempt {
                    some @ Some(_) => {
                        // nice we got it first time
                        some
                    }
                    None => {
                        if is_panicking() {
                            // WE NEED THIS LOG, skip taking the lock
                            None
                        } else {
                            // block on taking it normally
                            Some(self.0.lock())
                        }
                    }
                }
            };

            // log to serial
            unsafe {
                COM1.write_fmt(format_args!(
                    "[{:.08} {} {}] {}\n",
                    clock::since_boot().as_secs_f64(),
                    {
                        let tgt = record.target();
                        let idx = tgt.find("::").map(|idx| idx + 2).unwrap_or(0);
                        &tgt[idx..]
                    },
                    record.level(),
                    record.args()
                ))
                .unwrap();
            }

            if matches!(self.2, LogMode::SerialAndVga) && vga::is_initialized() {
                let (fg, bg) = match record.level() {
                    Level::Error => (Color::White, Color::Red),
                    Level::Warn => (Color::Yellow, Color::Black),
                    Level::Info => (Color::Cyan, Color::Black),
                    Level::Debug => (Color::LightBlue, Color::Black),
                    Level::Trace => (Color::LightCyan, Color::Black),
                };
                // take the vga lock normally unless panicking, then skip
                let vga = match vga::try_get() {
                    None if !is_panicking() => Some(vga::get()),
                    other => other,
                };

                if let Some(mut vga) = vga {
                    vga.set_colors(fg, bg);
                    vga.write_fmt(format_args!("[{}] {}\n", record.level(), record.args()));
                }
            }
        }
    }

    fn flush(&self) {}
}

pub fn set_log_mode(mode: LogMode) {
    unsafe { SERIAL_LOGGER.assume_init_mut().2 = mode };
}
