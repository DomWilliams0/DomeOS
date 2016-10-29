#ifndef __KERNEL_UTIL_H__
#define __KERNEL_UTIL_H__

#define UNUSED(x) (void)(x)

#define PRINT_INT(i)\
    do \
    {\
        char int_print[32];\
        ksize_t int_print_count = 0;\
        kuitos(i, int_print, &int_print_count);\
        kputs(int_print);\
\
    } while(0);

#endif
