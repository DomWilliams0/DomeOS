#ifndef DOMEOS_PAGING_H
#define DOMEOS_PAGING_H

#include "kernel.h"
#include "multiboot2.h"
#include "printf.h"

void paging_init_from_multiboot(int magic, void *header);

#endif
