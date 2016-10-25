#include "screen.h"
#include "gdt.h"
#include "idt.h"
#include "isr.h"

void kernel_main()
{
    // init descriptor tables
    gdt_init();
    idt_init();
    enable_interrupts();

    screen_init(SCREEN_COLOUR_WHITE, SCREEN_COLOUR_BLACK);

    char *test_string = "This is a line of text that fills up a row exactly, what are the chances ?!!!!!!";
    for (int i = 0; i < 26; ++i)
    {
        test_string[0] = '1' + i;
        screen_write_string(test_string);
    }
}
