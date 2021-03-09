
#include "printf.h"

void _putchar(char character)
{
    // unused
}

int _start() {
    // extract pid from rdi
    unsigned long long pid;
    asm volatile("" : "=D"(pid)::);

    char msg[128];

    for (int i = 0; ; ++i) {

        sprintf(msg, "pid %llu says hello #%d", pid, i);

        int ret;
        asm volatile
        (
        "syscall"
        : "=a" (ret)
        : "a"(0), "D"(msg), "S"(128)
        : "memory"
        );
    }


    return 0;
}