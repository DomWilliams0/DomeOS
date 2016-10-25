#include "util/io.h"
#include "util/util.h"
#include "screen.h"
#include "isr.h"
#include "irq.h"
#include "clock.h"


void clock_set_interval(int hz)
{
    int divisor = 1193180 / hz;

    struct pit_command_repr cmd = {
        .bcd     = 0, // binary
        .mode    = 3, // square wave
        .rw      = 3, // lo and hi bytes
        .channel = 0  // channel 0
    };

    // write command
    uint8_t *cmd_int = (uint8_t *)&cmd;
    io_write_port(PIT_COMMAND, *cmd_int);

    // write divisor
    io_write_port(PIT_CHANNEL0_DATA, divisor & 0xFF); // lo
    io_write_port(PIT_CHANNEL0_DATA, divisor >> 8);   // hi
}

static void clock_handler(struct stack_context *context)
{
    UNUSED(context);

    static uint32_t ticks = 0;

    if (++ticks % CLOCK_HERTZ == 0)
    {
        screen_write_string("A second!");
    }
    else
    {
        screen_write_char('.');
    }

}

void clock_init()
{
    clock_set_interval(CLOCK_HERTZ);
    irq_register_handler(0, &clock_handler);
}
