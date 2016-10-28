#ifndef __KERNEL_SERIAL_H__
#define __KERNEL_SERIAL_H__

void serial_init();

void serial_putc(char c);

void serial_puts(char *s);

#endif
