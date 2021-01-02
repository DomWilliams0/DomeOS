extern gdt64
extern kernel_main
global long_mode

section .boot.text
bits 64

long_mode:
	cli

	mov ax, 0
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax

	; ensure the stack is 16-byte aligned
	mov rax, rsp
	and rax, ~(16 - 1)
	mov rsp, rax

	; TODO setup page table for higher half
	; jump into kernel
	call kernel_main

	; loop forever
	cli
	jmp $

; vim: ft=nasm
