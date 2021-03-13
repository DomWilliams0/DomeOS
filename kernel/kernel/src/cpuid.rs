use enumflags2::*;

#[derive(BitFlags, Copy, Clone, Debug)]
#[repr(u64)]
pub enum CpuFeature {
    // begin ecx
    Sse3 = 1 << 0,
    Pclmul = 1 << 1,
    Dtes64 = 1 << 2,
    Monitor = 1 << 3,
    DsCpl = 1 << 4,
    Vmx = 1 << 5,
    Smx = 1 << 6,
    Est = 1 << 7,
    Tm2 = 1 << 8,
    Ssse3 = 1 << 9,
    Cid = 1 << 10,
    Fma = 1 << 12,
    Cx16 = 1 << 13,
    Etprd = 1 << 14,
    Pdcm = 1 << 15,
    Pcide = 1 << 17,
    Dca = 1 << 18,
    Sse4_1 = 1 << 19,
    Sse4_2 = 1 << 20,
    X2apic = 1 << 21,
    Movbe = 1 << 22,
    Popcnt = 1 << 23,
    Aes = 1 << 25,
    Xsave = 1 << 26,
    Osxsave = 1 << 27,
    Avx = 1 << 28,

    // begin edx << 32
    Fpu = 1 << 32,
    Vme = 1 << 33,
    De = 1 << 34,
    Pse = 1 << 35,
    Tsc = 1 << 36,
    Msr = 1 << 37,
    Pae = 1 << 38,
    Mce = 1 << 39,
    Cx8 = 1 << 40,
    Apic = 1 << 41,
    Sep = 1 << 43,
    Mtrr = 1 << 44,
    Pge = 1 << 45,
    Mca = 1 << 46,
    Cmov = 1 << 47,
    Pat = 1 << 48,
    Pse36 = 1 << 49,
    Psn = 1 << 50,
    Clf = 1 << 51,
    Dtes = 1 << 53,
    Acpi = 1 << 54,
    Mmx = 1 << 55,
    Fxsr = 1 << 56,
    Sse = 1 << 57,
    Sse2 = 1 << 58,
    Ss = 1 << 59,
    Htt = 1 << 60,
    Tm1 = 1 << 61,
    Ia64 = 1 << 62,
    Pbe = 1 << 63,
}

#[derive(BitFlags, Copy, Clone)]
#[repr(u32)]
enum CpuidFeatureEcx {
    Sse3 = 1 << 0,
    Pclmul = 1 << 1,
    Dtes64 = 1 << 2,
    Monitor = 1 << 3,
    DsCpl = 1 << 4,
    Vmx = 1 << 5,
    Smx = 1 << 6,
    Est = 1 << 7,
    Tm2 = 1 << 8,
    Ssse3 = 1 << 9,
    Cid = 1 << 10,
    Fma = 1 << 12,
    Cx16 = 1 << 13,
    Etprd = 1 << 14,
    Pdcm = 1 << 15,
    Pcide = 1 << 17,
    Dca = 1 << 18,
    Sse4_1 = 1 << 19,
    Sse4_2 = 1 << 20,
    X2apic = 1 << 21,
    Movbe = 1 << 22,
    Popcnt = 1 << 23,
    Aes = 1 << 25,
    Xsave = 1 << 26,
    Osxsave = 1 << 27,
    Avx = 1 << 28,
}

#[derive(BitFlags, Copy, Clone)]
#[repr(u32)]
enum CpuidFeatureEdx {
    Fpu = 1 << 0,
    Vme = 1 << 1,
    De = 1 << 2,
    Pse = 1 << 3,
    Tsc = 1 << 4,
    Msr = 1 << 5,
    Pae = 1 << 6,
    Mce = 1 << 7,
    Cx8 = 1 << 8,
    Apic = 1 << 9,
    Sep = 1 << 11,
    Mtrr = 1 << 12,
    Pge = 1 << 13,
    Mca = 1 << 14,
    Cmov = 1 << 15,
    Pat = 1 << 16,
    Pse36 = 1 << 17,
    Psn = 1 << 18,
    Clf = 1 << 19,
    Dtes = 1 << 21,
    Acpi = 1 << 22,
    Mmx = 1 << 23,
    Fxsr = 1 << 24,
    Sse = 1 << 25,
    Sse2 = 1 << 26,
    Ss = 1 << 27,
    Htt = 1 << 28,
    Tm1 = 1 << 29,
    Ia64 = 1 << 30,
    Pbe = 1 << 31,
}

impl CpuFeature {
    pub fn fetch() -> BitFlags<Self> {
        let ecx: u32;
        let edx: u32;
        unsafe { asm!("cpuid", in("eax") 1, out("ecx") ecx, out("edx") edx) }

        let flags_ecx = BitFlags::<CpuidFeatureEcx>::from_bits_truncate(ecx);
        let flags_edx = BitFlags::<CpuidFeatureEdx>::from_bits_truncate(edx);

        let combined = ((flags_edx.bits() as u64) << 32) | (flags_ecx.bits() as u64);
        BitFlags::from_bits_truncate(combined)
    }
}
