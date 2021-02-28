use core::mem::MaybeUninit;
use core::ops::{Shl, Shr};

use modular_bitfield::prelude::*;

use crate::descriptor_tables::common::DescriptorTablePointer;
use crate::descriptor_tables::tss::IST_IDX_DOUBLE_FAULT;
use crate::irq;
use memory::VIRT_KERNEL_BASE;

static mut IDT: MaybeUninit<InterruptDescriptorTable> = MaybeUninit::uninit();

// TODO this only needs to be temporary?
#[export_name = "idt_descriptor"]
pub(crate) static mut IDT_POINTER: MaybeUninit<DescriptorTablePointer> = MaybeUninit::uninit();

const IDT_ENTRY_COUNT: usize = 256;

#[bitfield]
#[derive(Copy, Clone)]
struct IdtEntry {
    /// Bottom 16 bits of offset
    base_low: B16,

    // Selector
    /// Requested privilege level
    rpl: B2,

    /// Table index
    /// Gdt: 0
    /// Ldt: 1
    ti: B1,

    /// Descriptor index in selected table
    descriptor_index: B13,

    /// Interrupt Stack Table offset
    ist: B3,

    /// If you say so
    reserved0: B5,

    // Flags
    /// 1110 for 32 bit interrupt gates
    gate_type: B4,

    /// Zero for interrupt gates
    storage_segment: bool,

    /// Ring 0 - 3
    ring: B2,

    /// Present/used
    present: bool,

    /// Middle 16 bits of base
    base_mid: B16,

    /// Upper 32 bits
    base_high: B32,

    /// More reserved
    reserved1: B32,
}

#[repr(transparent)]
struct InterruptDescriptorTable {
    entries: [IdtEntry; IDT_ENTRY_COUNT],
}

type InterruptHandler = unsafe extern "C" fn();

impl IdtEntry {
    fn with_handler(handler: *const InterruptHandler) -> Self {
        let mut e = IdtEntry::default();

        let addr: u64 = handler as *const _ as u64;
        e.set_addr(addr);
        e.set_present(true);
        e
    }

    fn set_addr(&mut self, addr: u64) {
        self.set_base_low(addr as u16);
        self.set_base_mid(addr.shr(16) as u16);
        self.set_base_high(addr.shr(32) as u32);
    }

    fn addr(&self) -> u64 {
        let low = self.base_low() as u64;
        let mid = self.base_mid() as u64;
        let high = self.base_high() as u64;

        low | mid.shl(16) | high.shl(32)
    }

    fn set_ist_index(&mut self, idx: Option<usize>) {
        let value = match idx {
            Some(idx) => {
                assert!(idx < 7);
                (idx + 1) as u8
            }
            None => 0,
        };

        self.set_ist(value);
    }
}

impl Default for IdtEntry {
    /// Common entry
    fn default() -> Self {
        let mut entry = IdtEntry::new(); // zeroed

        entry.set_rpl(0); // ring 0
        entry.set_ring(0);
        entry.set_ti(0); // gdt
        entry.set_descriptor_index(1); // code segment
        entry.set_gate_type(0xE); //3 2 bit interrupt gate
        entry.set_storage_segment(false); // interrupt gate
        entry.set_present(false); // not present until registered

        entry
    }
}

