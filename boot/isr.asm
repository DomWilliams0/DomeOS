%define isr_count 32

%macro declare_isr 1
	global isr%1
%endmacro

%macro define_isr 1
	isr%1:
%endmacro

%assign i 0
%rep isr_count
	declare_isr i
%assign i i+1
%endrep

; define isrs
; only certain push error codes onto the stack; 0 used in absence
%assign i 0
%rep isr_count

define_isr i
	 cli

; 08: double fault
; 10: bad tss
; 11: segment not present
; 12: stack fault
; 13: general protection fault
; 14: page fault
; 17: alignment check
%if i == 08 ||\
	i == 10 ||\
	i == 11 ||\
	i == 12 ||\
	i == 13 ||\
	i == 14 ||\
	i == 17

	; error code already pushed
%else
	; dummy value needed
	push byte 0
%endif
	push byte i
	jmp isr_stub

%assign i i+1
%endrep

extern fault_handler
isr_stub:
	; push all registers to stack
	pusha
	push	ds
	push	es
	push	fs
	push	gs

	; load data segment descriptor
	mov     ax, 0x10
	mov	    ds, ax
	mov	    es, ax
	mov	    fs, ax
	mov	    gs, ax

	; push stack pointer
	mov     eax, esp
	push    eax

	; "call" fault handler, preserving eip
	mov     eax, fault_handler
	call    eax

	; pop registers back off stack
	pop     eax
	pop     gs
	pop     fs
	pop     es
	pop     ds
	popa

	; clean up error code and isr id
	add     esp, 8

	; pop the rest and return
	iret
