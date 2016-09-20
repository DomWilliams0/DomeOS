#include "util/util.h"
#include "screen.h"


RESULT write_char(int x, int y, char c, screen_colour colour)
{
    RESULT result = RESULT_SUCCESS;

	// TODO asserts possible?
	// assert x >= 0 && x < SCREEN_WIDTH;
	// assert y >= 0 && y < SCREEN_HEIGHT;

	int index = x + (y * SCREEN_WIDTH);
	screen_char coloured = colour_char(c, colour);

	*(SCREEN_VIDEO_MEM + index) = coloured;

    return result;
}

RESULT write_string(int x, int y, char *s, screen_colour colour)
{
    RESULT result = RESULT_SUCCESS;

	int index = x + (y * SCREEN_WIDTH);

	char *c = s;
	while (*c != '\0')
	{
		screen_char coloured = colour_char(*c, colour);
		*(SCREEN_VIDEO_MEM + index) = coloured;
		++index;
		++c;
	}

    return result;
}

RESULT clear_screen(char c, screen_colour colour)
{
	RESULT result = RESULT_SUCCESS;

	unsigned int len = (SCREEN_WIDTH) * (SCREEN_HEIGHT);
	screen_char coloured = colour_char(c, colour);
	for (unsigned int i = 0; i < len; ++i)
	{
		*(SCREEN_VIDEO_MEM + i) = coloured;
	}

	return result;
}
