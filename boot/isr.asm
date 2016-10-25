%define isr_count 32
%define irq_count 16

; declaration/definition macros

; isr
%macro declare_isr 1
	global isr%1
%endmacro

%macro define_isr 1
	isr%1:
%endmacro

; irq
%macro declare_irq 1
	global irq%1
%endmacro

%macro define_irq 1
	irq%1:
%endmacro

; declarations of symbols
; isrs
%assign i 0
%rep isr_count
	declare_isr i
%assign i i+1
%endrep

; irqs
%assign i 0
%rep irq_count
	declare_irq i
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

; define irqs
%assign i 0
%rep irq_count

define_irq i
	cli
	push byte 0
	push byte i
	jmp irq_stub
%assign i i+1
%endrep

; common stub
extern fault_handler
extern irq_handler

%macro define_stub 1

%ifidn %1,irq
%define stub_name    irq_stub
%define stub_handler irq_handler
%else
%define stub_name    isr_stub
%define stub_handler fault_handler
%endif

stub_name:
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

	; "call" handler, preserving eip
	mov     eax, stub_handler
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
%endmacro

define_stub irq
define_stub isr
