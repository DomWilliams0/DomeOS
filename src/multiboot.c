#include "multiboot2.h"
#include "printf.h"
#include "kernel.h"

struct boot_info
{
	multiboot_uint32_t size;
	multiboot_uint32_t reserved;
};


int parse_multiboot(int multiboot_magic,
                    void *multiboot_header)
{
	return 1;
}
