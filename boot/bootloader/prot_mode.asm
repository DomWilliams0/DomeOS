[bits 16]
switch_to_prot_mode:
	
	; disable interrupts
	cli

	; load GDT
	lgdt    [gdt_descriptor]

	; set prot mode bit
	mov     eax, cr0
	or      eax, 0x1
	mov     cr0, eax

	; far jump to flush pipeline
	mov     ebx, 0xbeef
	jmp     CODE_SEG:init_prot_mode

[bits 32]
init_prot_mode:
; definitely in protected mode here

	; update segments
	mov     ax, DATA_SEG
	mov     ds, ax
	mov     ss, ax
	mov     es, ax
	mov     fs, ax
	mov     gs, ax

	; update stack pointer
	mov     ebp, 0x90000
	mov     esp, ebp

	; boot kernel
	call    ENTRY_PROT_MODE
	
%include "boot/gdt.asm"
