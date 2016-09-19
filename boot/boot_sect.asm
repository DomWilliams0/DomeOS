; declare strings
nice_string:
	db 'Wow, nice string', 0
notsonice_string:
	db 'Did your mum allocate it?', 0
separator_string:
	db '.  ', 0

main:
	; bios offset
	org 0x7c00

	; init stack
	mov     bp, 0x8000
	mov     sp, bp

	; print them
	mov     ax, nice_string
	call    print_string

	mov     ax, separator_string
	call    print_string

	mov     ax, notsonice_string
	call    print_string

	; infinite loop
	jmp     $

	%include "print_string.asm"

	; padding and magic bios number
	times   510-($-$$) db 0
	dw      0xaa55

