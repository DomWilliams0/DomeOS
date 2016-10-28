#include "util/string.h"

#define _UINT_TO_STR(func_name, max_buf_len, base, prefix, prefix_len)\
    void func_name(unsigned int ux, char *out, ksize_t *n_written)\
    {\
        char digits[] = "0123456789ABCDEF";\
        \
        ksize_t written = 0;\
        char arr[max_buf_len];\
        \
        unsigned int number = ux;\
        do\
        {\
            arr[written++] = digits[number % base];\
            number /= base;\
        }\
        while (number > 0);\
        \
        for (unsigned int i = 0; i < written; ++i)\
            out[prefix_len + i] = arr[written - i - 1];\
        \
        for (unsigned int i = 0; i < prefix_len; ++i)\
            out[i] = prefix[i];\
        \
        written += prefix_len;\
        \
        out[written] = '\0';\
        *n_written = written;\
    }

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

_UINT_TO_STR(kuxtos, MAX_UINT_HEX_STRING_DIGITS, 16, "0x", 2)
_UINT_TO_STR(kuitos, MAX_UINT_DEC_STRING_DIGITS, 10, "", 0)
_UINT_TO_STR(kubtos, MAX_UINT_BIN_STRING_DIGITS, 2, "0b", 2)
