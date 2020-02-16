org 0x7c00 ; will be loaded at this address
[bits 16]  ; real mode
stage0:
    jmp     loader

; OEM parameter block - for fs reading

s_booting    db    "[+] Booting DomeOS...", 0
s_error      db    "[!] Fatal error", 0
s_sectorsz   db    "[+] Sector size is not 52?!", 0
s_newline    db    0x0d, 0x0a, 0

; ds=>si: nul terminated string
print:
    mov     bh, 1      ; new line flag set initially
.loop:
    lodsb              ; load ds:si into al and increment si
    or      al, al     ; al=current character
    jz      .newline   ; nul terminator found
    mov     ah, 0eh    ; for int 10h -> get next character
    int     10h
    jmp     .loop      ; loop

.newline:
    cmp     bh, 1
    je     .newline_prep ; if bh is 1, loop again with a newline
    ret                   ; otherwise its 0 so just return

.newline_prep:
    xor     bh, bh ; zero bh so next loop will ret
    mov     si, s_newline
    jmp     .loop


loader:
    cli

    ; TODO preserve DL, its the drive number
    xor    ax, ax ; null data segments because theyre in the same 0x7c00:0 range as code
    mov    ds, ax
    mov    es, ax

    mov    si, s_booting
    call   print
    mov     si, ds

    ; TODO get available memory

    ; TODO check if supported with ah=41h

    ; get drive params
    mov     ax, 7e0h
    mov     ds, ax ; use 0x50:0 as data segment (from 0x500 physical)
    mov     byte [0], 1eh ; length
    mov     ah, 48h ; for int 13h -> extended read drive params, leave dl as it is
    int     13h
    jc fatal_error

    ; verify sector size == 512 TODO use this value instead of 512
    mov     ax, [18h]
    cmp     ax, 512
    jnz     .bad_sector_size

    jmp $

.bad_sector_size:
    xor     ax, ax
    mov     ds, ax
    mov     si, s_sectorsz
    call    print
    ret


fatal_error:
    cli
    xor     ax, ax
    mov     ds, ax
    mov     si, s_error
    call    print
    hlt

times 510 - ($-$$) db 0 ; pad to 512 bytes

dw 0xAA55 ; boot signature
