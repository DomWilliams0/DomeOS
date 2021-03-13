use crate::cpuid::CpuFeature;
use crate::io::{ApicBase, Msr};
use crate::memory::{frame_allocator, AddressSpace, FrameAllocator};
use common::*;
use memory::{
    MapFlags, MapTarget, MemoryError, PhysicalAddress, VirtualAddress, FRAME_SIZE,
    VIRT_IO_UNCACHEABLE_BASE, VIRT_IO_UNCACHEABLE_MAX,
};

#[derive(Display)]
pub enum ApicError {
    /// CPU does not have a local APIC
    ApicNotPresent,

    /// DMA allocation error: {0}
    AllocationDma(MemoryError),
}

#[derive(Copy, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
enum ApicRegister {
    /// APIC ID Register
    ApicId = 0x20,
    /// APIC Version Register
    ApicVersion = 0x30,
    /// Task Priority Register (TPR)
    Tpr = 0x80,
    /// Arbitration Priority Register (APR)
    Apr = 0x90,
    /// Processor Priority Register (PPR)
    Ppr = 0xA0,
    /// End of Interrupt Register (EOI)
    Eoi = 0xB0,
    /// Remote Read Register
    RemoteRead = 0xC0,
    /// Logical Destination Register (LDR)
    Ldr = 0xD0,
    /// Destination Format Register (DFR)
    Dfr = 0xE0,
    /// Spurious Interrupt Vector Register
    SpuriousIv = 0xF0,
    /// In-Service Register (ISR)
    Isr1 = 0x100,
    /// In-Service Register (ISR)
    Isr2 = 0x110,
    /// In-Service Register (ISR)
    Isr3 = 0x120,
    /// In-Service Register (ISR)
    Isr4 = 0x130,
    /// In-Service Register (ISR)
    Isr5 = 0x140,
    /// In-Service Register (ISR)
    Isr6 = 0x150,
    /// In-Service Register (ISR)
    Isr7 = 0x160,
    /// In-Service Register (ISR)
    Isr8 = 0x170,
    /// Trigger Mode Register (TMR)
    Tmr1 = 0x180,
    /// Trigger Mode Register (TMR)
    Tmr2 = 0x190,
    /// Trigger Mode Register (TMR)
    Tmr3 = 0x1a0,
    /// Trigger Mode Register (TMR)
    Tmr4 = 0x1b0,
    /// Trigger Mode Register (TMR)
    Tmr5 = 0x1c0,
    /// Trigger Mode Register (TMR)
    Tmr6 = 0x1d0,
    /// Trigger Mode Register (TMR)
    Tmr7 = 0x1e0,
    /// Trigger Mode Register (TMR)
    Tmr8 = 0x1f0,
    /// Interrupt Request Register (IRR)
    Irr1 = 0x200,
    /// Interrupt Request Register (IRR)
    Irr2 = 0x210,
    /// Interrupt Request Register (IRR)
    Irr3 = 0x220,
    /// Interrupt Request Register (IRR)
    Irr4 = 0x230,
    /// Interrupt Request Register (IRR)
    Irr5 = 0x240,
    /// Interrupt Request Register (IRR)
    Irr6 = 0x250,
    /// Interrupt Request Register (IRR)
    Irr7 = 0x260,
    /// Interrupt Request Register (IRR)
    Irr8 = 0x270,
    /// Error Status Register (ESR)
    Esr = 0x280,
    /// Interrupt Command Register Low (bits 31:0)
    IcrLo = 0x300,
    /// Interrupt Command Register High (bits 63:32)
    IcrHi = 0x310,
    /// Timer Local Vector Table Entry
    Timer = 0x320,
    /// Thermal Local Vector Table Entry
    Thermal = 0x330,
    /// Performance Counter Local Vector Table Entry
    PerfCounter = 0x340,
    /// Local Interrupt 0 Vector Table Entry
    Lint0 = 0x350,
    /// Local Interrupt 1 Vector Table Entry
    Lint1 = 0x360,
    /// Error Vector Table Entry
    Error = 0x370,
    /// Timer Initial Count Register
    TimerInitialCount = 0x380,
    /// Timer Current Count Register
    TimerCurrentCount = 0x390,
    /// Timer Divide Configuration Register
    TimerDivideCfg = 0x3E0,
    // TODO extended AMD registers
}

