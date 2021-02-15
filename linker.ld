/* entry point */
ENTRY(_start)

KERNEL_PHYS = 0x100000;
KERNEL_VIRT = 0xffffffff80000000;

SECTIONS
{
    . = KERNEL_PHYS;

    .boot :
    {
        KEEP(*(.multiboot))
        *(.boot.*)
    }

    . += KERNEL_VIRT;

    .text : AT(ADDR(.text) - KERNEL_VIRT)
    {
        _code = .;
        *(.text.*)
        *(.rodata.*)
        . = ALIGN(4096);
    }

    /* initialised rw data */
    .data : AT(ADDR(.data) - KERNEL_VIRT)
    {
        _data = .;
        *(.data.*)
        . = ALIGN(4096);
    }

    /* uninitialised rw data and stack */
    .bss : AT(ADDR(.bss) - KERNEL_VIRT)
    {
        _bss = .;
        *(.bss.*)
        *(COMMON)
        . = ALIGN(4096);
    }

    /DISCARD/ :
    {
        *(.comment)
    }

    KERNEL_END = .;
}
