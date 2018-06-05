#ifndef DOMEOS_IO_H
#define DOMEOS_IO_H

typedef unsigned short io_port;
typedef char port_data;

port_data io_read_port(io_port port);

void io_write_port(io_port port, port_data data);

#endif
