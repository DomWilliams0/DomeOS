#include "serial.h"
#include "io.h"

#define SERIAL_PORT 0x3F8 // COM1

void serial_init()
{
    // disable interrupts
    io_write_port(SERIAL_PORT + 1, 0x00);

    // enable DLAB by setting high bit
    io_write_port(SERIAL_PORT + 3, 0x80);

    // set lo and hi bytes of baud rate
    // default (115200) / 6 = 19200
    io_write_port(SERIAL_PORT + 0, 0x03);
    io_write_port(SERIAL_PORT + 1, 0x00);

    // 8 bits       11
    // no parity    000
    // 1 stop bit   0
    // disable DLAB 0
    io_write_port(SERIAL_PORT + 3, 0x03);

    // enable fifo
    // threshold of 14 bytes
    io_write_port(SERIAL_PORT + 2, 0xC7);

    // enable interrupts for;
    // data available    1
    // transmitter empty 1
    // not break/error   0
    // status change     1
    io_write_port(SERIAL_PORT + 4, 0x0B);
}

static int can_send()
{
    return io_read_port(SERIAL_PORT + 5) & 0x20;
}

void serial_putc(char c)
{
    while (!can_send());

    io_write_port(SERIAL_PORT, c);
}

void serial_puts(char *s)
{
    while (!can_send());

    while(*s)
    {
        io_write_port(SERIAL_PORT, *s);
        s++;
    }
}

