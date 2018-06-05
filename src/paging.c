#include "paging.h"

struct boot_info {
	multiboot_uint32_t size;
	multiboot_uint32_t reserved;
};

void paging_init_from_multiboot(int magic, void *header) {
	if (magic != MULTIBOOT2_BOOTLOADER_MAGIC) {
		// bad magic
		// TODO proper error handling
		printf("bad magic: 0x%x, expected 0x%x\n", magic, MULTIBOOT2_BOOTLOADER_MAGIC);
		halt();
	}

	struct boot_info *boot_info = (struct boot_info *) header;
	// TODO find memory tag
}
