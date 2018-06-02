global _start
extern kernel_main

global KERNEL_VMA
KERNEL_VMA equ 0x100000 ; 1MiB

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
	; call kernel_main

	; print pretty message instead
	mov dword [0xb8000], 0x2b451a4e
	mov dword [0xb8004], 0x4d543e41
	mov dword [0xb8008], 0x5f4f

	; loop
	cli
	jmp $


; vim: ft=nasm
