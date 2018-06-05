#include "irq.h"
#include "io.h"
#include "interrupts.h"
#include "printf.h"

void *irq_handlers[IRQ_HANDLER_COUNT] = { 0 };

void irq_register_handler(uint32_t irq, void (*handler)(struct intr_context *))
{
	if (irq < IRQ_HANDLER_COUNT)
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

	// keyboard and clock enabled only for now
	io_write_port(PIC_MASTER_DATA,    0xFC);
	io_write_port(PIC_SLAVE_DATA,     0xFF);
}

void irq_handler(struct intr_context *ctx)
{
	// find handler
	irq_handler_func handler = irq_handlers[ctx->int_no - 32];
	if (handler)
		handler(ctx);

	// confirm success with slave controller, if necessary
	if (ctx->int_no >= 40)
	{
		io_write_port(PIC_SLAVE_COMMAND, PIC_END_OF_INTERRUPT);
	}

	// confirm success with master controller
	io_write_port(PIC_MASTER_COMMAND, PIC_END_OF_INTERRUPT);
}
