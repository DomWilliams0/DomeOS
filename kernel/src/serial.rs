use core::fmt::{Error, Write};
use core::mem::MaybeUninit;
use core::ops::BitAnd;

use log::{Level, LevelFilter, Log, Metadata, Record};
use spin::Mutex;

use crate::clock;
use crate::io::Port;
use crate::panic::is_panicking;
use crate::serial::LogMode::SerialOnly;
use crate::vga;
use crate::vga::Color;

static mut COM1: SerialPort = SerialPort {
    port: Port(0x3F8),
    dlab: false,
};

struct SerialPort {
    port: Port,
    dlab: bool,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
enum SerialRegister {
    /// Int value is unused because of overlap
    Data = 100,
    /// Int value is unused because of overlap
    InterruptEnable = 101,
    /// Int value is unused because of overlap
    DivisorLsb = 200,
    /// Int value is unused because of overlap
    DivisorMsb = 201,

    FifoControl = 2,
    LineControl = 3,
    ModemControl = 4,
    LineStatus = 5,
    ModemStatus = 6,
    Scratch = 7,
}

static mut SERIAL_LOGGER: MaybeUninit<LockedSerialLogger> = MaybeUninit::uninit();

pub enum LogMode {
    SerialOnly,
    SerialAndVga,
}

struct SerialLogger {
    mode: LogMode,
}

struct LockedSerialLogger(Mutex<SerialLogger>, LevelFilter);

impl SerialPort {
    fn register(&self, register: SerialRegister) -> Port {
        let offset = match register {
            // enforce dlab
            SerialRegister::Data | SerialRegister::InterruptEnable if self.dlab => panic!(
                "DLAB must be unset to access serial register {:?}",
                register
            ),
            SerialRegister::DivisorLsb | SerialRegister::DivisorMsb if !self.dlab => {
                panic!("DLAB must be set to access serial register {:?}", register)
            }

            // return overlapping values
            SerialRegister::Data | SerialRegister::DivisorLsb => 0,
            SerialRegister::InterruptEnable | SerialRegister::DivisorMsb => 1,

            // others ignore dlab and are already their correct offsets
            r => r as u16,
        };
        self.port + offset
    }

    unsafe fn write(&mut self, register: SerialRegister, val: u8) {
        self.register(register).write_u8(val);

        if register == SerialRegister::LineControl {
            self.dlab = val.bitand(0x80) != 0;
        }
    }
    unsafe fn can_send(&self) -> bool {
        self.register(SerialRegister::LineStatus)
            .read_u8()
            .bitand(0x20)
            != 0
    }

    unsafe fn puts(&mut self, s: &str) {
        while !self.can_send() {}
        for c in s.bytes() {
            self.write(SerialRegister::Data, c);
        }
    }
}

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

        // init logger
        let logger = SERIAL_LOGGER.assume_init_mut();
        *logger = LockedSerialLogger(Mutex::new(SerialLogger { mode: SerialOnly }), log_level);

        // safety: interrupts are disabled at this point, so can use racy variant
        log::set_logger_racy(logger).unwrap();
        log::set_max_level(log_level);
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        unsafe {
            self.puts(s);
        }
        Ok(())
    }
}

impl Log for LockedSerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // no locking needed
        metadata.level() <= self.1
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() <= Level::Error && self.0.is_locked() && is_panicking() {
                // WE NEED THIS LOG
                unsafe { self.0.force_unlock() }
            }

            let logger = self.0.lock();

            // serial always
            unsafe {
                COM1.write_fmt(format_args!(
                    "[{:.08} {} {}] {}\n",
                    clock::since_boot().as_secs_f64(),
                    record.target(),
                    record.level(),
                    record.args()
                ))
                .unwrap();
            }

            // vga sometimes
            if matches!(logger.mode, LogMode::SerialAndVga) && vga::is_initialized() {
                let (fg, bg) = match record.level() {
                    Level::Error => (Color::White, Color::Red),
                    Level::Warn => (Color::Yellow, Color::Black),
                    Level::Info => (Color::Cyan, Color::Black),
                    Level::Debug => (Color::LightBlue, Color::Black),
                    Level::Trace => (Color::LightCyan, Color::Black),
                };
                let _colors = vga::set_colors(fg, bg);
                vga::_raw_print(format_args!("[{}] {}\n", record.level(), record.args()));
            }
        }
    }

    fn flush(&self) {}
}

pub fn set_log_mode(mode: LogMode) {
    let mut log = unsafe { SERIAL_LOGGER.assume_init_mut().0.lock() };

    log.mode = mode;
}
