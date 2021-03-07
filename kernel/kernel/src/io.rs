use core::ops::Add;

pub use msrs::*;

#[derive(Copy, Clone)]
pub struct Port(u16);

pub trait Msr: Sized {
    const MSR: u32;
    const NAME: &'static str;

    fn with_value(val: u64) -> Self;
    fn value(&self) -> u64;

    unsafe fn load() -> Self {
        let (hi, lo): (u32, u32);
        asm!("rdmsr", out("eax") lo, out("edx") hi, in("ecx") Self::MSR, options(nostack));
        Self::with_value(((hi as u64) << 32) | (lo as u64))
    }

    unsafe fn store(&self) {
        let value = self.value();
        common::trace!("setting MSR {} to {:#x}", Self::NAME, value);

        let lo = value as u32;
        let hi = (value >> 32) as u32;
        asm!("wrmsr", in("ecx") Self::MSR, in("eax") lo, in("edx") hi, options(nostack))
    }
}

impl Port {
    pub const fn new(port: u16) -> Self {
        Port(port)
    }

    pub unsafe fn write_u8(self, val: u8) {
        asm!("out dx, al", in("dx") self.0, in("al") val, options(nomem, nostack));
    }

    pub unsafe fn read_u8(self) -> u8 {
        let ret: u8;
        asm!("in al, dx", out("al") ret, in("dx") self.0, options(nomem, nostack));
        ret
    }
}

impl Add<u16> for Port {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

//
// pub const LSTAR: Self = Self(0xC0000082);
//
// /// The kernel's RIP for SYSCALL in compatibility mode.
// pub const CSTAR: Self = Self(0xC0000083);
//
// /// The low 32 bits are the SYSCALL flag mask. If a bit in this is set, the corresponding bit
// /// in rFLAGS is cleared
// pub const SFMASK: Self = Self(0xC0000083);

mod msrs {
    use crate::io::Msr;
    use modular_bitfield::prelude::*;

    /// The kernel's RIP SYSCALL entry for 64 bit software.
    #[derive(Copy, Clone)]
    pub struct LStar(u64);

    #[bitfield]
    #[derive(Copy, Clone)]
    pub struct Star {
        #[skip]
        eip_protected: B32,

        pub syscall: B16,
        pub sysret: B16,
    }

    /// Extended Feature Enable Register
    #[bitfield]
    #[derive(Copy, Clone)]
    pub struct Efer {
        pub syscall_extensions: bool,

        #[skip]
        _padding: B63,
    }

    impl Msr for Efer {
        const MSR: u32 = 0xC0000080;
        const NAME: &'static str = "EFER";

        fn with_value(val: u64) -> Self {
            Self::from_bytes(val.to_ne_bytes())
        }

        fn value(&self) -> u64 {
            u64::from_ne_bytes(self.into_bytes())
        }
    }

    impl Msr for LStar {
        const MSR: u32 = 0xC0000082;
        const NAME: &'static str = "LSTAR";

        fn with_value(val: u64) -> Self {
            Self(val)
        }

        fn value(&self) -> u64 {
            self.0
        }
    }

    impl Msr for Star {
        const MSR: u32 = 0xC0000081;
        const NAME: &'static str = "STAR";

        fn with_value(val: u64) -> Self {
            Self::from_bytes(val.to_ne_bytes())
        }

        fn value(&self) -> u64 {
            u64::from_ne_bytes(self.into_bytes())
        }
    }
}
