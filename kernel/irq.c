#include "isr.h"
#include "irq.h"
#include "util/io.h"

void *irq_handlers[IRQ_HANDLER_COUNT] = { 0 };

void irq_register_handler(uint32_t irq, void (*handler)(struct stack_context *))
{
    irq_handlers[irq] = handler;
}

// remaps irqs from interrupt 8-15 to 32-47
void irq_remap()
{
    io_write_port(PIC_MASTER_COMMAND, 0x11);
    io_write_port(PIC_SLAVE_COMMAND,  0x11);

    io_write_port(PIC_MASTER_DATA,    0x20);
    io_write_port(PIC_SLAVE_DATA,     0x28);

    io_write_port(PIC_MASTER_DATA,    0x04);
    io_write_port(PIC_SLAVE_DATA,     0x02);

    io_write_port(PIC_MASTER_DATA,    0x01);
    io_write_port(PIC_SLAVE_DATA,     0x01);

    io_write_port(PIC_MASTER_DATA,    0x0);
    io_write_port(PIC_SLAVE_DATA,     0x0);
}

void irq_handler(struct stack_context *context)
{
    // find handler
    irq_handler_func handler = irq_handlers[context->int_id - 32];
    if (handler)
    {
        handler(context);
    }

    // confirm success with slave controller, if necessary
    if (context->int_id >= 40)
    {
        io_write_port(PIC_SLAVE_COMMAND, PIC_END_OF_INTERRUPT);
    }

    // confirm success with master controller
    io_write_port(PIC_MASTER_COMMAND, PIC_END_OF_INTERRUPT);
}
