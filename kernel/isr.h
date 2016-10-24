#ifndef __KERNEL_ISR_H__
#define __KERNEL_ISR_H__

#include <stdint.h>

struct stack_context
{
    // segments
    uint32_t gs;
    uint32_t fs;
    uint32_t es;
    uint32_t ds;

    // pusha
    uint32_t edi;
    uint32_t esi;
    uint32_t ebp;
    uint32_t esp;

    uint32_t ebx;
    uint32_t edx;
    uint32_t ecx;
    uint32_t eax;

    // pushed by isr label
    uint32_t int_id;
    uint32_t err;

    // pushed by processor
    uint32_t eip;
    uint32_t cs;
    uint32_t eflags;
    uint32_t useresp;
    uint32_t ss;
};

void fault_handler(struct stack_context *context);

#endif
