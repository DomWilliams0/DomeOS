#ifndef __KERNEL_CLOCK_H__
#define __KERNEL_CLOCK_H__

#define PIT_CHANNEL0_DATA 0x40
#define PIT_CHANNEL2_DATA 0x42
#define PIT_COMMAND       0x43

#include <stdint.h>

#define CLOCK_HERTZ 100

struct pit_command_repr
{
    // bcd: 0 - 9999, disgusting
    uint8_t bcd:     1;

    // operating mode
    // e.g. 011: square wave
    uint8_t mode:    3;

    // access mode
    // lobyte only, hibyte only, lobyte + hibyte
    uint8_t rw:      2;

    // channel 0 - 2
    uint8_t channel: 2;
}__attribute__((packed));

void clock_set_interval(int hz);

void clock_init();

#endif
