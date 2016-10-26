/* entry point */
ENTRY(_start)

SECTIONS
{
    /* start sections at 1 MiB */
    . = 1M;

    /* multiboot header */
    .text BLOCK(4K) : ALIGN(4K)
    {
        *(.multiboot)
        *(.text)
    }

    /* read only */
    .rodata BLOCK(4K) : ALIGN(4K)
    {
        *(.rodata)
    }

    /* initialised rw data */
    .data BLOCK(4K) : ALIGN(4K)
    {
        *(.data)
    }

    /* uninitialised rw data and stack */
    .bss BLOCK(4K) : ALIGN(4K)
    {
        *(COMMON)
        *(.bss)
    }
}
