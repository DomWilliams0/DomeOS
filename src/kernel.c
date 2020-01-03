#include "interrupts.h"
#include "idt.h"
#include "screen.h"
#include "multiboot.h"

#include "printf.h"

void halt()
{
	disable_interrupts();
	while(1);
}

void log_registers(struct intr_context *ctx)
{
	printf("%s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"\n%s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"\n%s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"\n%s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"\n%s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"  %s: 0x%x"
			"\n"
			, "rax", ctx->rax, "rbx", ctx->rbx, "rcx", ctx->rcx
			, "rdx", ctx->rdx, "rsi", ctx->rsi, "rdi", ctx->rdi
			,"rbp", ctx->rbp , "r8", ctx->r8, "r9", ctx->r9
			,"r10", ctx->r10 , "r11", ctx->r11, "r12", ctx->r12
			,"r13", ctx->r13 , "r14", ctx->r14, "r15", ctx->r15
			,"rip", ctx->rip , "rflags", ctx->rflags, "rsp", ctx->rsp
			,"ss", ctx->ss
			);
}



void kernel_main(int multiboot_magic, void *multiboot_header)
{
	screen_init(SCREEN_COLOUR_LIGHT_GREEN, SCREEN_COLOUR_DARK_GREY);
	printf("Booting\n");

    (void) multiboot_magic;
    (void) multiboot_header;
/* TODO
	if (parse_multiboot(multiboot_magic, multiboot_header) != 0) {
		printf("Failed to parse multiboot header, halting\n");
		halt();
		return;
	}
*/

	idt_init();
	enable_interrupts();

	printf("nothing to do, hanging\n");

	// hang forever
	while (1) {
        __asm__ __volatile__ ("hlt");
	}
}
