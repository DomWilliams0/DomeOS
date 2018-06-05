#include "interrupts.h"
#include "idt.h"
#include "screen.h"

#include "printf.h"

void kernel_main()
{
	screen_init(SCREEN_COLOUR_LIGHT_GREEN, SCREEN_COLOUR_DARK_GREY);
	printf("hullo there lad");

	idt_init();
	enable_interrupts();

	// hang forever
	while (1);
}
