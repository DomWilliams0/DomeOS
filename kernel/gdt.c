#include "gdt.h"

// high level structure agnostic description
struct gdt_entry_desc
{
    uint32_t base;
    uint32_t limit;

    uint8_t  present;
    uint8_t  priv;
    uint8_t  exec;
    uint8_t  dir_conf;
    uint8_t  rw;
    uint8_t  gran;
    uint8_t  size;

};

struct gdt_entry_repr gdt_entries[GDT_ENTRY_COUNT];
struct gdt_descriptor gdt_descriptor;

static void register_entry(int index, struct gdt_entry_desc *desc)
{
    struct gdt_entry_repr *repr = &gdt_entries[index];

    repr->base_low   = desc->base & 0xFFFFFF;
    repr->base_high  = (desc->base >> 24) & 0xFF;

    repr->limit_low  = desc->limit & 0xFFFF;
    repr->limit_high = (desc->limit >> 16) & 0x0F;

    repr->present    = desc->present;
    repr->priv       = desc->priv;
    repr->one        = 1;
    repr->exec       = desc->exec;
    repr->dir_conf   = desc->dir_conf;
    repr->rw         = desc->rw;
    repr->accessed   = 0;
    repr->gran       = desc->gran;
    repr->size       = desc->size;
    repr->x86_64     = 0;
    repr->avl        = 0;
}

void gdt_init()
{
    struct gdt_entry_desc entry_null,
                          entry_code,
                          entry_data;

    // null entry
    entry_null = (struct gdt_entry_desc) { 0 };


    // code segment
    entry_code = (struct gdt_entry_desc)
    {
        .base     = 0x0,
        .limit    = 0xfffff,

        .present  = 1,
        .priv     = 0, // ring 0
        .exec     = 1, // code
        .dir_conf = 0, // not conforming
        .rw       = 0, // not readable
        .gran     = 1, // 4k pages
        .size     = 1  // 32 bit
    };


    // data segment
    entry_data = (struct gdt_entry_desc)
    {
        .base     = 0x0,
        .limit    = 0xfffff,

        .present  = 1,
        .priv     = 0, // ring 0
        .exec     = 0, // data
        .dir_conf = 0, // expand downwards
        .rw       = 0, // not writable
        .gran     = 1, // 4k pages
        .size     = 1  // 32 bit
    };


    // register in table
    register_entry(0, &entry_null);
    register_entry(1, &entry_code);
    register_entry(2, &entry_data);

    // set descriptor pointer
    gdt_descriptor.base  = (uint32_t) &gdt_entries;
    gdt_descriptor.limit = (sizeof(struct gdt_entry_repr) * GDT_ENTRY_COUNT) - 1;

    // replace existing gdt with the new one
    gdt_flush();
}
