
bits 64
section .data
g_message: db "bonjour!"
g_invalid: db "bad",0x41,0xe2,0x28,0xa1

section .text

global _start
_start:

; int syscall_log(utf8 str, str len bytes)
mov rax, 0 ; syscall 0
mov rdi, g_message
mov rsi, 8
o64 syscall

mov rdi, g_invalid
mov rsi, 8
o64 syscall

; ptr overflows over end of userspace
mov rsi, 0x300000000000
o64 syscall

jmp $
ud2
