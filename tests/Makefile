TEST_DIR = tests/
OBJ_DIR  = obj/
BIN_DIR  = bin/
KSRC_DIR = ../src

BIN_NAME = tests
BIN_PATH = ${BIN_DIR}/${BIN_NAME}

K_SRC    = $(shell find ${KSRC_DIR} -type f -name "*.c")

SRC      = $(shell find . -type f -name '*.c')
OBJ      = $(patsubst %.c, ${OBJ_DIR}/%.o, $(notdir ${SRC} ${K_SRC}))

CC_CMD   = gcc -O0 -Wall -I../include

VPATH = $(shell find ${KSRC_DIR} -type d)

all: run

.PHONY: create_dirs

create_dirs:
	@mkdir -p ${OBJ_DIR} ${BIN_DIR}

build_kernel: ;

$(BIN_NAME): create_dirs build_kernel ${OBJ}
	${CC_CMD} -o ${BIN_PATH} ${OBJ}

$(OBJ_DIR)/%.o: %.c
	${CC_CMD} -c -o $@ $<

run: ${BIN_NAME}
	./${BIN_PATH}

clean:
	rm -rfv ${BIN_DIR} ${OBJ_DIR}
