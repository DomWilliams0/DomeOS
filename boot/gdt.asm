gdt_start:

; null descriptor
gdt_null:
	dd  0x0
	dd  0x0
	
; code segment
gdt_code:
; base=0x0, limit=0xfffff
; 1st flags:  1(present)00(priv)1(descriptor type)
; type flags: 1(code)0(conforming)1(readable)0(accessed)
; 2nd flags:  1(granularity)1(32-bit default)0(64 bit)0(AVL)

	dw  0xffff		; limit 0-15
	dw  0x0         ; base 0-15
	dw  0x0         ; base 16-23
	db  10011010b	; 1st and type flags
	db  11001111b	; 2nd flags, limit 16-19
	db  0x0         ; base 24-31

; data segment
gdt_data:
; same as code except:
; type flags: 0(code)0(expand down)0(writable)0(accessed)
	dw  0xffff		; limit 0-15
	dw  0x0         ; base 0-15
	dw  0x0         ; base 16-23
	db  10010010b	; 1st and type flags
	db  11001111b	; 2nd flags, limit 16-19
	db  0x0         ; base 24-31
	
; end label to allow calculation of gdt size
gdt_end:

gdt_descriptor:
	dw  gdt_end - gdt_start - 1	; size of GDT
	dd  gdt_start				; start address

; offsets to segments
CODE_SEG    equ gdt_code - gdt_start
DATA_SEG    equ gdt_data - gdt_start
