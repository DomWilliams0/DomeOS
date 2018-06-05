#ifndef DOMEOS_INTERRUPTST_H
#define DOMEOS_INTERRUPTST_H

#include <stdint.h>

struct intr_context {
	uint64_t rax;
	uint64_t rbx;
	uint64_t rcx;
	uint64_t rdx;
	uint64_t rsi;
	uint64_t rdi;
	uint64_t rbp;

	uint64_t r8;
	uint64_t r9;
	uint64_t r10;
	uint64_t r11;
	uint64_t r12;
	uint64_t r13;
	uint64_t r14;
	uint64_t r15;

	uint64_t int_no;
	uint64_t err_code;

	// pushed by CPU
	uint64_t rip;
	uint64_t cs;
	uint64_t rflags;
	uint64_t rsp;
	uint64_t ss;
};

void fault_handler(struct intr_context *ctx);

void irq_handler(struct intr_context *ctx);

void enable_interrupts();

void disable_interrupts();

#endif
