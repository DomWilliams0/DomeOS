/* entry point */
ENTRY(_start)

SECTIONS
{
	/* start at arbitrary address */
	. = KERNEL_VMA;

	.boot : 
	{
		KEEP(*(.multiboot))
	}

	.text : AT(ADDR(.text) - KERNEL_VMA)
	{
		_code = .;
		*(.text)
		*(.rodata)
		. = ALIGN(4096);
	}

	/* initialised rw data */
	.data : AT(ADDR(.data) - KERNEL_VMA)
	{
		_data = .;
		*(.data)
		. = ALIGN(4096);
	}

	/* exception handling frames */
	.eh_frame : AT(ADDR(.eh_frame) - KERNEL_VMA)
	{
		_ehframe = .;
		*(.eh_frame)
		. = ALIGN(4096);
	}

	/* uninitialised rw data and stack */
	.bss : AT(ADDR(.bss) - KERNEL_VMA)
	{
		_bss = .;
		*(.bss)
		*(COMMON)
		. = ALIGN(4096);
	}

	/DISCARD/ :
	{
		*(.comment)
	}
}
