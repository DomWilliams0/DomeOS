; align loaded modules on page boundaries
MBALIGN     equ 1 << 0

; provided memory map
MEMINFO     equ 1 << 1

; multiboot flag
MB_FLAGS    equ MBALIGN | MEMINFO

; multiboot magic
MAGIC       equ 0x1BADB002

; checksum
CHECKSUM    equ -(MAGIC + MB_FLAGS)


; multiboot header
section .multiboot
align   4
    dd MAGIC
    dd MB_FLAGS
    dd CHECKSUM


; allocate petit stack of 16KiB
section .bss
align   4
stack_bottom:
resb 16384
stack_top:

section .text

; gdt flushing
global  gdt_flush
extern  gdt_descriptor
gdt_flush:
lgdt    [gdt_descriptor]

; 0x10 = 16 = 2nd entry = data segment
mov     ax, 0x10
mov     ds, ax
mov     es, ax
mov     fs, ax
mov     gs, ax
mov     ss, ax

; 0x08 = 08 = 1st entry = code segment
jmp     0x08:flush_far_jump

flush_far_jump:
    ret

; entry point
global _start:function (_start.end - _start)

_start:

; setup stack
mov     esp, stack_top

; boot kernel
extern  kernel_main
call    kernel_main

; hang
cli
jmp     $

.end:
