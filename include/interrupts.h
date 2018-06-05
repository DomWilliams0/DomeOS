#ifndef DOMEOS_INTERRUPTS_H
#define DOMEOS_INTERRUPTS_H

#include "kernel.h"

void fault_handler(struct intr_context *ctx);

void enable_interrupts();

void disable_interrupts();

#endif
