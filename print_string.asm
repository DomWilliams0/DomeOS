print_string:
	; save registers
	push bx
	push cx
	push dx

	; counter in cx
	mov		cx, 0

	; string in dx
	mov		dx, ax

print_loop:
	; put char addr in bx
	mov		bx,	dx
	add		bx,	cx

	; put char in al
	mov		al,	[bx]

	; reached end
	test	al,	al
	jz		loop_end

	; print char and increment
	mov		ah,	0x0e
	int		0x10
	inc		cx
	jmp		print_loop

loop_end:
	; restore registers and return
	pop dx
	pop cx
	pop bx
	ret
