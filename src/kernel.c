#include "screen.h"
#include "gdt.h"
#include "idt.h"
#include "isr.h"
#include "clock.h"
#include "serial.h"

#include "io.h"
#include "string.h"
#include "util.h"
#include "boot/multiboot.h"


// static unsigned long discover_memory(multiboot_info_t *mbinfo, unsigned int magic)
// {
//     if (mbinfo->flags & 1)
//         return mbinfo->mem_upper;
//
//     return 0;
// }

void kernel_main(multiboot_info_t *mbinfo, unsigned int magic)
{
    serial_init();
    gdt_init();
    idt_init();
    clock_init();
    enable_interrupts();

    screen_init(SCREEN_COLOUR_WHITE, SCREEN_COLOUR_BLACK);

    char *test_string = "This is a line of text that fills up a row exactly, what are the chances ?!!!!!!";
    for (int i = 0; i < 5; ++i)
    {
        test_string[0] = '1' + i;
        screen_write_string(test_string);
    }

    // hang forever
    while (1);
}
