#[naked]
pub unsafe extern "C" fn syscall_entry() -> ! {
    // TODO ensure all ptr args are in userspace

    asm!(
        // rcx=rip to return to, must be restored before sysret
        // r11=rflags

        // back we go
        //  * restore rcx
        "sysretq",
        options(noreturn)
    )
    // // backup return address and rax
    // let (rcx, rax): (u64, u64);
    // unsafe { asm!("", out("rcx") rcx, out("rax") rax) }
    //
    // // TODO switch to kernel stack at some point
    // common::info!("syscall!!");
    //
    // // back we go
    // unsafe { asm!("sysretq", in("rcx") rcx, in("rax") rax, options(noreturn)) }
}
