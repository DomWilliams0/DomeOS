#include "logging.h"
#include "serial.h"

void log(char *prefix, char *message)
{
    serial_puts(prefix);
    serial_puts(message);
    serial_putc('\n');
}
