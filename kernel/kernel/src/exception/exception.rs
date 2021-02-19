use core::convert::TryFrom;
use core::fmt::Debug;

use common::*;

use crate::exception::page_fault::PageFaultException;
use crate::irq::InterruptContext;
use enumflags2::BitFlags;
use memory::VirtualAddress;

#[derive(Display)]
pub enum ExceptionError {
    /// Unknown exception {0:#x}
    Unknown(u64),

    /// Exception {0:?} not implemented
    NotImplemented(&'static str),
}

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
pub enum Exception {
    DivideByZero,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRangeExceeded,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    CoprocessorSegmentOverrun,
    InvalidTss,
    SegmentNotPresent,
    StackSegmentFault,
    GeneralProtectionFault {
        rip: VirtualAddress,
        segment: Option<u64>,
    },
    PageFault(PageFaultException),
    x87FloatingPointException,
    AlignmentCheck,
    MachineCheck,
    SimdFloatingPointException,
    VirtualizationException,
    SecurityException,
    Reserved,
}

fn read_cr2() -> VirtualAddress {
    let value: u64;
    unsafe {
        llvm_asm!("mov %cr2, $0" : "=r" (value));
    }
    VirtualAddress(value)
}

impl TryFrom<&InterruptContext> for Exception {
    type Error = ExceptionError;

    fn try_from(value: &InterruptContext) -> Result<Self, Self::Error> {
        use Exception::*;
        use ExceptionError::*;
        Ok(match value.int_no {
            0 => DivideByZero,
            1 => Debug,
            2 => NonMaskableInterrupt,
            3 => Breakpoint,
            4 => Overflow,
            5 => BoundRangeExceeded,
            6 => InvalidOpcode,
            7 => DeviceNotAvailable,
            8 => DoubleFault,
            9 => CoprocessorSegmentOverrun,
            10 => return Err(NotImplemented("InvalidTss")),
            11 => return Err(NotImplemented("SegmentNotPresent")),
            12 => return Err(NotImplemented("StackSegmentFault")),
            13 => {
                let segment = if value.err_code == 0 {
                    None
                } else {
                    Some(value.err_code)
                };
                GeneralProtectionFault {
                    rip: VirtualAddress(value.rip),
                    segment,
                }
            }
            14 => {
                let bits: u8 = value.err_code.bit_range(5, 0);
                let flags = BitFlags::from_bits(bits)
                    .unwrap_or_else(|e| panic!("unexpected page fault flags: {}", e));
                let addr = read_cr2();
                PageFault(PageFaultException::new(flags, addr))
            }
            16 => x87FloatingPointException,
            17 => return Err(NotImplemented("AlignmentCheck")),
            18 => MachineCheck,
            19 => SimdFloatingPointException,
            20 => VirtualizationException,
            30 => return Err(NotImplemented("SecurityException")),
            15 | 21..=29 => Reserved,
            _ => return Err(ExceptionError::Unknown(value.int_no)),
        })
    }
}

impl Exception {
    pub fn handle(self, ctx: &InterruptContext) {
        use Exception::*;

        match self {
            PageFault(pf) => pf.handle(),
            _ => panic!("unhandled exception {:?}\n{:?}", self, ctx),
        }
    }
}
