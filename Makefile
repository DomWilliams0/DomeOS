SOURCES      = $(shell find ${KERNEL_DIR} -type f -name '*.c')
HEADERS      = $(shell find ${KERNEL_DIR} -type f -name '*.h')

BOOT_DIR     = boot/
KERNEL_DIR   = kernel/

KERNEL_BIN   = kernel.bin
OBJ          = ${SOURCES:.c=.o}

RUN_CMD      = qemu-system-x86_64 -kernel ${KERNEL_BIN} -monitor stdio
NASM_CMD     = nasm $< -felf32 -i ${BOOT_DIR} -o $@

# default
all: kernel.bin

# building
kernel.bin: ${BOOT_DIR}/multiboot.o ${OBJ}
	i686-elf-gcc -T kernel/linker.ld -ffreestanding -O0 -nostdlib -lgcc -g -o ${KERNEL_BIN} $^

tests:
	make -C testing

%.o: %.c
	i686-elf-gcc -ffreestanding -c -O0 -Wall -Wextra -o $@ $<

%.o: %.asm
	${NASM_CMD}

%.bin: %.asm
	${NASM_CMD}

clean:
	find . -name "*.o" -delete
	rm -f ${KERNEL_BIN}
	make -C testing clean

# running
run:
	${RUN_CMD}

build-run: kernel.bin
	${RUN_CMD}

tests-run:
	make -C testing run

debug:
	${RUN_CMD} -s -S
