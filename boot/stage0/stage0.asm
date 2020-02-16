org 0x7c00 ; will be loaded at this address
[bits 16]  ; real mode

stage0:
	cli
	jmp $

times 510 - ($-$$) db 0 ; pad to 512 bytes

dw 0xAA55 ; boot signature
