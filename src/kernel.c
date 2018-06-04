#include "interrupts.h"
#include "idt.h"

void kernel_main()
{
	idt_init();
	enable_interrupts();

	// hang forever
	while (1);
}
