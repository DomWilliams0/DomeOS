use crate::cpu::CpuState;
use core::ffi::c_void;
use memory::VIRT_USERSPACE_MAX;
use syscall::{SyscallError, SyscallResult};

const COUNT: usize = 1;
static TRAMPOLINES: [unsafe extern "C" fn() -> !; COUNT] = [tramp_validate_log];

#[naked]
pub unsafe extern "C" fn syscall_entry() -> ! {
    const ERR_INVALID: u64 = SyscallResult::error(SyscallError::InvalidSyscall).to_u64();

    asm!(
        // rax=syscall number
        // rcx=rip to return to, must be restored before sysret
        // r11=rflags

        // make kernel structures available
        "swapgs",

        // check syscall number is valid
        "cmp rax, {syscall_count}", // TODO works with negative too?
        "jae 1f",

        // jmp to syscall trampoline, clobbering r10
        "mov r10, {trampoline_base}",
        "add r10, rax",
        "jmp r10",

        // bad syscall
        "1: mov rax, {err_invalid}",
        "swapgs",
        "sysretq",

        trampoline_base = sym TRAMPOLINES,
        syscall_count = const COUNT,
        err_invalid = const ERR_INVALID,
        options(noreturn)
    )
}

// TODO autogenerate trampolines based on number of args
// TODO reshuffle needed for args 2 and 3 (rbx->rdx, rdx->rcx (backing up rcx to e.g. r10?)
#[naked]
unsafe extern "C" fn tramp_validate_log() -> ! {
    const ERR_ARGS: u64 = SyscallResult::error(SyscallError::InvalidSyscall).to_u64();
    asm!(
        // rdi = pointer value
        "mov r10, {userspace_max}",
        "cmp rdi, r10",
        "jae 1f",

        // rsi = length
        // bounds check length of string (r10 = max-(str+len))
        "sub r10, rdi",
        "sub r10, rsi",
        "js 1f", // error if negative, sf=1

        // args validated, switch to kernel stack and run handler
        "mov rsp, gs:{gs_stack_offset}",
        "jmp {handler}",

        // bad args. we can sysret immediately without any other reg restoring required because
        // no other registers have been clobbered or functions called
        "1: mov rax, {err_args}",
        "swapgs",
        "sysretq",

        userspace_max = const VIRT_USERSPACE_MAX,
        gs_stack_offset = const CpuState::THREAD_KERNEL_STACK_OFFSET,
        handler = sym tramp_to_rust_log,
        err_args = const ERR_ARGS,
        options(noreturn)
    )
}

#[naked]
unsafe extern "C" fn common_return() -> ! {
    asm!("", options(noreturn))
}

unsafe extern "C" fn tramp_to_rust_log(string: *mut c_void, len: u64) -> ! {
    // call rust abi handler
    syscall_log(string, len);

    common_return()
}

fn syscall_log(_string: *mut c_void, _len: u64) {
    common::info!("wow!");
    unsafe { asm!("", in("rcx") 0x41414141) }
}
