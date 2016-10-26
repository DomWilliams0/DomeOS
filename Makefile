SOURCES      = $(shell find ${KERNEL_DIR} -type f -name '*.c')
HEADERS      = $(shell find ${KERNEL_DIR} -type f -name '*.h')

TEST_SOURCES = $(shell find ${TESTS_DIR} -type f -name '*.c')
TEST_HEADERS = $(shell find ${TESTS_DIR} -type f -name '*.h')

BOOT_DIR     = boot/
KERNEL_DIR   = kernel/
TESTS_DIR    = testing/

KERNEL_BIN   = kernel.bin
TESTS_BIN    = kernel_tests

OBJ          = ${SOURCES:.c=.o}
TEST_OBJ     = ${TEST_SOURCES:.c=.o}

RUN_CMD      = qemu-system-x86_64 -kernel ${KERNEL_BIN} -monitor stdio
NASM_CMD     = nasm $< -felf32 -i ${BOOT_DIR} -o $@

# default
all: kernel.bin

# building
kernel.bin: ${BOOT_DIR}/multiboot.o ${OBJ}
	i686-elf-gcc -T kernel/linker.ld -ffreestanding -O0 -nostdlib -lgcc -g -o ${KERNEL_BIN} $^

tests: kernel.bin ${TEST_SOURCES} ${TEST_HEADERS}
	gcc -O0 -Wall -Wextra -o ${TESTS_BIN} ${TEST_SOURCES} ${TEST_HEADERS}

%.o: %.c ${HEADERS}
	i686-elf-gcc -ffreestanding -c -O0 -Wall -Wextra -o $@ $<

%.o: %.asm
	${NASM_CMD}

%.bin: %.asm
	${NASM_CMD}

clean:
	find . -name "*.o" -delete
	rm -f ${KERNEL_BIN} ${TESTS_BIN}

# running
run:
	${RUN_CMD}

build-run: kernel.bin
	${RUN_CMD}

tests-run: tests
	./${TESTS_BIN}

debug:
	${RUN_CMD} -s -S
