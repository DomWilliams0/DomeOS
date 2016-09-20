#include "screen.h"
#include "gdt.h"

void kernel_main()
{
    // create gdt
    gdt_init();

    screen_init(SCREEN_COLOUR_WHITE, SCREEN_COLOUR_BLACK);

    char *test_string = "This is a line of text that fills up a row exactly, what are the chances ?!!!!!!";
    for (int i = 0; i < 26; ++i)
    {
        test_string[0] = '1' + i;
        screen_write_string(test_string);
    }
}
