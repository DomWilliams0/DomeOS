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
; 30: security exception
%if i == 08 ||\
	i == 10 ||\
	i == 11 ||\
	i == 12 ||\
	i == 13 ||\
	i == 14 ||\
	i == 17 ||\
	i == 30

	; error code already pushed
%else
	; dummy value needed
	push byte 0
%endif
	push byte i ; int number
	jmp isr_stub

%assign i i+1
%endrep

; define irqs
%assign i 32
%rep irq_count

%assign irq_i i-32
define_irq irq_i
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
	; store all registers
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rbp
	push rdi
	push rsi
	push rdx
	push rcx
	push rbx
	push rax

	; 21*8 = sizeof intr_context
	lea rdi, [rsp + (21*8)]

	; TODO store segment descriptors only if 32 bit must be supported

	; swap user and kernel gs registers
	; TODO dont bother when already in kernel mode
	swapgs

	; call handler
	call stub_handler

	; restore registers
	pop rax
	pop rbx
	pop rcx
	pop rdx
	pop rsi
	pop rbp
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15

	iretq
%endmacro

define_stub irq
define_stub isr

; vim: ft=nasm
