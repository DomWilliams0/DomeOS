SRC_DIR    = src/
INC_DIR    = include/
BOOT_DIR   = ${SRC_DIR}/boot/
TEST_DIR   = tests/

SOURCES    = $(shell find ${SRC_DIR} -type f -name '*.c')
HEADERS    = $(shell find ${INC_DIR} -type f -name '*.h')

KERNEL_BIN = kernel.bin
OBJ        = ${SOURCES:.c=.o}

RUN_CMD    = qemu-system-x86_64 -kernel ${KERNEL_BIN} -monitor stdio
NASM_CMD   = nasm $< -felf32 -i ${BOOT_DIR} -o $@
CC_CMD     = i686-elf-gcc -ffreestanding -O0 -Wall -Wextra -Iinclude

# default
all: kernel.bin

# building
kernel.bin: ${BOOT_DIR}/multiboot.o ${OBJ}
	${CC_CMD} -Tlinker.ld -nostdlib -lgcc -g -o ${KERNEL_BIN} $^

tests:
	make -C ${TEST_DIR}

%.o: %.c
	${CC_CMD} -c -o $@ $<

%.o: %.asm
	${NASM_CMD}

%.bin: %.asm
	${NASM_CMD}

clean:
	find ${SRC_DIR} -name "*.o" -delete
	rm -f ${KERNEL_BIN}
	make -C ${TEST_DIR} clean

# running
run:
	${RUN_CMD}

build-run: kernel.bin
	${RUN_CMD}

tests-run:
	make -C ${TEST_DIR} run

debug:
	${RUN_CMD} -s -S
