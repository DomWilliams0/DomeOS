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

; idt flushing
global  idt_flush
extern  idt_descriptor
idt_flush:
	lidt    [idt_descriptor]
	ret

