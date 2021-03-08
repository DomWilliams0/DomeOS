
bits 64
section .data
g_message: db "bonjour from userspace!!1!"

section .text

global _start
_start:

; int syscall_log(utf8 str, str len bytes)

mov rax, 0 ; syscall 0
mov rdi, g_message
mov rsi, 26
o64 syscall

jmp $
