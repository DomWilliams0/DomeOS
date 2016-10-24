#include "idt.h"

struct idt_entry_repr idt_entries[IDT_ENTRY_COUNT] = { 0 };
struct idt_descriptor idt_descriptor;

void idt_init()
{
	// TODO add entries

	// set descriptor pointer
	idt_descriptor.base = (uint32_t) &idt_entries;
	idt_descriptor.limit = (sizeof(struct idt_entry_repr) * IDT_ENTRY_COUNT) - 1;

	// replace existing idt with the new one
	idt_flush();
}


