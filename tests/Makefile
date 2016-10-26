TEST_SOURCES   = $(shell find . -type f -name '*.c')
TEST_OBJ       = ${TEST_SOURCES:.c=.o}

KERNEL_SOURCES = $(shell find ../src -type f -name '*.c')
KERNEL_OBJ     = ${KERNEL_SOURCES:.c=.o}

TESTS_BIN      = tests.bin
CC_CMD = gcc -O0 -Wall -I../include

all: run

$(TESTS_BIN): ${TEST_OBJ} ${KERNEL_OBJ}
	${CC_CMD} -o $@ $^

%.o: %.c
	${CC_CMD} -c -o $@ $<

run: ${TESTS_BIN}
	./${TESTS_BIN}

clean:
	find . -name "*.o" -delete
	rm -f ${TESTS_BIN}
