# Syscalls

* `SYSCALL`/`SYSRET` are used, which unconditionally clobber `rcx` and `r11`
* Syscall number is passed in `rax`
	* TODO use high bits to specify platform compatibility (Windows, POSIX, DomeOS)
* Return value is passed in `rax`
* Arguments are passed right-to-left in `rdi`, `rsi`, `rbx`, `rdx`, `r8`, `r9`
	* Only integers (including pointers) allowed
	* Limited to 6
