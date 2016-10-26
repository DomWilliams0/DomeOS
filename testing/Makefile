TEST_SOURCES   = $(shell find . -type f -name '*.c')
TEST_OBJ       = ${TEST_SOURCES:.c=.o}

KERNEL_SOURCES = $(shell find ../kernel -type f -name '*.c')
KERNEL_OBJ     = ${KERNEL_SOURCES:.c=.o}

TESTS_BIN      = tests.bin

all: $(TESTS_BIN)

$(TESTS_BIN): ${TEST_OBJ} ${KERNEL_OBJ}
	gcc -O0 -Wall -o $@ $^

%.o: %.c
	gcc -c -O0 -Wall -o $@ $<

run: ${TESTS_BIN}
	./${TESTS_BIN}

clean:
	find . -name "*.o" -delete
	rm -f ${TESTS_BIN}
