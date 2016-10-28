#include "util/string.h"

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
		if (*c++ != *d++)
			return 0;

	return 1;
}

void kuxtos(unsigned int ux, char *out, ksize_t *n_written)
{
    static char digits[] = "0123456789ABCDEF";
    const int base = 16;

    ksize_t written = 0;
    char arr[8]; // max hex char

    unsigned int number = ux;
    do
    {
        arr[written++] = digits[number % base];
        number /= base;
    }
    while (number > 0);

    out[0] = '0';
    out[1] = 'x';

    for (unsigned int i = 0; i < written; ++i)
        out[2 + i] = arr[written - i - 1];

    // 0x prefix
    written += 2;

    out[written] = '\0';

    *n_written = written;
}
