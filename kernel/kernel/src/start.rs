use common::*;

use crate::cpu::CpuState;
use crate::descriptor_tables::{SEL_KERNEL_CODE, SEL_USER_CODE};
use crate::io::{Efer, GsBase, KernelGsBase, LStar, Msr, Star};
use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::memory::{KernelInterruptStacks, Stacks};
use crate::multiboot;
use crate::multiboot::Multiboot;
use crate::vga::{self, Color};
use crate::{clock, descriptor_tables, logging};
use memory::VirtualAddress;

// TODO guard page to detect and handle stack overflow
pub fn start(multiboot: &'static multiboot::multiboot_info) -> ! {
    vga::init(Color::LightGreen, Color::Black);
    logging::set_log_mode(LogMode::SerialAndVga);

    descriptor_tables::init();
    clock::init();

    let multiboot = Multiboot::new(multiboot);

    // init memory and get ourselves a heap
    if let Err(err) = crate::memory::init(multiboot) {
        error!("early memory setup failed: {}", err);
        hang();
    }

    // now we have a heap we can start using boxed error types

    // finally enable interrupts now that the higher half mappings are in place, so the isrs are
    // actually mapped
    enable_interrupts();

    // TODO 1 stack per core only, this needs to be shared
    let mut interrupt_stacks = Stacks::<KernelInterruptStacks>::new();
    let (interrupt_stack, _) = interrupt_stacks
        .new_stack()
        .expect("failed to map kernel interrupt stack");

    // prepare for syscalls, processes and userspace
    enable_syscalls();
    crate::process::init_kernel_process();

    // init per-cpu state
    let _cpu = init_cpu_state(interrupt_stack);

    // use this stack for interrupts on this cpu
    // TODO is tss shared or unique to cpu?
    crate::descriptor_tables::tss().set_privilege_stack(0, interrupt_stack);

    // begin testing
    let process = crate::process::experiment_new_process().expect("failed");
    debug!("process created");

    let thread = {
        let inner = process.inner_locked();
        let thread = inner.threads().next().expect("no main thread");
        thread.clone()
    };

    unsafe { thread.switch_to() }
}

fn hang() -> ! {
    disable_interrupts();

    info!("hanging forever");
    loop {
        unsafe { asm!("hlt") }
    }
}

fn enable_syscalls() {
    unsafe {
        // enable syscall/sysret instructions
        let mut efer = Efer::load();
        efer.set_syscall_extensions(true);
        efer.store();

        // register syscall entrypoint
        let lstar = LStar::with_value(crate::syscall::syscall_entry as *const () as u64);
        lstar.store();

        // register syscall segments
        let mut star = Star::load();
        star.set_sysret(SEL_USER_CODE as u16); // cs = this+16, SS.Sel = this+8
        star.set_syscall(SEL_KERNEL_CODE as u16);
        star.store();
    }
}

/// Allocates a new CpuState and writes its ptr to KernelGSbase
fn init_cpu_state(interrupt_stack_top: VirtualAddress) -> &'static mut CpuState {
    let state = crate::cpu::CpuState::new(interrupt_stack_top);

    unsafe {
        KernelGsBase::with_value(state as u64).store();
        GsBase::with_value(state as u64).store();

        &mut *state
    }
}
