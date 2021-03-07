use core::fmt::{Debug, Error, Formatter};

use common::*;

use crate::exception::Exception;
use crate::io::Port;
use core::convert::TryFrom;

const PIC_MASTER_COMMAND: Port = Port::new(0x20);
const PIC_MASTER_DATA: Port = Port::new(0x21);

const PIC_SLAVE_COMMAND: Port = Port::new(0xA0);
const PIC_SLAVE_DATA: Port = Port::new(0xA1);

const PIC_END_OF_INTERRUPT: u8 = 0x20;

/// Master PIC remapped offset
const PIC_MASTER_OFFSET: u8 = 0x20;

/// Slave PIC remapped offset
const PIC_SLAVE_OFFSET: u8 = PIC_MASTER_OFFSET + 8;

const IRQ_HANDLER_COUNT: usize = 16;

pub type IrqHandler = extern "C" fn(*const InterruptContext);

static mut IRQ_HANDLERS: [Option<IrqHandler>; IRQ_HANDLER_COUNT] = [None; IRQ_HANDLER_COUNT];

/// Flag for debug asserts that we're not currently in an interrupt/exception
#[cfg(debug_assertions)]
static mut HANDLING_INTERRUPT: bool = false;

/// Sets and clears global HANDLING_INTERRUPT on init/drop
struct InterruptGuard;

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum Irq {
    Clock = 0,
    Keyboard = 1,
    FloppyDisk = 6,
    Mouse = 12,
    Coprocessor = 13,
    PrimaryHardDisk = 14,
    SecondaryHardDisk = 15,
}

#[repr(C)]
pub struct InterruptContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,

    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    pub int_no: u64,
    pub err_code: u64,

    // pushed by CPU
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

/// Remap master PIC to 0x20 and slave to 0x28
pub unsafe fn remap() {
    // save masks
    let masks = (PIC_MASTER_DATA.read_u8(), PIC_SLAVE_DATA.read_u8());

    PIC_MASTER_COMMAND.write_u8(0x11);
    PIC_SLAVE_COMMAND.write_u8(0x11);

    PIC_MASTER_DATA.write_u8(PIC_MASTER_OFFSET);
    PIC_SLAVE_DATA.write_u8(PIC_SLAVE_OFFSET);

    PIC_MASTER_DATA.write_u8(0x04);
    PIC_SLAVE_DATA.write_u8(0x02);

    PIC_MASTER_DATA.write_u8(0x01);
    PIC_SLAVE_DATA.write_u8(0x01);

    // keyboard and clock enabled only for now, the rest are masked out
    PIC_MASTER_DATA.write_u8(0xFC);
    PIC_SLAVE_DATA.write_u8(0xFF);

    // restore masks
    PIC_MASTER_DATA.write_u8(masks.0);
    PIC_SLAVE_DATA.write_u8(masks.1);
}

pub fn register_handler(irq: Irq, handler: IrqHandler) {
    info!("registering IRQ {:?} handler: {:?}", irq, handler);
    unsafe {
        IRQ_HANDLERS[irq as usize] = Some(handler);
    }
}

pub fn unregister_handler(irq: Irq) {
    info!("unregistering IRQ {:?} handler", irq);
    unsafe {
        IRQ_HANDLERS[irq as usize] = None;
    }
}

#[cfg(debug_assertions)]
pub fn is_in_interrupt() -> bool {
    unsafe { HANDLING_INTERRUPT }
}

#[no_mangle]
pub extern "C" fn irq_handler(ctx: *const InterruptContext) {
    let _guard = InterruptGuard::init();

    let ctx: &InterruptContext = unsafe { &*ctx };
    let irq = (ctx.int_no - PIC_MASTER_OFFSET as u64) as usize; // remap to original irq
    debug_assert!(irq < IRQ_HANDLER_COUNT);

    // TODO spurious irqs

    // call handler
    unsafe {
        if let Some(handler) = IRQ_HANDLERS.get_unchecked(irq as usize) {
            handler(ctx);
        }

        // acknowledge
        eoi(irq);
    }
}

#[inline]
unsafe fn eoi(irq: usize) {
    if irq >= 8 {
        PIC_SLAVE_COMMAND.write_u8(PIC_END_OF_INTERRUPT);
    }

    PIC_MASTER_COMMAND.write_u8(PIC_END_OF_INTERRUPT);
}

#[no_mangle]
pub extern "C" fn fault_handler(ctx: *const InterruptContext) {
    let _guard = InterruptGuard::init();

    let ctx: &InterruptContext = unsafe { &*ctx };
    match Exception::try_from(ctx) {
        Ok(exc) => exc.handle(ctx),
        Err(err) => panic!("error handling exception: {}", err),
    }
}

impl InterruptGuard {
    fn init() -> Self {
        #[cfg(debug_assertions)]
        unsafe {
            HANDLING_INTERRUPT = true
        }
        Self
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe {
            HANDLING_INTERRUPT = false
        }
    }
}

pub fn enable_interrupts() {
    debug!("enabling interrupts");
    unsafe { asm!("sti") };
}

pub fn disable_interrupts() {
    debug!("disabling interrupts");
    unsafe { asm!("cli") };
}

impl Debug for InterruptContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // safety: packed field access
        write!(
            f,
            "\
             rax: {:#010x} rbx: {:#010x} rcx: {:#010x} rdx: {:#010x}\n\
             rsi: {:#010x} rdi: {:#010x} rbp: {:#010x} r8:  {:#010x}\n\
             r9:  {:#010x} r10: {:#010x} r11: {:#010x} r12: {:#010x}\n\
             r13: {:#010x} r14: {:#010x} r15: {:#010x} rip: {:#010x}\n\
             cs:  {:#010x} rsp: {:#010x} rflags: {:#010x} ss: {:#010x}\n\
             int_no: {:#x} err_code: {:#x}",
            self.rax,
            self.rbx,
            self.rcx,
            self.rdx,
            self.rsi,
            self.rdi,
            self.rbp,
            self.r8,
            self.r9,
            self.r10,
            self.r11,
            self.r12,
            self.r13,
            self.r14,
            self.r15,
            self.rip,
            self.cs,
            self.rsp,
            self.rflags,
            self.ss,
            self.int_no,
            self.err_code,
        )
    }
}
