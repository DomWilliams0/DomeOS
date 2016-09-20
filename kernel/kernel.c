#include "screen.h"

void kernel_main()
{
	// clear screen
	screen_colour colour = create_colour(SCREEN_COLOUR_LIGHT_GREY, SCREEN_COLOUR_BLACK);
	clear_screen(' ', colour);

	// test strings
    char *test_string = "Test string";
    enum screen_colour_primitive bg = SCREEN_COLOUR_BLACK;
    for (int y = 0, x = 0, fg = SCREEN_COLOUR_BLUE; y < SCREEN_HEIGHT; ++y, x += 11, ++fg)
    {
        screen_colour sc = create_colour(fg % SCREEN_COLOUR_WHITE, bg);
	    write_string(x, y, test_string, sc);
    }
}
