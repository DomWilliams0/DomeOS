#include "string.h"

void kmemcpy(void *dst, void *src, ksize_t n)
{
    while (n--)
        *(char *)dst++ = *(char *)src++;
}

void kmemset(void *s, int c, ksize_t n)
{
    while (n--)
        *(int *)s++ = c;
}
