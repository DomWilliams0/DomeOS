to_print:
	db 'Alright lad',0

	; offset
	org 0x7c00

	; counter in cx
	mov		cx, 0

printer:
	; put char addr in bx
	mov		bx,	to_print
	add		bx,	cx

	; put char in al
	mov		al,	[bx]

	; reached end
	test	al,	al
	jz		end

	; print char and increment
	mov		ah,	0x0e
	int		0x10
	inc		cx
	jmp		printer

end:
	; print underscore
	mov		al, '_'
	int		0x10

	; loop 5eva
	jmp		$

	; pad to 512 and add magic BIOS marker
	times	510-($-$$) db 0
	dw 		0xaa55
