#include "screen.h"
#include "isr.h"

char *exceptions[] =
{
        "Division By Zero",            // 00
        "Debug",                       // 01
        "Non Maskable Interrupt",      // 02
        "Breakpoint",                  // 03
        "Into Detected Overflow",      // 04
        "Out of Bounds",               // 05
        "Invalid Opcode",              // 06
        "No Coprocessor",              // 07
        "Double Fault",                // 08
        "Coprocessor Segment Overrun", // 09
        "Bad TSS",                     // 10
        "Segment Not Present",         // 11
        "Stack Fault",                 // 12
        "General Protection Fault",    // 13
        "Page Fault",                  // 14
        "Unknown Interrupt",           // 15
        "Coprocessor Fault",           // 16
        "Alignment Check",             // 17
        "Machine Check",               // 18
        "Reserved",                    // 19
        "Reserved",                    // 20
        "Reserved",                    // 21
        "Reserved",                    // 22
        "Reserved",                    // 23
        "Reserved",                    // 24
        "Reserved",                    // 25
        "Reserved",                    // 26
        "Reserved",                    // 27
        "Reserved",                    // 28
        "Reserved",                    // 29
        "Reserved",                    // 30
        "Reserved",                    // 31
};

void fault_handler(struct stack_context *context)
{
    if (context->int_id < 32)
    {
        screen_write_string("Exception: ");
        screen_write_string(exceptions[context->int_id]);
        screen_write_string(" (err:ERROR_CODE_HERE)\nHalting.");
        while(1){}
    }
}

void enable_interrupts()
{
    __asm__ __volatile__ ("sti");
}
