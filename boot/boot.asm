global _start
extern kernel_main

section .bss
align 4096

; allocate petit stack of 16KiB
stack_bottom:
	resb 16384
stack_top:


section .text
bits	 32

; entry point
global _start:function(_start.end - _start)
_start:

	; setup stack
	mov esp, stack_top

	; jump into kernel
	call kernel_main

	; loop
	cli
	jmp $


; vim: ft=nasm
