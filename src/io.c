#include "io.h"

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
