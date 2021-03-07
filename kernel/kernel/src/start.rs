use common::*;

use crate::descriptor_tables::{SEL_KERNEL_CODE, SEL_USER_CODE};
use crate::io::{Efer, LStar, Msr, Star};
use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::memory::{KernelInterruptStacks, Stacks};
use crate::multiboot;
use crate::multiboot::Multiboot;
use crate::process::ThreadRef;
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

    // prepare for syscalls, processes and userspace
    enable_syscalls();
    crate::process::init_kernel_process();

    // TODO 1 stack per core only
    let mut interrupt_stacks = Stacks::<KernelInterruptStacks>::new();
    let (_interrupt_stack, _) = interrupt_stacks
        .new_stack()
        .expect("failed to map kernel interrupt stack");
    // TODO register stack for use by interrupts

    // begin testing
    let process = crate::process::experiment_new_process().expect("failed");
    debug!("process created");

    let _thread: ThreadRef = {
        let inner = process.inner_locked();
        let thread = inner.threads().next().expect("no main thread");
        thread.clone()
    };

    // use this stack for interrupts after jumping to userspace
    {
        let rsp: u64;
        unsafe {
            asm!("mov {0}, rsp", out(reg) rsp);
        }
        crate::descriptor_tables::tss().set_privilege_stack(0, VirtualAddress(rsp));
    }

    // unsafe { thread.run_now() }
    hang()
}

fn breakpoint() {
    unsafe { asm!("int3") }
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
