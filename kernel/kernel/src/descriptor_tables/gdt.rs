use crate::descriptor_tables::tss::TaskStateSegment;
use crate::descriptor_tables::DescriptorTablePointer;
use common::*;
use memory::PhysicalAddress;
use modular_bitfield::prelude::*;

static mut GDT: InitializedGlobal<GlobalDescriptorTable> = InitializedGlobal::uninit();

pub const SEL_KERNEL_CODE: u8 = 0x08;
pub const SEL_KERNEL_DATA: u8 = 0x10;
pub const SEL_USER_BASE: u8 = 0x1b;
pub const SEL_USER_DATA: u8 = 0x23;
pub const SEL_USER_CODE: u8 = 0x2b;

#[repr(C)]
pub struct GlobalDescriptorTable {
    entries: [u64; 8],
    next_available: u8,
}

#[bitfield]
#[derive(Copy, Clone)]
struct SegmentDescriptor {
    limit_0_15: B16,
    base_0_15: B16,
    base_16_23: B8,

    /// Type
    ty: B4,

    user_segment: bool,
    dpl: B2,
    present: bool,

    limit_16_19: B4,

    available: bool,
    long_mode: bool,
    default_size: bool,
    granularity: bool,

    base_24_31: B8,
}
#[bitfield]
#[derive(Copy, Clone)]
struct SegmentSelector {
    rpl: B3,
    idx: B5,
}

pub fn init() {
    let mut gdt = GlobalDescriptorTable::default();

    // 0 is already null
    let cs = gdt.add_entry(0, SegmentDescriptor::kernel_code());
    let ds = gdt.add_entry(0, SegmentDescriptor::kernel_data());

    let base_user = gdt.add_null_entry(3);
    let ds_user = gdt.add_entry(3, SegmentDescriptor::user_data());
    let cs_user = gdt.add_entry(3, SegmentDescriptor::user_code());

    let tss_addr = super::tss::TaskStateSegment::init();
    let tss = gdt.add_tss_entry(SegmentDescriptor::tss(tss_addr));

    debug_assert_eq!(cs.into_u8(), SEL_KERNEL_CODE, "kernel code");
    debug_assert_eq!(ds.into_u8(), SEL_KERNEL_DATA, "kernel data");

    debug_assert_eq!(base_user.into_u8(), SEL_USER_BASE, "user base");
    debug_assert_eq!(cs_user.into_u8(), SEL_USER_CODE, "user code");
    debug_assert_eq!(ds_user.into_u8(), SEL_USER_DATA, "user data");

    debug_assert_eq!(SEL_USER_BASE + 0x08, SEL_USER_DATA);
    debug_assert_eq!(SEL_USER_BASE + 0x10, SEL_USER_CODE);

    unsafe {
        gdt.load();
        cs.load_cs();
        ds.load_segments();
        tss.load_tss();
    }
}

impl Default for GlobalDescriptorTable {
    fn default() -> Self {
        Self {
            entries: [0; 8],
            next_available: 1, // skip null
        }
    }
}

impl GlobalDescriptorTable {
    fn next_free_index(&mut self, n: usize) -> Option<usize> {
        let idx = self.next_available as usize;
        if (idx + n - 1) < self.entries.len() {
            self.next_available += n as u8;
            Some(idx)
        } else {
            None
        }
    }

    fn add_entry(&mut self, rpl: u8, descriptor: SegmentDescriptor) -> SegmentSelector {
        let idx = self.next_free_index(1).expect("not enough GDT entries");
        self.entries[idx] = descriptor.into_u64();

        let selector = SegmentSelector::new().with_rpl(rpl).with_idx(idx as u8);
        trace!(
            "gdt[{} ({:#x})] = {:#x}",
            idx,
            selector.into_u8(),
            descriptor.into_u64()
        );
        selector
    }

    fn add_null_entry(&mut self, rpl: u8) -> SegmentSelector {
        let idx = self.next_free_index(1).expect("not enough GDT entries");
        self.entries[idx] = 0;

        let selector = SegmentSelector::new().with_rpl(rpl).with_idx(idx as u8);
        trace!("gdt[{} ({:#x})] = NULL", idx, selector.into_u8(),);
        selector
    }

    fn add_tss_entry(&mut self, (low, high): (u64, u64)) -> SegmentSelector {
        let idx = self.next_free_index(2).expect("not enough GDT entries");
        self.entries[idx] = low;
        self.entries[idx + 1] = high;

        let selector = SegmentSelector::new().with_rpl(0).with_idx(idx as u8);
        trace!(
            "gdt[{}:{} ({:#x})] = {:#x}",
            idx,
            idx + 1,
            selector.into_u8(),
            (low as u128) | ((high as u128) << 64)
        );
        selector
    }

    unsafe fn load(self) {
        let count = self.next_available as usize;

        // replace global with self
        GDT.init(self);

        let table_ptr = DescriptorTablePointer {
            base: GDT.get() as *mut _ as u64,
            limit: (count * core::mem::size_of::<u64>() - 1) as u16,
        };

        asm!("lgdt [{0}])", in(reg) &table_ptr, options(nostack));
    }
}

impl Default for SegmentDescriptor {
    /// Default bits set for flat segment
    fn default() -> Self {
        // accessed, writeable
        let ty = 0b0011;
        Self::new()
            .with_user_segment(true)
            .with_present(true)
            .with_ty(ty)
            .with_granularity(true)
            .with_limit_0_15(0xffff)
            .with_limit_16_19(0xf)
    }
}

impl SegmentDescriptor {
    fn kernel_code() -> Self {
        let ty = 0b1011; // accessed, writeable, executable
        Self::default().with_ty(ty).with_long_mode(true)
    }

    fn kernel_data() -> Self {
        Self::default().with_default_size(true)
    }

    fn user_code() -> Self {
        Self::kernel_code().with_dpl(3)
    }

    fn user_data() -> Self {
        Self::kernel_data().with_dpl(3)
    }

    /// (low, high)
    fn tss(tss: PhysicalAddress) -> (u64, u64) {
        let ptr = tss.address();

        let ty = 0b1001;
        let mut low = Self::new().with_present(true).with_ty(ty);

        use bit_field::BitField;
        low.set_base_0_15(ptr.get_bits(0..16) as u16);
        low.set_base_16_23(ptr.get_bits(16..24) as u8);
        low.set_base_24_31(ptr.get_bits(24..32) as u8);

        low.set_limit_0_15((core::mem::size_of::<TaskStateSegment>() - 1) as u16);

        let high = ptr.get_bits(32..64);

        (low.into_u64(), high)
    }

    fn into_u64(self) -> u64 {
        u64::from_le_bytes(self.into_bytes())
    }
}

impl SegmentSelector {
    fn into_u8(self) -> u8 {
        u8::from_le_bytes(self.into_bytes())
    }

    /// This segment must be a code segment
    unsafe fn load_cs(self) {
        let selector = self.into_u8();
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) selector as u64,
            tmp = lateout(reg) _,
        );
    }

    /// This segment must be a data segment
    unsafe fn load_segments(self) {
        let selector = self.into_u8();
        asm!(
            "mov ax, {sel:x}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
        sel = in(reg) selector as i16,
        )
    }

    /// This segment must be a TSS entry
    unsafe fn load_tss(self) {
        let selector = self.into_u8();
        asm!(
        "mov ax, {sel:x}",
        "ltr ax",
        sel = in(reg) selector as u16,
        )
    }
}
