#include "screen.h"
#include "string.h"

struct
{
	int cursor_x;
	int cursor_y;

	screen_colour fg;
	screen_colour bg;

} state;

static screen_char get_colour(char c)
{
	return colour_char(c, create_colour(state.fg, state.bg));
}

void screen_init(screen_colour fg, screen_colour bg)
{
	state.cursor_x  = 0;
	state.cursor_y  = 0;

	state.fg = fg;
	state.bg = bg;

	screen_clear();
}


void screen_clear()
{
	screen_char c = get_colour(' ');

	size_t n = SCREEN_WIDTH * SCREEN_HEIGHT;
	while (n--)
		*(SCREEN_VIDEO_MEM + n) = c;
}

void screen_scroll_down()
{
	int used_rows = state.cursor_y - SCREEN_HEIGHT + 1;
	int effective_height = SCREEN_HEIGHT - used_rows;

	// move all rows up
	kwmemcpy(SCREEN_VIDEO_MEM, SCREEN_VIDEO_MEM + SCREEN_WIDTH, SCREEN_WIDTH * effective_height);

	// clear bottom row
	screen_char blank = get_colour(' ');
	kwmemset(SCREEN_VIDEO_MEM + effective_height * SCREEN_WIDTH, blank, SCREEN_WIDTH);

	state.cursor_y -= 1;
}

void screen_write_char(char c)
{
	// TODO bool
	int new_line = 0;
	int visible_char = 1;

	// special char
	if (c == '\n')
	{
		new_line = 1;
		visible_char = 0;
	}

	// print char
	if (visible_char)
	{
		int index = state.cursor_x + (state.cursor_y * SCREEN_WIDTH);
		*(SCREEN_VIDEO_MEM + index) = get_colour(c);

		state.cursor_x += 1;
	}

	// new line
	if (new_line || state.cursor_x >= SCREEN_WIDTH)
	{
		state.cursor_x = 0;
		state.cursor_y += 1;

		if (state.cursor_y >= SCREEN_HEIGHT)
		{
			screen_scroll_down();
		}
	}
}

void screen_write_string(char *s)
{
	char *c = s;
	while (*c != '\0')
	{
		screen_write_char(*c);
		++c;
	}
}
