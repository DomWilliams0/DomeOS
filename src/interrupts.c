#include <stdint.h>

void fault_handler(struct intr_context *ctx) {
	while (1);
}

void irq_handler(struct intr_context *ctx) {

}

void enable_interrupts() {
	__asm__ __volatile__ ("sti");
}

void disable_interrupts() {
	__asm__ __volatile__ ("cli");
}
