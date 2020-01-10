use core::convert::TryFrom;
use core::fmt::{Debug, Error, Formatter};

use bitfield::BitRange;
use enumflags2::BitFlags;

use crate::irq::InterruptContext;

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
    GeneralProtectionFault,
    PageFault(PageFaultFlags, VirtualAddress),
    x87FloatingPointException,
    AlignmentCheck,
    MachineCheck,
    SimdFloatingPointException,
    VirtualizationException,
    SecurityException,
    Reserved,
}

// TODO move somewhere else
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "VirtualAddress({:#x})", self.0)
    }
}

#[derive(BitFlags, Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
enum PageFaultFlag {
    Present = 0b00001,
    Write = 0b00010,
    User = 0b00100,
    ReservedWrite = 0b01000,
    InstrFetch = 0b10000,
}

pub struct PageFaultFlags(BitFlags<PageFaultFlag>);

impl Debug for PageFaultFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "(")?;
        if self.0.contains(PageFaultFlag::Present) {
            write!(f, "PAGE_PROTECTION_VIOLATION")?;
        } else {
            write!(f, "NON_PRESENT_PAGE")?;
        }

        if self.0.contains(PageFaultFlag::Write) {
            write!(f, " | CAUSED_BY_WRITE")?;
        } else {
            write!(f, " | CAUSED_BY_READ")?;
        }

        if self.0.contains(PageFaultFlag::User) {
            write!(f, " | USER")?;
        }

        if self.0.contains(PageFaultFlag::ReservedWrite) {
            write!(f, " | RESERVED_WRITE")?;
        }

        if self.0.contains(PageFaultFlag::InstrFetch) {
            write!(f, " | INSTR_FETCH")?;
        }

        write!(f, ")")?;
        Ok(())
    }
}

fn read_cr2() -> VirtualAddress {
    let value: u64;
    unsafe {
        asm!("mov %cr2, $0" : "=r" (value));
    }
    VirtualAddress(value)
}

impl TryFrom<&InterruptContext> for Exception {
    type Error = u64;

    fn try_from(value: &InterruptContext) -> Result<Self, Self::Error> {
        Ok(match value.int_no {
            0 => Exception::DivideByZero,
            1 => Exception::Debug,
            2 => Exception::NonMaskableInterrupt,
            3 => Exception::Breakpoint,
            4 => Exception::Overflow,
            5 => Exception::BoundRangeExceeded,
            6 => Exception::InvalidOpcode,
            7 => Exception::DeviceNotAvailable,
            8 => Exception::DoubleFault,
            9 => Exception::CoprocessorSegmentOverrun,
            10 => unimplemented!("Exception::InvalidTss"),
            11 => unimplemented!("Exception::SegmentNotPresent"),
            12 => unimplemented!("Exception::StackSegmentFault"),
            13 => unimplemented!("Exception::GeneralProtectionFault"),
            14 => {
                let bits: u8 = value.err_code.bit_range(5, 0);
                let flags = BitFlags::from_bits(bits).unwrap();
                let addr = read_cr2();
                Exception::PageFault(PageFaultFlags(flags), addr)
            }
            16 => Exception::x87FloatingPointException,
            17 => unimplemented!("Exception::AlignmentCheck"),
            18 => Exception::MachineCheck,
            19 => Exception::SimdFloatingPointException,
            20 => Exception::VirtualizationException,
            30 => unimplemented!("Exception::SecurityException"),
            15 | 21..=29 => Exception::Reserved,
            _ => return Err(value.int_no),
        })
    }
}
