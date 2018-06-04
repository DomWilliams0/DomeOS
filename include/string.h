#ifndef DOMEOS_STRING_H
#define DOMEOS_STRING_H

#include <stddef.h>

void kmemcpy(void *dst, void *src, size_t n);

void kwmemcpy(void *dst, void *src, size_t n);

void kmemset(void *s, int c, size_t n);

void kwmemset(void *s, int c, size_t n);

int kmemcmp(void *a, void *b, size_t n);

#endif
