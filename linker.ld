/* entry point */
ENTRY(_start)

SECTIONS
{
	/* start sections at 1MB */
	. = 0x100000;

	.boot :
	{
		KEEP(*(.multiboot))
	}

	.text :
	{
		_code = .;
		*(.text)
		*(.rodata)
		. = ALIGN(4096);
	}

	/* initialised rw data */
	.data :
	{
		_data = .;
		*(.data)
		. = ALIGN(4096);
	}

	/* exception handling frames */
	.eh_frame :
	{
		_ehframe = .;
		*(.eh_frame)
		. = ALIGN(4096);
	}

	/* uninitialised rw data and stack */
	.bss :
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

	KERNEL_END = .;
}
