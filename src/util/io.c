#include "screen.h"
#include "util/io.h"
#include "error.h"

char io_read_port(io_port port)
{
    port_data data;
    __asm__ __volatile__ ("inb %1, %0" : "=a" (data) : "dN" (port));

    return data;
}

void io_write_port(io_port port, port_data data)
{
    __asm__ __volatile__ ("outb %1, %0" : : "dN" (port), "a" (data));
}


void kputc(char c)
{
    screen_write_char(c);
}

void kputs(char *s)
{
    screen_write_string(s);
    screen_write_char('\n');
}

void kwrites(char *s)
{
    screen_write_string(s);
}

void print_error(struct err_state *error)
{
    if (error && error->err)
    {
        char *err_str = 0;
        get_error_str(error->err, &err_str);
        if (err_str)
        {
            screen_write_string("error ");
            screen_write_string(err_str);
            screen_write_string(" func ");
            screen_write_string(error->func);
            screen_write_char(' ');
            screen_write_string(error->file);
            screen_write_char(':');
            // screen_write_string(error->line);
            screen_write_string("LINE_GOES_HERE");
            screen_write_char('\n');
        }
    }
}
