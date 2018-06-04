#ifndef DOMEOS_IDT_H
#define DOMEOS_IDT_H

#include <stdint.h>

#define IDT_ENTRY_COUNT 256

// low level bit representation
struct idt_entry_repr
{
	// bottom 16 bits of offset
	uint16_t base_low;

	// selector

	// requested privilege level
	uint32_t rpl:              2;

	// table index
	// gdt: 0
	// ldt: 1
	uint32_t ti:               1;

	// descriptor index in selected table
	uint32_t descriptor_index: 13;

	// if you say so
	uint8_t zero;

	// flags byte

	// 1110 for 32 bit interrupt gates
	uint32_t gate_type:        4;

	// zero for interrupt gates
	uint32_t storage_segment:  1;

	// ring 0 - 3
	uint32_t ring:             2;

	// present/used
	uint32_t present:          1;

	// middle 16 bits of base
	uint16_t base_mid:         16;

	// upper 32 bits
	uint32_t base_high;

	// more reserved
	uint32_t zero_more;

}__attribute__((packed));

struct idt_descriptor
{
	uint16_t limit;
	uint64_t base;
}__attribute__((packed));

void idt_init();

extern void idt_flush();

#endif
