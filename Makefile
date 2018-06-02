SRC_DIR  = src
INC_DIR  = include
OBJ_DIR  = obj
BIN_DIR  = bin
BOOT_DIR = boot

SOURCES_C   = $(shell find ${SRC_DIR} -type f -name '*.c')
SOURCES_ASM = $(shell find ${BOOT_DIR} -type f -name '*.asm')
HEADERS     = $(shell find ${INC_DIR} -type f -name '*.h')
OBJ_C       = $(patsubst %.c, ${OBJ_DIR}/%.o, $(notdir ${SOURCES_C}))
OBJ_ASM     = $(patsubst %.asm, ${OBJ_DIR}/%.o, $(notdir ${SOURCES_ASM}))
OBJ         = $(OBJ_C) $(OBJ_ASM)

BIN_NAME = kernel.bin
BIN_PATH = ${BIN_DIR}/${BIN_NAME}

RUN_CMD  = qemu-system-x86_64 -kernel ${BIN_PATH} -monitor stdio -serial file:serial.log
NASM_CMD = nasm $< -felf32 -i ${BOOT_DIR}/ -o $@
CC_CMD   = i686-elf-gcc -ffreestanding -O0 -Wall -Wextra -Iinclude

VPATH = $(shell find ${SRC_DIR} ${INC_DIR} -type d)

# default
.PHONY: all
all: kernel.bin

# building
$(BIN_NAME): ${OBJ}
	${CC_CMD} -Tlinker.ld -nostdlib -lgcc -g -o ${BIN_PATH} ${OBJ}

$(OBJ_DIR)/%.o: %.c | build_dirs
	${CC_CMD} -c -o $@ $<

$(OBJ_DIR)/%.o: ${BOOT_DIR}/%.asm | build_dirs
	${NASM_CMD}

# phonies
.PHONY: build_dirs
build_dirs:
	@mkdir -p ${OBJ_DIR} ${BIN_DIR}

.PHONY: clean
clean:
	rm -rf ${BIN_DIR} ${OBJ_DIR}

# running
.PHONY: run-only
run-only:
	${RUN_CMD}

.PHONY: run
run: ${BIN_NAME}
	${RUN_CMD}

.PHONY: debug
debug:
	${RUN_CMD} -s -S
