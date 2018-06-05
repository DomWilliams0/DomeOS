#include <stdint.h>

#include "interrupts.h"
#include "kernel.h"
#include "screen.h"
#include "printf.h"

static char *exceptions[] =
{
	"Division By Zero",            // 00
	"Debug",                       // 01
	"Non Maskable Interrupt",      // 02
	"Breakpoint",                  // 03
	"Into Detected Overflow",      // 04
	"Out of Bounds",               // 05
	"Invalid Opcode",              // 06
	"No Coprocessor",              // 07
	"Double Fault",                // 08
	"Coprocessor Segment Overrun", // 09
	"Bad TSS",                     // 10
	"Segment Not Present",         // 11
	"Stack Fault",                 // 12
	"General Protection Fault",    // 13
	"Page Fault",                  // 14
	"Unknown Interrupt",           // 15
	"Coprocessor Fault",           // 16
	"Alignment Check",             // 17
	"Machine Check",               // 18
	"Reserved",                    // 19
	"Reserved",                    // 20
	"Reserved",                    // 21
	"Reserved",                    // 22
	"Reserved",                    // 23
	"Reserved",                    // 24
	"Reserved",                    // 25
	"Reserved",                    // 26
	"Reserved",                    // 27
	"Reserved",                    // 28
	"Reserved",                    // 29
	"Reserved",                    // 30
	"Reserved",                    // 31
};

static void log_exception(int int_no, int err) {
	switch (int_no) {
		case 13: {
					 struct {
						 int external: 1;
						 int tbl: 2;
						 int selector: 13;
					 }__attribute__((packed)) *selector = (void *)&err;
					 printf("external=%d, tbl=%d, selector=0x%x\n", selector->external, selector->tbl, selector->selector);
				 }



		default: break;
	}

}


// log and never return
// TODO print all registers
void fault_handler(struct intr_context *ctx)
{
	if (ctx->int_no < 32)
	{
		// TODO generalise this error logging and use in kernel.c too
		printf("\n=======\n");
		screen_set_colours(SCREEN_COLOUR_WHITE, SCREEN_COLOUR_RED);
		printf("Unhandled exception %d: %s\nError code: %d\n", ctx->int_no, exceptions[ctx->int_no], ctx->err_code);
		log_exception(ctx->int_no, ctx->err_code);
		printf("Halting\n");

		log_registers(ctx);
		halt();
	}
}

void enable_interrupts() {
	__asm__ __volatile__ ("sti");
}

void disable_interrupts() {
	__asm__ __volatile__ ("cli");
}
