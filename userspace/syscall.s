
bits 64
section .data
g_message: db "bonjour!"

section .text

global _start
_start:

; int syscall_log(utf8 str, str len bytes)
loop:
mov rax, 0 ; syscall 0
mov rdi, g_message
mov rsi, 8
o64 syscall

jmp loop
ud2
