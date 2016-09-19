; bios offset
[org 0x7c00]

	; init stack
	mov     bp, 0x9000
	mov     sp, bp

	; switch mode
	mov     ax, str_real_mode
	call    print_string
    call    switch_to_prot_mode

	; hang
	jmp     $

; protected mode has been fully initialised
ENTRY_PROT_MODE:
    mov     eax, str_prot_mode
    call    print_string

    ; hang
    jmp     $
    
; includes
%include "boot/print_string.asm"
%include "boot/prot_mode.asm"

; strings
str_real_mode   db "Started in 16-bit real mode", 0
str_prot_mode   db "Landed in 32-bit protected mode", 0

; padding and magic bios number
times   510-($-$$) db 0
dw      0xaa55

