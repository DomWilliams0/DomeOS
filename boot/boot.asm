global _start
extern kernel_main
extern long_mode
extern gdt64_flush
extern KERNEL_VMA

section .bss
align 4096
bss_start:

; page tables
p4_table:
	resb 4096
p3_table:
	resb 4096
p2_table:
	resb 4096

; allocate petit stack of 16KiB
stack_bottom:
	resb 16384
stack_top:

bss_end:

section .text
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
	call init_page_tables
	call enable_paging

	; pop multiboot params
	pop esi
	pop edi

	; init and jump to long mode
	call gdt64_flush

init_page_tables:
	; 0b11 = present and writable bits

	; first P4 -> p3
	mov eax, p3_table
	or eax, 0b11
	mov [p4_table], eax

	; first P3 -> p2
	mov eax, p2_table
	or eax, 0b11
	mov [p3_table], eax

	; map all p2 entries to 2MiB entries
	mov ecx, 0

.loop:
	mov eax, 0x200000	; 2MiB
	mul ecx
	or eax, 0b10000011 ; present + w + huge
	mov [p2_table + ecx * 8], eax

	inc ecx
	cmp ecx, 512
	jne .loop

	ret

enable_paging:

	; put p4 in cr3
	mov eax, p4_table
	mov cr3, eax

	; pae (bit 5)
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; long bit in EFER
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; paging bit
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

; vim: ft=nasm
