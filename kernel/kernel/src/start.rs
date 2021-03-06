use common::*;

use crate::descriptor_tables::{SEL_USER_CODE, SEL_USER_DATA};
use crate::irq::{disable_interrupts, enable_interrupts};
use crate::logging::LogMode;
use crate::memory::AddressSpace;
use crate::multiboot;
use crate::multiboot::Multiboot;
use crate::process::ThreadRef;
use crate::vga::{self, Color};
use crate::{clock, descriptor_tables, logging};
use memory::{kilobytes, megabytes, MapFlags, MapTarget, VirtualAddress, FRAME_SIZE};

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

    let process = crate::process::experiment_new_process().expect("failed");
    debug!("process created");

    let thread: ThreadRef = {
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

    unsafe { thread.run_now() }
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

#[deprecated]
fn experiment_userspace() {
    let mut addr = AddressSpace::new().expect("damn");

    // map some user space code
    let mut code = addr
        .map_range(
            VirtualAddress::with_literal(0x6666_0000),
            0x1000,
            MapTarget::Any,
            MapFlags::Writeable | MapFlags::Executable | MapFlags::User,
        )
        .expect("damn");

    // switch to user address space
    unsafe {
        addr.load_unconditionally();
    }

    // write simple shellcode
    let code_exe = code.write().unwrap();
    code_exe.fill(0x90); // nops
                         // code_exe[2] = 0xfa; // cli
    code_exe[10] = 0xeb; // jmp to start of nops
    code_exe[11] = 0xf4;

    // use this stack for interrupts
    crate::descriptor_tables::tss().set_privilege_stack(0, rsp());

    let entry_point = code.address();

    trace!("diving into usermode code at {:?}", entry_point);
    unsafe {
        asm!(
        "cli", // no more interrupts
        "mov ax, {ds_user}", // user data segment
        "mov ds,ax",
        "mov es,ax",
        "mov fs,ax",
        "mov gs,ax", // ss handled by iret

        "mov rax, rsp",
        "push {ds_user}", // user data selector
        "push rax", // user rsp
        "pushf", // eflags

        "push {cs_user}", // user code selector
        "push {entry}", // user rip

        "iretq", // and awaay we go
        ds_user = const SEL_USER_DATA,
        cs_user = const SEL_USER_CODE,
        entry = in(reg) entry_point.address(),
        out("rax") _,
        )
    };
}

fn rsp() -> VirtualAddress {
    let rsp: u64;
    unsafe {
        asm!("mov {0}, rsp", out(reg) rsp);
    }
    VirtualAddress(rsp)
}

fn run_on_new_stack(run_me: fn() -> !) -> ! {
    // TODO move to constants
    const KERNEL_STACKS_START: u64 = 0xffff_8000_0000_0000;
    const KERNEL_STACK_MAX_SIZE: u64 = megabytes(8);
    const KERNEL_STACK_SIZE: u64 = kilobytes(64) / FRAME_SIZE;
    const KERNEL_STACKS_MAX: u64 = 512;

    let core_id = 0; // TODO actually multicore
    assert!(core_id < KERNEL_STACKS_MAX, "core={}", core_id);

    let stack_start =
        VirtualAddress::with_literal(KERNEL_STACKS_START + (core_id * KERNEL_STACK_MAX_SIZE));

    let new_stack = crate::process::allocate_kernel_stack(
        &mut AddressSpace::current(),
        stack_start,
        KERNEL_STACK_SIZE as usize,
    )
    .expect("couldn't allocate kernel stack");

    debug!("new stack allocated at {:?}", new_stack);

    unsafe {
        asm!(
        "mov rax, {func}", // we won't be able to access this on the old stack
        "mov rsp, {stack_top}", // switcharoo
        "call rax",
        stack_top = in(reg) new_stack.address(),
        func = in(reg) run_me,
        out("rax") _,
        )
    };

    unreachable!()
}