impl Default for InterruptDescriptorTable {
    fn default() -> Self {
        let mut table = InterruptDescriptorTable {
            entries: [IdtEntry::default(); IDT_ENTRY_COUNT],
        };

        table.register(0, externs::isr0);
        table.register(1, externs::isr1);
        table.register(2, externs::isr2);
        table.register(3, externs::isr3);
        table.register(4, externs::isr4);
        table.register(5, externs::isr5);
        table.register(6, externs::isr6);
        table.register(7, externs::isr7);
        table
            .register(8, externs::isr8)
            .set_ist_index(Some(IST_IDX_DOUBLE_FAULT));
        table.register(9, externs::isr9);
        table.register(10, externs::isr10);
        table.register(11, externs::isr11);
        table.register(12, externs::isr12);
        table.register(13, externs::isr13);
        table.register(14, externs::isr14);
        table.register(15, externs::isr15);
        table.register(16, externs::isr16);
        table.register(17, externs::isr17);
        table.register(18, externs::isr18);
        table.register(19, externs::isr19);
        table.register(20, externs::isr20);
        table.register(21, externs::isr21);
        table.register(22, externs::isr22);
        table.register(23, externs::isr23);
        table.register(24, externs::isr24);
        table.register(25, externs::isr25);
        table.register(26, externs::isr26);
        table.register(27, externs::isr27);
        table.register(28, externs::isr28);
        table.register(29, externs::isr29);
        table.register(30, externs::isr30);
        table.register(31, externs::isr31);

        table.register(32, externs::irq0);
        table.register(33, externs::irq1);
        table.register(34, externs::irq2);
        table.register(35, externs::irq3);
        table.register(36, externs::irq4);
        table.register(37, externs::irq5);
        table.register(38, externs::irq6);
        table.register(39, externs::irq7);
        table.register(40, externs::irq8);
        table.register(41, externs::irq9);
        table.register(42, externs::irq10);
        table.register(43, externs::irq11);
        table.register(44, externs::irq12);
        table.register(45, externs::irq13);
        table.register(46, externs::irq14);
        table.register(47, externs::irq15);

        table
    }
}

impl InterruptDescriptorTable {
    /// Handler must be in the higher half mapped range, i.e. above VIRT_KERNEL_BASE
    fn register(&mut self, index: usize, handler: InterruptHandler) -> &mut IdtEntry {
        let ptr = handler as *const InterruptHandler;
        debug_assert!(index < IDT_ENTRY_COUNT);
        debug_assert!(ptr as u64 > VIRT_KERNEL_BASE);

        let entry = &mut self.entries[index];
        *entry = IdtEntry::with_handler(ptr);
        entry
    }

    unsafe fn load(self) {
        // replace global IDT with self
        IDT.as_mut_ptr().write(self);

        // point at global IDT
        IDT_POINTER.as_mut_ptr().write(DescriptorTablePointer {
            base: IDT.as_ptr() as u64,
            limit: core::mem::size_of::<InterruptDescriptorTable>() as u16,
        });

        let pointer = IDT_POINTER.as_ptr();
        llvm_asm!("lidt ($0)" :: "r" (pointer) : "memory");
    }
}

pub fn init() {
    unsafe {
        irq::remap();

        let idt = InterruptDescriptorTable::default();
        idt.load()
    }
}

mod externs {
    extern "C" {
        pub fn isr0();
        pub fn isr1();
        pub fn isr2();
        pub fn isr3();
        pub fn isr4();
        pub fn isr5();
        pub fn isr6();
        pub fn isr7();
        pub fn isr8();
        pub fn isr9();
        pub fn isr10();
        pub fn isr11();
        pub fn isr12();
        pub fn isr13();
        pub fn isr14();
        pub fn isr15();
        pub fn isr16();
        pub fn isr17();
        pub fn isr18();
        pub fn isr19();
        pub fn isr20();
        pub fn isr21();
        pub fn isr22();
        pub fn isr23();
        pub fn isr24();
        pub fn isr25();
        pub fn isr26();
        pub fn isr27();
        pub fn isr28();
        pub fn isr29();
        pub fn isr30();
        pub fn isr31();

        pub fn irq0();
        pub fn irq1();
        pub fn irq2();
        pub fn irq3();
        pub fn irq4();
        pub fn irq5();
        pub fn irq6();
        pub fn irq7();
        pub fn irq8();
        pub fn irq9();
        pub fn irq10();
        pub fn irq11();
        pub fn irq12();
        pub fn irq13();
        pub fn irq14();
        pub fn irq15();
    }
}
