SOURCES     = $(shell find kernel -type f -name '*.c')
HEADERS     = $(shell find kernel -type f -name '*.h')

BOOT_DIR    = boot/
KERNEL_DIR  = kernel/

KERNEL_BIN  = kernel.bin
OBJ         = ${SOURCES:.c=.o}

RUN_CMD     = qemu-system-x86_64 -kernel ${KERNEL_BIN} -monitor stdio
NASM_CMD    = nasm $< -felf32 -i ${BOOT_DIR} -o $@

# default
all: kernel.bin

# building
kernel.bin: ${BOOT_DIR}/multiboot.o ${OBJ}
	i686-elf-gcc -T kernel/linker.ld -ffreestanding -O2 -nostdlib -lgcc -g -o ${KERNEL_BIN} $^

%.o: %.c ${HEADERS}
	i686-elf-gcc -ffreestanding -c -O2 -Wall -Wextra -o $@ $< 

%.o: %.asm
	${NASM_CMD}

%.bin: %.asm
	${NASM_CMD}

clean:
	rm -fr *.bin *.o kernel/*.o boot/*.o

# running
run:
	${RUN_CMD}

build-run: kernel.bin
	${RUN_CMD}

debug:
	${RUN_CMD} -s -S
