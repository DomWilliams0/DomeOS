use crate::interrupts::InterruptContext;
use crate::io::Port;
use common::*;
use modular_bitfield::prelude::*;

const CMD_AND_STATUS_PORT: Port = Port::new(0x64);
const DATA_PORT: Port = Port::new(0x60);

const TIMEOUT: u8 = 10;

pub struct Ps2Controller {}

#[derive(Display)]
pub enum Ps2Error {
    /// Controller failed test (returned {0:#x})
    ControllerFailedTest(u8),

    /// All ports failed test
    AllFailedTest,

    /// Timed out sending byte {byte:#x} to port {device:#x}
    SendTimeout { byte: u8, device: u8 },

    /// Timed out receiving from port {0:#x}
    RecvTimeout(u8),
}

#[derive(Copy, Clone)]
enum Command {
    /// Read byte n where 0x00 <= n <= 0x1f
    Read(u8),

    /// Write byte n where 0x00 <= n <= 0x1f. (n, byte to write)
    Write(u8, u8),

    ReadConfig,
    WriteConfig(ControllerConfig),

    WriteToSecond(u8),

    DisableFirst,
    EnableFirst,
    DisableSecond,
    EnableSecond,

    TestController,
    TestFirst,
    TestSecond,

    ResetFirst,
    ResetSecond,
}

#[bitfield(bits = 8)]
#[derive(Copy, Clone)]
struct Status {
    /// Must be SET before reading data from 0x60
    output_status: bool,

    /// Must be CLEAR before writing to 0x60 or 0x64
    input_status: bool,

    system: bool,

    /// * 0: input buffer is for device
    /// * 1: input buffer is for controller
    command_or_data: bool,

    unknown: B2,

    timeout_error: bool,
    parity_error: bool,
}

#[bitfield(bits = 8)]
#[derive(Copy, Clone)]
struct ControllerConfig {
    first_irq: bool,
    second_irq: bool,
    system: bool,

    reserved0: bool,

    first_clock: bool,
    second_clock: bool,
    first_port_translation: bool,

    reserved1: bool,
}

enum CommandByte {
    Single(u8),
    Double(u8, u8),
}

extern "C" fn on_key(_: *const InterruptContext) {
    debug!("NICE");
}

impl Ps2Controller {
    pub fn initialise() -> Result<Self, Ps2Error> {
        // TODO ensure usb is enabled and legacy usb mode disabled

        // disable
        Command::DisableFirst.issue()?;
        Command::DisableSecond.issue()?;

        // flush
        let _ = try_read_byte();

        // configure - disable irqs and translation
        let config = ControllerConfig::from_bytes([Command::ReadConfig.issue()?])
            .with_first_irq(false)
            .with_second_irq(false)
            .with_first_port_translation(false);

        Command::WriteConfig(config).issue()?;

        let mut has_second = config.second_clock();

        // test
        let controller_test = Command::TestController.issue()?;
        if controller_test != 0x55 {
            return Err(Ps2Error::ControllerFailedTest(controller_test));
        }

        // restore config in case the test reset it
        Command::WriteConfig(config).issue()?;

        // check if really dual channeled
        if has_second {
            Command::EnableSecond.issue()?;
            let config = ControllerConfig::from_bytes([Command::ReadConfig.issue()?]);
            if config.second_clock() {
                // nice, disable again
                Command::DisableSecond.issue()?;
            } else {
                // lol jk actually single channeled
                has_second = false;
            }
        }

        debug!(
            "PS/2 controller is {} channel",
            if has_second { "dual" } else { "single" }
        );

        // test devices
        let mut enabled = [true, true];
        enabled[0] = Command::TestFirst.issue()? == 0;
        enabled[1] = has_second && Command::TestSecond.issue()? == 0;

        // enable and reset devices
        let mut new_config = config;
        if enabled[0] {
            Command::EnableFirst.issue()?;
            // just send reset and swallow answer if any
            let _ = Command::ResetFirst.issue();
            new_config.set_first_irq(true);
        }

        if enabled[1] {
            Command::EnableSecond.issue()?;
            let _ = Command::ResetSecond.issue();
            new_config.set_second_irq(false);
        }

        // update config
        Command::WriteConfig(new_config).issue()?;

        // TODO register irq

        Ok(Ps2Controller {})
    }
}

impl Command {
    fn command_byte(self) -> CommandByte {
        use Command::*;
        use CommandByte::*;
        match self {
            Read(n) => {
                assert!(n <= 0x1f);
                Single(0x20 + n)
            }
            Write(n, byte) => {
                assert!(n <= 0x1f);
                Double(0x60 + n, byte)
            }
            ReadConfig => Single(0x20),
            WriteConfig(b) => Double(0x60, b.into_bytes()[0]),

            DisableFirst => Single(0xad),
            EnableFirst => Single(0xae),
            DisableSecond => Single(0xa7),
            EnableSecond => Single(0xa8),

            TestController => Single(0xaa),
            TestFirst => Single(0xab),
            TestSecond => Single(0xa9),

            WriteToSecond(u8) => Double(0xd4, u8),

            ResetFirst => Single(0xff),
            ResetSecond => Double(0xd4, 0xff),
        }
    }

    fn expects_response(self) -> bool {
        use Command::*;
        match self {
            Read(_) | ReadConfig | TestController | TestFirst | TestSecond | ResetFirst
            | ResetSecond => true,
            Write(_, _)
            | WriteConfig(_)
            | WriteToSecond(_)
            | DisableFirst
            | EnableFirst
            | DisableSecond
            | EnableSecond => false,
        }
    }

    /// Response is Ok(0) if not needed
    fn issue(self) -> Result<u8, Ps2Error> {
        // TODO add command context to any errors
        let cmd = self.command_byte();

        // write first byte
        let first = match cmd {
            CommandByte::Single(b) | CommandByte::Double(b, _) => b,
        };

        unsafe {
            CMD_AND_STATUS_PORT.write_u8(first);
        }

        if let CommandByte::Double(_, second) = cmd {
            // send second byte
            try_send_byte(second)?;
        }

        if self.expects_response() {
            // read response
            try_read_byte()
        } else {
            Ok(0)
        }
    }
}

impl Status {
    fn read() -> Self {
        let byte = unsafe { CMD_AND_STATUS_PORT.read_u8() };
        debug_assert_ne!(byte, 0);

        let status = Self::from_bytes([byte; 1]);
        debug_assert!(status.system());

        status
    }
}

#[inline]
fn try_send_byte(byte: u8) -> Result<(), Ps2Error> {
    for _ in 0..TIMEOUT {
        let status = Status::read();
        if !status.input_status() {
            unsafe { DATA_PORT.write_u8(byte) };
            return Ok(());
        }
    }

    Err(Ps2Error::SendTimeout {
        byte,
        device: DATA_PORT.id() as u8,
    })
}

#[inline]
fn try_read_byte() -> Result<u8, Ps2Error> {
    for _ in 0..TIMEOUT {
        let status = Status::read();
        if status.output_status() {
            return Ok(unsafe { DATA_PORT.read_u8() });
        }
    }
    Err(Ps2Error::RecvTimeout(DATA_PORT.id() as u8))
}

// TODO public api
fn send_data_to_first_device(byte: u8) -> Result<(), Ps2Error> {
    try_send_byte(byte)
}

fn send_data_to_second_device(byte: u8) -> Result<(), Ps2Error> {
    Command::WriteToSecond(byte).issue().map(|_| ())
}
