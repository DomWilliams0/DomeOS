#include "string.h"

void memcpy(void *dst, void *src, size_t n)
{
    while (n--)
        *(char *)dst++ = *(char *)src++;
}

void memset(void *s, int c, size_t n)
{
    while (n--)
        *(int *)s++ = c;
}
