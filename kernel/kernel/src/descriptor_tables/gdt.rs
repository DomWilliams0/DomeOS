use crate::descriptor_tables::common::DescriptorTablePointer;
use common::*;
use modular_bitfield::prelude::*;

static mut GDT: InitializedGlobal<GlobalDescriptorTable> = InitializedGlobal::uninit();

pub struct GlobalDescriptorTable {
    entries: [u64; 6],
    next_available: u8,
}

#[bitfield]
#[derive(Copy, Clone)]
struct SegmentDescriptor {
    limit_0_15: B16,
    base_0_15: B16,
    base_16_23: B8,

    accessed: bool,
    writable: bool,
    conforming: bool,
    executable: bool,
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
    let cs = gdt.add_entry(0, SegmentDescriptor::kernel_code());
    let ds = gdt.add_entry(0, SegmentDescriptor::kernel_data());

    unsafe {
        gdt.load();
        cs.load_cs();
        ds.load_segments();
    }
}

impl Default for GlobalDescriptorTable {
    fn default() -> Self {
        Self {
            entries: [0; 6],
            next_available: 1, // skip null
        }
    }
}

impl GlobalDescriptorTable {
    fn next_free_index(&mut self) -> Option<usize> {
        let idx = self.next_available as usize;
        if idx < self.entries.len() {
            self.next_available += 1;
            Some(idx)
        } else {
            None
        }
    }

    fn add_entry(&mut self, rpl: u8, descriptor: SegmentDescriptor) -> SegmentSelector {
        let idx = self.next_free_index().expect("not enough GDT entries");
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

    unsafe fn load(self) {
        let count = self.next_available as usize;

        // replace global with self
        GDT.init(self);

        let table_ptr = DescriptorTablePointer {
            base: GDT.get() as *const _ as u64,
            limit: (count * core::mem::size_of::<u64>() - 1) as u16,
        };

        let ptr = &table_ptr as *const _;
        llvm_asm!("lgdt ($0)" :: "r" (ptr) : "memory");
    }
}

impl Default for SegmentDescriptor {
    /// Default bits set for flat segment
    fn default() -> Self {
        Self::new()
            .with_user_segment(true)
            .with_present(true)
            .with_writable(true)
            .with_accessed(true)
            .with_granularity(true)
            .with_limit_0_15(0xffff)
            .with_limit_16_19(0xf)
    }
}

impl SegmentDescriptor {
    fn kernel_code() -> Self {
        Self::default().with_executable(true).with_long_mode(true)
    }

    fn kernel_data() -> Self {
        Self::default().with_default_size(true)
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
}
