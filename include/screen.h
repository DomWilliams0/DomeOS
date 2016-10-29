#ifndef __KERNEL_SCREEN_H__
#define __KERNEL_SCREEN_H__

#include <stdint.h>

#define SCREEN_VIDEO_MEM    (screen_char *)0xb8000
#define SCREEN_WIDTH        80
#define SCREEN_HEIGHT       25

#define SCREEN_PORT_CTRL    0x3D4
#define SCREEN_PORT_DATA    0x3D5

enum screen_colour_primitive
{
    SCREEN_COLOUR_BLACK         = 0x0,
    SCREEN_COLOUR_BLUE          = 0x1,
    SCREEN_COLOUR_GREEN         = 0x2,
    SCREEN_COLOUR_CYAN          = 0x3,
    SCREEN_COLOUR_RED           = 0x4,
    SCREEN_COLOUR_MAGENTA       = 0x5,
    SCREEN_COLOUR_BROWN         = 0x6,
    SCREEN_COLOUR_LIGHT_GREY    = 0x7,
    SCREEN_COLOUR_DARK_GREY     = 0x8,
    SCREEN_COLOUR_LIGHT_BLUE    = 0x9,
    SCREEN_COLOUR_LIGHT_GREEN   = 0xA,
    SCREEN_COLOUR_LIGHT_CYAN    = 0xB,
    SCREEN_COLOUR_LIGHT_RED     = 0xC,
    SCREEN_COLOUR_LIGHT_MAGENTA = 0xD,
    SCREEN_COLOUR_LIGHT_BROWN   = 0xE,
    SCREEN_COLOUR_WHITE         = 0xF
};

typedef uint16_t screen_char;
typedef uint8_t  screen_colour;

// colour operations
static inline screen_colour create_colour(enum screen_colour_primitive fg,
                                          enum screen_colour_primitive bg)
{
    return fg | bg << 4;
}

static inline screen_char colour_char(char c, screen_colour colour)
{
    return (screen_char) c | (screen_char) colour << 8;
}

// screen operations
void screen_init(screen_colour fg, screen_colour bg);

void screen_clear();

void screen_scroll_down();

void screen_write_char(char c);

void screen_write_string(char *s);

#endif
