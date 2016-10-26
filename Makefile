SRC_DIR  = src/
INC_DIR  = include/
OBJ_DIR  = obj/
BIN_DIR  = bin/

BOOT_DIR = ${SRC_DIR}/boot/
TEST_DIR = tests/

SOURCES  = $(shell find ${SRC_DIR} -type f -name '*.c')
HEADERS  = $(shell find ${INC_DIR} -type f -name '*.h')
OBJ      = $(patsubst %.c, ${OBJ_DIR}/%.o, $(notdir ${SOURCES})) ${OBJ_DIR}/multiboot.o

BIN_NAME = kernel.bin
BIN_PATH = ${BIN_DIR}/${BIN_NAME}

RUN_CMD  = qemu-system-x86_64 -kernel ${BIN_PATH} -monitor stdio
NASM_CMD = nasm $< -felf32 -i ${BOOT_DIR} -o $@
CC_CMD   = i686-elf-gcc -ffreestanding -O0 -Wall -Wextra -Iinclude

VPATH = $(shell find ${SRC_DIR} ${INC_DIR} -type d)

# default
all: kernel.bin

# building
.PHONY: create_dirs

create_dirs:
	@mkdir -p ${OBJ_DIR} ${BIN_DIR}

$(BIN_NAME): create_dirs ${OBJ}
	${CC_CMD} -Tlinker.ld -nostdlib -lgcc -g -o ${BIN_PATH} ${OBJ}

.PHONY: tests

tests:
	make -C ${TEST_DIR} run

$(OBJ_DIR)/%.o: %.c
	${CC_CMD} -c -o $@ $<

$(OBJ_DIR)/%.o: ${BOOT_DIR}/%.asm
	${NASM_CMD}

%.bin: %.asm
	${NASM_CMD}

clean:
	rm -rfv ${BIN_DIR} ${OBJ_DIR}

# running
run:
	${RUN_CMD}

build-run: ${BIN_NAME}
	${RUN_CMD}

debug:
	${RUN_CMD} -s -S
