#include "idt.h"

struct idt_entry_repr idt_entries[IDT_ENTRY_COUNT] = { 0 };
struct idt_descriptor idt_descriptor;

static void register_entry(int index, void (*handler)(void))
{
    struct idt_entry_repr entry = {
        .base_low         = (uint32_t) (handler) & 0xFFFF,
        .base_high        = ((uint32_t) (handler) >> 16) & 0xFFFF,

        .rpl              = 0,   // ring 0
        .ring             = 0,   // ring 0

        .ti               = 0,   // gdt
        .descriptor_index = 1,   // code segment,
        .zero             = 0,   // okie dokie
        .gate_type        = 0xE, // 32 bit interrupt gate
        .storage_segment  = 0,   // interrupt gate
        .present          = 1    // used
    };

    idt_entries[index] = entry;
}

static void register_all_entries()
{
    register_entry(0,  isr0);
    register_entry(1,  isr1);
    register_entry(2,  isr2);
    register_entry(3,  isr3);
    register_entry(4,  isr4);
    register_entry(5,  isr5);
    register_entry(6,  isr6);
    register_entry(7,  isr7);
    register_entry(8,  isr8);
    register_entry(9,  isr9);
    register_entry(10, isr10);
    register_entry(11, isr11);
    register_entry(12, isr12);
    register_entry(13, isr13);
    register_entry(14, isr14);
    register_entry(15, isr15);
    register_entry(16, isr16);
    register_entry(17, isr17);
    register_entry(18, isr18);
    register_entry(19, isr19);
    register_entry(20, isr20);
    register_entry(21, isr21);
    register_entry(22, isr22);
    register_entry(23, isr23);
    register_entry(24, isr24);
    register_entry(25, isr25);
    register_entry(26, isr26);
    register_entry(27, isr27);
    register_entry(28, isr28);
    register_entry(29, isr29);
    register_entry(30, isr30);
    register_entry(31, isr31);
}

void idt_init()
{
    // register isrs
    register_all_entries();

	// set descriptor pointer
	idt_descriptor.base = (uint32_t) &idt_entries;
	idt_descriptor.limit = (sizeof(struct idt_entry_repr) * IDT_ENTRY_COUNT) - 1;

	// replace existing idt with the new one
	idt_flush();
}


