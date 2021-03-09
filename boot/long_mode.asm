extern gdt64
extern kernel_main
global long_mode
global init_pml4

%include "boot.h"

section .data
align 0x1000

; ident map of first 32MB, and mirror it at the -2GB mark.
; matches kernel::memory::KERNEL_IDENTITY_MAPPING.
;
; tyvm https://github.com/eteran/os64/blob/master/arch/x86_64/boot.S
init_pml4:
	; +3 ===> present and R/W
	dq init_pdp - KERNEL_VIRT + 3 ; [0x0000000000000000 - 0x00000007ffffffff)
	times 510 dq 0
	dq init_pdp - KERNEL_VIRT + 3 ; [0xfffffff800000000 - 0xffffffffffffffff)

init_pdp:
	dq init_pd - KERNEL_VIRT + 3
	times 509 dq 0
	dq init_pd - KERNEL_VIRT + 3
	dq 0

init_pd:
	dq 0x0000000000000083 ; 0MB - 2MB
	dq 0x0000000000200083 ; 2MB - 4MB
	dq 0x0000000000400083 ; 4MB - 6MB
	dq 0x0000000000600083 ; 6MB - 8MB
	dq 0x0000000000800083 ; 8MB - 10MB
	dq 0x0000000000a00083 ; ...
	dq 0x0000000000c00083
	dq 0x0000000000e00083

	dq 0x0000000001000083
	dq 0x0000000001200083
	dq 0x0000000001400083
	dq 0x0000000001600083
	dq 0x0000000001800083
	dq 0x0000000001a00083
	dq 0x0000000001c00083
	dq 0x0000000001e00083 ; 30MB - 32MB

	times 496 dq 0


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

	; ensure stack is 16 byte aligned and in high memory
	mov rax, rsp
	and rax, ~(16 - 1)
	add rax, KERNEL_VIRT
	mov rsp, rax

	; ensure frame pointer is null so unwinding knows when to stop
	xor rbp, rbp

	; jump into kernel
	call kernel_main

	; loop forever
	cli
	jmp $

; vim: ft=nasm
