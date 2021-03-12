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
        common::trace!("{} = {:#x}", Self::NAME, value);

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

    pub const fn id(&self) -> u16 {
        self.0
    }
}

impl Add<u16> for Port {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self(self.0 + rhs)
    }
}

mod msrs {
    use crate::io::Msr;
    use modular_bitfield::prelude::*;

    /// The kernel's RIP SYSCALL entry for 64 bit software.
    #[derive(Copy, Clone)]
    pub struct LStar(u64);

    /// Ring 0 and Ring 3 Segment bases, as well as SYSCALL EIP.
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

    /// This register is used by the SWAPGS instruction. This instruction
    /// exchanges the value located in KernelGSbase with the value located in GS.base.
    #[derive(Copy, Clone)]
    pub struct KernelGsBase(u64);

    #[derive(Copy, Clone)]
    pub struct GsBase(u64);

    impl Msr for Efer {
        const MSR: u32 = 0xC000_0080;
        const NAME: &'static str = "EFER";

        fn with_value(val: u64) -> Self {
            Self::from_bytes(val.to_ne_bytes())
        }

        fn value(&self) -> u64 {
            u64::from_ne_bytes(self.into_bytes())
        }
    }

    impl Msr for LStar {
        const MSR: u32 = 0xC000_0082;
        const NAME: &'static str = "LSTAR";

        fn with_value(val: u64) -> Self {
            Self(val)
        }

        fn value(&self) -> u64 {
            self.0
        }
    }

    impl Msr for Star {
        const MSR: u32 = 0xC000_0081;
        const NAME: &'static str = "STAR";

        fn with_value(val: u64) -> Self {
            Self::from_bytes(val.to_ne_bytes())
        }

        fn value(&self) -> u64 {
            u64::from_ne_bytes(self.into_bytes())
        }
    }

    impl Msr for KernelGsBase {
        const MSR: u32 = 0xC000_0102;
        const NAME: &'static str = "KernelGSbase";

        fn with_value(val: u64) -> Self {
            Self(val)
        }

        fn value(&self) -> u64 {
            self.0
        }
    }

    impl Msr for GsBase {
        const MSR: u32 = 0xC000_0101;
        const NAME: &'static str = "GS.Base";

        fn with_value(val: u64) -> Self {
            Self(val)
        }

        fn value(&self) -> u64 {
            self.0
        }

        // unsafe fn load() -> Self {
        //     let val: u64;
        //     asm!("rdgsbase rax", out("rax") val,  options(nostack));
        //     Self(val)
        // }
        //
        // unsafe fn store(&self) {
        //     common::trace!("{} = {:#x}", Self::NAME, self.0);
        //     asm!("wrgsbase rax", in("rax") self.0,  options(nostack));
        // }
    }
}
