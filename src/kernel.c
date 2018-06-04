#include "interrupts.h"
#include "idt.h"
#include "screen.h"

void kernel_main()
{
	screen_init(SCREEN_COLOUR_LIGHT_GREEN, SCREEN_COLOUR_DARK_GREY);
	screen_write_string("hullo there");

	idt_init();
	enable_interrupts();

	// hang forever
	while (1);
}
