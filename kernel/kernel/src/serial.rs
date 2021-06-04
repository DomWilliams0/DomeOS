use core::fmt::{Error, Write};
use core::ops::BitAnd;

use crate::io::Port;

pub struct SerialPort {
    port: Port,
    dlab: bool,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum SerialRegister {
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

impl SerialPort {
    pub const fn new(port: Port, dlab: bool) -> Self {
        Self { port, dlab }
    }

    pub fn register(&self, register: SerialRegister) -> Port {
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

    pub unsafe fn write(&mut self, register: SerialRegister, val: u8) {
        self.register(register).write_u8(val);

        if register == SerialRegister::LineControl {
            self.dlab = val.bitand(0x80) != 0;
        }
    }

    pub unsafe fn recv(&mut self) -> u8 {
        while !self.can_recv() {}
        self.register(SerialRegister::Data).read_u8()
    }

    unsafe fn can_send(&self) -> bool {
        self.register(SerialRegister::LineStatus)
            .read_u8()
            .bitand(0x20)
            != 0
    }

    unsafe fn can_recv(&self) -> bool {
        self.register(SerialRegister::LineStatus)
            .read_u8()
            .bitand(0x1)
            != 0
    }

    unsafe fn puts(&mut self, s: &str) {
        for c in s.bytes() {
            while !self.can_send() {}
            self.write(SerialRegister::Data, c);
        }
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