pub struct Apic {
    base_phys: PhysicalAddress,
    base_virt: VirtualAddress,
}

/// Interrupts should be disabled still until after this returns
pub fn init() -> Result<Apic, ApicError> {
    // ensure apic is present
    let cpu_features = CpuFeature::fetch();
    if !cpu_features.contains(CpuFeature::Apic) {
        return Err(ApicError::ApicNotPresent);
    }

    // TODO use MADT in APCI tables if available

    // allocate and map DMA page for APIC registers
    // TODO doesn't work in qemu but does on bochs and real hardware
    let allocate_apic_registers = || -> Result<_, MemoryError> {
        let apic_frame_bomb = frame_allocator().as_bomb(FrameAllocator::allocate_low)?;

        let mut addr_space = AddressSpace::current();
        let base = addr_space
            .find_free_space(VirtualAddress::with_literal(VIRT_IO_UNCACHEABLE_BASE), 1)
            .and_then(|base| {
                if (base + FRAME_SIZE).address() >= VIRT_IO_UNCACHEABLE_MAX {
                    Err(MemoryError::NoMoreIo)
                } else {
                    Ok(base)
                }
            })?;

        addr_space.map_range(
            base,
            FRAME_SIZE,
            MapTarget::Specific(apic_frame_bomb.frame()),
            MapFlags::CacheDisable | MapFlags::Writeable | MapFlags::Commit | MapFlags::Io,
        )?;

        Ok((apic_frame_bomb.defuse(), base))
    };

    let (apic_registers_frame, apic_registers_base) =
        allocate_apic_registers().map_err(ApicError::AllocationDma)?;

    debug!("APIC base address = {:?}", apic_registers_frame.address());

    // relocate
    unsafe {
        let mut msr = ApicBase::load();
        msr.set_enabled(true);
        msr.set_base_address(apic_registers_frame.address());
        debug_assert!(msr.bsc());
        msr.store();
    }

    let apic = Apic {
        base_phys: apic_registers_frame.address(),
        base_virt: apic_registers_base,
    };

    // enable LAPIC
    //  * spurious vector = 0
    //  * software enable = 1
    //  * focus cpu core checking = 0
    apic.write(ApicRegister::SpuriousIv, 1 << 8);

    trace!("APIC id={:#x}", apic.read(ApicRegister::ApicId));
    trace!("APIC vers={:#x}", apic.read(ApicRegister::ApicVersion));

    Ok(apic)
}

impl Apic {
    unsafe fn reg_ptr(&self, reg: ApicRegister) -> *mut u32 {
        debug_assert_eq!(
            self.base_phys,
            ApicBase::load().base_address(),
            "APIC base is wrong"
        );

        let base = self.base_virt;
        let offset = reg as usize;

        base.as_ptr::<u8>().add(offset) as *mut u32
    }

    fn read(&self, reg: ApicRegister) -> u32 {
        unsafe { self.reg_ptr(reg).read_volatile() }
    }

    fn write(&self, reg: ApicRegister, val: u32) {
        unsafe { self.reg_ptr(reg).write_volatile(val) }
    }

    /// TPR value (AMD64 Volume 2 16.6.4). Panics if either are >= 16.
    /// 1 = lowest priority, 15 = highest. 0 allows all
    pub fn set_threshold(&self, task_priority: u8, task_priority_subclass: u8) {
        assert!(task_priority < 16 && task_priority_subclass < 16);
        let value = (task_priority << 4) | task_priority_subclass; // mask not required with assert
        self.write(ApicRegister::Tpr, value as u32);
    }
}
