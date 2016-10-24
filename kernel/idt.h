#ifndef __KERNEL_IDT_H__
#define __KERNEL_IDT_H__

#include <stdint.h>

#define IDT_ENTRY_COUNT 256

// low level bit representation
struct idt_entry_repr
{
    // bottom 16 bits of offset
    uint32_t base_low:         16;

    // selector

    // requested privilege level
    uint8_t rpl:               2;

    // table index
    // gdt: 0
    // ldt: 1
    uint8_t ti:                1;

    // descriptor index in selected table
    uint16_t descriptor_index: 13;

    // if you say so
    uint16_t zero:             8;

    // flags byte

    // 1110 for 32 bit interrupt gates
    uint8_t gate_type:         4;

    // zero for interrupt gates
    uint8_t storage_segment:   1;

    // ring 0 - 3
    uint8_t ring:              2;

    // present/used
    uint8_t present:           1;

    // upper 16 bits of base
    uint32_t base_high:        16;

}__attribute__((packed));

struct idt_descriptor
{
    uint16_t limit;
    uint32_t base;
}__attribute__((packed));

void idt_init();

extern void idt_flush();

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

#endif
