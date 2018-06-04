#include "idt.h"

extern void isr0();
extern void isr1();
extern void isr2();
extern void isr3();
extern void isr4();
extern void isr5();
extern void isr6();
extern void isr7();
extern void isr8();
extern void isr9();
extern void isr10();
extern void isr11();
extern void isr12();
extern void isr13();
extern void isr14();
extern void isr15();
extern void isr16();
extern void isr17();
extern void isr18();
extern void isr19();
extern void isr20();
extern void isr21();
extern void isr22();
extern void isr23();
extern void isr24();
extern void isr25();
extern void isr26();
extern void isr27();
extern void isr28();
extern void isr29();
extern void isr30();
extern void isr31();

extern void irq0();
extern void irq1();
extern void irq2();
extern void irq3();
extern void irq4();
extern void irq5();
extern void irq6();
extern void irq7();
extern void irq8();
extern void irq9();
extern void irq10();
extern void irq11();
extern void irq12();
extern void irq13();
extern void irq14();
extern void irq15();

struct idt_entry_repr idt_entries[IDT_ENTRY_COUNT] = { 0 };
struct idt_descriptor idt_descriptor;

static void register_entry(int index, void (*handler)(void))
{
	static struct idt_entry_repr common_entry = {
		.rpl              = 0,   // ring 0
		.ring             = 0,

		.ti               = 0,   // gdt
		.descriptor_index = 1,   // code segment
		.zero             = 0,   // okie dokie
		.gate_type        = 0xE, // 32 bit interrupt gate
		.storage_segment  = 0,   // interrupt gate
		.present          = 1    // present
	};

	uint64_t addr = (uint64_t) handler;
	common_entry.base_low  = (uint16_t) addr;
	common_entry.base_mid  = (uint16_t) (addr >> 16);
	common_entry.base_high = (uint32_t) (addr >> 32);

	idt_entries[index] = common_entry;
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

	/*
	register_entry(32, irq0);
	register_entry(33, irq1);
	register_entry(34, irq2);
	register_entry(35, irq3);
	register_entry(36, irq4);
	register_entry(37, irq5);
	register_entry(38, irq6);
	register_entry(39, irq7);
	register_entry(40, irq8);
	register_entry(41, irq9);
	register_entry(42, irq10);
	register_entry(43, irq11);
	register_entry(44, irq12);
	register_entry(45, irq13);
	register_entry(46, irq14);
	register_entry(47, irq15);
	*/
}

void idt_init()
{
	// TODO enable irqs
	// shift irqs to avoid collisions
	// irq_remap();

	// register isrs and irqs
	register_all_entries();

	// set descriptor pointer
	idt_descriptor.base = (uint64_t) &idt_entries;
	idt_descriptor.limit = (sizeof(struct idt_entry_repr) * IDT_ENTRY_COUNT) - 1;

	// replace existing idt with the new one
	idt_flush();
}



