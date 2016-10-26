#ifndef __KERNEL_CLOCK_H__
#define __KERNEL_CLOCK_H__

#define PIT_CHANNEL0_DATA 0x40
#define PIT_CHANNEL2_DATA 0x42
#define PIT_COMMAND       0x43

#include <stdint.h>

#define CLOCK_HERTZ 100

void clock_set_interval(int hz);

void clock_init();

#endif
