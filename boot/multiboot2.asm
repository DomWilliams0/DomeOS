MB_MAGIC    equ 0xe85250d6
MB_ARCH     equ 0 ; i386
MB_LEN      equ header_end - header_start
MB_CHECKSUM equ 0x100000000 - (MB_MAGIC + 0 + MB_LEN)

section .multiboot
header_start:
	dd MB_MAGIC
	dd MB_ARCH
	dd MB_LEN
	dd MB_CHECKSUM

	dw 0
	dw 0
	dd 8
header_end:

; vim: ft=nasm
