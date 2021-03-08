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

        // args validated! switch to kernel stack
        "mov r10, rsp",
        "mov rsp, gs:{gs_stack_offset}",

        // preserve values
        // TODO r11 for user flags too
        "push rcx", // user rip
        "push r10", // user rsp

        // call C ABI syscall handler
        // TODO preserve registers for C ABI
        "call {handler}",
        "jmp {success_return}",

        // bad args. we can sysret immediately without any other reg restoring required because
        // no other registers have been clobbered or functions called
        "1: mov rax, {err_args}",
        "swapgs",
        "sysretq",

        userspace_max = const VIRT_USERSPACE_MAX,
        gs_stack_offset = const CpuState::THREAD_KERNEL_STACK_OFFSET,
        handler = sym tramp_to_rust_log,
        success_return = sym common_return,
        err_args = const ERR_ARGS,
        options(noreturn)
    )
}

/// Returns to userspace after a syscall handler has run
///
/// Assumptions:
/// * swapgs has been run once
/// * we're on kernel stack of current thread
/// * stack is (top-->bottom) [ user rsp, user rip ]
#[naked]
unsafe extern "C" fn common_return() -> ! {
    asm!(
        "pop r10", // user rsp
        "pop rcx", // user rip
        // restore user stack
        "mov rsp, r10",
        // back we go
        "swapgs",
        "sysretq",
        options(noreturn)
    )
}

/// Trampoline through C ABI to Rust ABI of syscall handler
unsafe extern "C" fn tramp_to_rust_log(string: *mut c_void, len: u64) -> SyscallResult {
    // call rust abi handler
    let res = syscall_log(string, len);

    // return value in rax
    SyscallResult::from(res)
}

fn syscall_log(string: *mut c_void, len: u64) -> Result<(), SyscallError> {
    // safety: verified by syscall handler
    let slice = unsafe { core::slice::from_raw_parts(string as *mut u8, len as usize) };
    match core::str::from_utf8(slice) {
        Ok(s) => {
            common::info!("message from userspace: '{}'", s);
            Ok(())
        }
        Err(_) => {
            common::warn!("bad utf8!");
            Err(SyscallError::InvalidArguments)
        }
    }
}
