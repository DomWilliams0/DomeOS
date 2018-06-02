extern gdt64
global long_mode

[BITS 64]

long_mode:
	cli

	mov ax, 0
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax

	; jump into kernel
	; call kernel_main

	; print pretty message instead
	mov dword [0xb8000], 0x2b451a4e
	mov dword [0xb8004], 0x4d543e41
	mov dword [0xb8008], 0x5f4f

	hlt

; vim: ft=nasm
