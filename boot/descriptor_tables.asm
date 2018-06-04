extern long_mode

[bits 32]
gdt64:
	dq 0
.cs: equ $ - gdt64
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code, data not needed
	; flags set: descriptor type, present, exec, 64 bit
.ptr:
	dw $ - gdt64 - 1 ; length of gdt
	dq gdt64

gdt64_flush:
lgdt [gdt64.ptr]
jmp gdt64.cs:long_mode

; vim: ft=nasm
