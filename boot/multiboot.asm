MB_HEADER_MAGIC equ 0x1BADB002
MB_HEADER_FLAGS equ  1 | 2 ; PAGE_ALIGN | MEMORY_INFO
MB_CHECKSUM     equ -(MB_HEADER_MAGIC + MB_HEADER_FLAGS)

section .multiboot
header_start:
	dd MB_HEADER_MAGIC
	dd MB_HEADER_FLAGS
	dd MB_CHECKSUM

	dd 0
	dd 0
	dd 0
	dd 0
	dd 0

	dd 0
	dd 0
	dd 0
	dd 0
header_end:

; vim: ft=nasm
