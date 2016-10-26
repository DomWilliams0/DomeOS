#include "string.h"

void kmemcpy(void *dst, void *src, ksize_t n)
{
	char *s = (char *)src;
	char *d = (char *)dst;
	while (n--)
		*d++ = *s++;
}

void kwmemcpy(void *dst, void *src, ksize_t n)
{
	unsigned short *s = (unsigned short *)src;
	unsigned short *d = (unsigned short *)dst;
	while (n--)
		*d++ = *s++;
}

void kmemset(void *s, int c, ksize_t n)
{
	char *dst = (char *)s;
	while (n--)
		*dst++ = c;
}

void kwmemset(void *s, int c, ksize_t n)
{
	unsigned short *dst = (unsigned short *)s;
	while (n--)
		*dst++ = c;
}

int kmemcmp(void *a, void *b, ksize_t n)
{
	char *c = (char *)a;
	char *d = (char *)b;
	while (n--)
		if (*c != *d)
			return 0;

	return 1;
}
