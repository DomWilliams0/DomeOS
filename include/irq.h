#ifndef __KERNEL_IRQ_H__
#define __KERNEL_IRQ_H__

#include <stdint.h>

#define IRQ_HANDLER_COUNT 16

#define PIC_MASTER_COMMAND   0x20
#define PIC_MASTER_DATA      0x21
#define PIC_SLAVE_COMMAND    0xA0
#define PIC_SLAVE_DATA       0xA1
#define PIC_END_OF_INTERRUPT 0x20

struct stack_context;
typedef void (*irq_handler_func)(struct stack_context *);

void irq_register_handler(uint32_t irq, irq_handler_func handler);

void irq_remap();

void irq_handler(struct stack_context *context);

#endif
