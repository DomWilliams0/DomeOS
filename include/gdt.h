#ifndef __KERNEL_GDT_H__
#define __KERNEL_GDT_H__

#include <stdint.h>

#define GDT_ENTRY_COUNT 3

// low level bit representation
struct gdt_entry_repr
{
    // bottom 16 bits of limit
    uint32_t limit_low:  16;

    // bottom 24 bits of limit
    uint32_t base_low:   24;

    // access byte

    // must be 0, cpu sets to 1 when it accesses this segment
    uint32_t  accessed:  1;

    // code: read access
    // data: write access
    uint32_t  rw:        1;

    // code: conforming: if rings <= current can execute this segment
    // data: direction : if this segment grows up
    uint32_t  dir_conf:  1;

    // code: 1
    // data: 0
    uint32_t  exec:      1;

    // who knows
    uint32_t  one:       1;

    // ring level 0-3
    uint32_t  priv:      2;

    // must be 1
    uint32_t  present:   1;

    // randomly, upper part of the limit
    uint32_t limit_high: 4;

    // flags

    // debugging
    uint32_t  avl:       1;

    // 64 bit mode
    uint32_t  x86_64:    1;

    // 32 bit mode: 1
    // 16 bit mode: 0
    uint32_t  size:      1;

    // 4KiB pages (allows full 4GiB range): 1
    // 1B blocks                          : 0
    uint32_t  gran:      1;

    // upper part of base
    uint32_t base_high:  8;

}__attribute__((packed));


struct gdt_descriptor
{
    uint16_t limit;
    uint32_t base;

}__attribute__((packed));

void gdt_init();

extern void gdt_flush();

#endif
