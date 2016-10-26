#ifndef __KERNEL_STRING_H__
#define __KERNEL_STRING_H__

typedef unsigned long ksize_t;

void kmemcpy(void *dst, void *src, ksize_t n);

void kmemset(void *s, int c, ksize_t n);

#endif
