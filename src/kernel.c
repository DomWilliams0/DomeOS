#include <stdnoreturn.h>

#include "screen.h"
#include "gdt.h"
#include "idt.h"
#include "isr.h"
#include "clock.h"
#include "serial.h"

#include "io.h"
#include "logging.h"
#include "string.h"
#include "util.h"
#include "boot/multiboot.h"

static noreturn void panic(char *msg) {
	screen_write_string("--- PANIC ---\n");
	screen_write_string(msg);

	// disable interrupts
	__asm__ __volatile__ ("cli");

	// hang
	while (1);
}

static void discover_memory(multiboot_info_t *mbinfo)
{
	if (((mbinfo->flags >> 6) & 1) == 0)
	{
	  panic("Bad multiboot flag");
  }

	memory_map_t *map = (memory_map_t *)mbinfo->mmap_addr;
	memory_map_t *map_end = mbinfo->mmap_addr + mbinfo->mmap_length;

	while (map < map_end)
	{
	  if (map->type == 1)
	  {
		// usable!
		unsigned long addr = map->base_addr_high << 32 | map->base_addr_low;
		unsigned long len = map->length_high << 32 | map->length_low;

		log_raw("found some memory (addr, len):");
		char buf[40];
		itoa(addr, buf, 16);
		log_raw(buf);
		log_raw(" | ");
		itoa(len, buf, 10);
		log_raw(buf);
		log_raw("\n");

	}
	  else LOG_DEBUG("unusable memory");

	  // skip to next
	  map += map->size + sizeof(map->size);
  }
}

void kernel_main(multiboot_info_t *mbinfo, unsigned int magic)
{
	serial_init();
	gdt_init();
	idt_init();
	clock_init();
	enable_interrupts();
	screen_init(SCREEN_COLOUR_WHITE, SCREEN_COLOUR_BLACK);

	discover_memory(mbinfo);

	char *test_string = "This is a line of text that fills up a row exactly, what are the chances ?!!!!!!";
	for (int i = 0; i < 5; ++i)
	{
	  test_string[0] = '1' + i;
	  screen_write_string(test_string);
  }

	// hang forever
	while (1);
}
