global _start
extern kernel_main
extern long_mode
extern gdt64_flush
extern init_pml4

%include "boot.h"

section .boot.bss
align 4096
bss_start:

; allocate petit stack
stack_bottom:
	resb INITIAL_STACK_SIZE
stack_top:

bss_end:

section .boot.text
bits	 32

; entry point
_start:

	; setup stack
	mov esp, stack_top

	; TODO reset EFLAGS?

	; multiboot parameters, preserved for kernel entrypoint
	push eax
	push ebx

	; paging
	call enable_paging

	; pop multiboot params
	pop esi
	pop edi

	; init and jump to long mode
	call gdt64_flush

enable_paging:
	; put p4 in cr3
	mov eax, (init_pml4 - KERNEL_VIRT)
	mov cr3, eax

	; pae (bit 5)
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; long bit and nx-enable bit in EFER
	mov ecx, 0xC0000080
	rdmsr
	or eax, (1 << 8) | (1 << 11)
	wrmsr

	; paging bit and write-protect
	mov eax, cr0
	or eax, 1 << 31 | 1 << 16
	mov cr0, eax

	ret

; vim: ft=nasm
