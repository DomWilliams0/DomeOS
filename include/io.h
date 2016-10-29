#ifndef __KERNEL_IO_H__
#define __KERNEL_IO_H__

typedef unsigned short io_port;
typedef char port_data;

port_data io_read_port(io_port port);

void io_write_port(io_port port, port_data data);

void kputc(char c);

void kputs(char *s);

void kwrites(char *s);

#endif
