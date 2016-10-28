#ifndef __KERNEL_STRING_H__
#define __KERNEL_STRING_H__

typedef unsigned long ksize_t;

void kmemcpy(void *dst, void *src, ksize_t n);

void kwmemcpy(void *dst, void *src, ksize_t n);

void kmemset(void *s, int c, ksize_t n);

void kwmemset(void *s, int c, ksize_t n);

int kmemcmp(void *a, void *b, ksize_t n);

// unsigned int to hex string
void kuxtos(unsigned int ux, char *out, ksize_t *n_written);

// unsigned int to decimal string
void kuitos(unsigned int ui, char *out, ksize_t *n_written);

// unsigned int to binary string
void kubtos(unsigned int ub, char *out, ksize_t *n_written);

#endif
