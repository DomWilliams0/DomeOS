print_string:
    ; save registers
    push    bx

    ; current string position in bx
    mov     bx, ax

print_loop:
    ; put char in al
    mov     al, [bx]

    ; reached end
    test    al, al
    jz      loop_end

    ; print char and increment
    mov     ah, 0x0e
    int     0x10
    inc     bx
    jmp     print_loop

loop_end:
    ; restore registers and return
    pop     bx
    ret
