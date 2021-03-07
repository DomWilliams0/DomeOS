pub fn syscall_entry() -> ! {
    // backup return address and rax
    let (rcx, rax): (u64, u64);
    unsafe { asm!("", out("rcx") rcx, out("rax") rax) }

    // TODO switch to kernel stack at some point
    common::info!("syscall!!");

    // back we go
    unsafe { asm!("sysretq", in("rcx") rcx, in("rax") rax, options(noreturn)) }
}
