org 0x7c00 ; will be loaded at this address
[bits 16]  ; real mode
stage0:
    jmp     loader

; OEM parameter block - for fs reading

bpbOEM                db "DomeOS  "
bpbBytesPerSector:    dw 512
bpbSectorsPerCluster: db 1
bpbReservedSectors:   dw 1
bpbNumberOfFATs:      db 2
bpbRootEntries:       dw 224
bpbTotalSectors:      dw 2880
bpbMedia:             db 0xF0
bpbSectorsPerFAT:     dw 9
bpbSectorsPerTrack:   dw 18
bpbHeadsPerCylinder:  dw 2
bpbHiddenSectors:     dd 0
bpbTotalSectorsBig:   dd 0
bsDriveNumber:        db 0
bsUnused:             db 0
bsExtBootSignature:   db 0x29
bsSerialNumber:       dd 0xa0a1a2a3
bsVolumeLabel:        db "MOS      FLOPPY "
bsFileSystem:         db "FAT12    "

s_booting    db    "[+] Booting DomeOS...", 0

; ds=>si: nul terminated string
print:
    lodsb
    or      al, al  ; al=current character
    jz      .done   ; nul terminator found
    mov     ah, 0eh ; for int 10h -> get next character
    int     10h
    jmp     print  ; loop
.done:
    ret

loader:
    xor    ax, ax ; null data secments because theyre in the same 0x7c00:0 range as code
    mov    ds, ax
    mov    es, ax

    mov    si, s_booting
    call   print

    ; TODO get available memory

    cli   ; disable interrupts
    jmp $ ; loop forever

times 510 - ($-$$) db 0 ; pad to 512 bytes

dw 0xAA55 ; boot signature
