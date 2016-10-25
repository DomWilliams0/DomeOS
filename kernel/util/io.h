#ifndef __KERNEL_IO_H__
#define __KERNEL_IO_H__

typedef unsigned short io_port;
typedef char port_data;

char io_read_port(io_port port);

void io_write_port(io_port port, port_data data);

#endif
