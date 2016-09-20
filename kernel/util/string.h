#ifndef __KERNEL_STRING_H__
#define __KERNEL_STRING_H__

typedef unsigned long size_t;

void memcpy(void *dst, void *src, size_t n);

void memset(void *s, int c, size_t n);

#endif
