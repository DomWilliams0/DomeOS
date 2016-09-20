SOURCES     = $(wildcard kernel/*.c kernel/util/*.c)
HEADERS     = $(wildcard kernel/*.h kernel/util/*.h)

KERNEL_BIN  = kernel.bin
OBJ         = ${SOURCES:.c=.o}

RUN_CMD     = qemu-system-x86_64 -kernel ${KERNEL_BIN}

# default
all: kernel.bin

# building
kernel.bin: boot/multiboot.o ${OBJ}
	i686-elf-gcc -T kernel/linker.ld -ffreestanding -O2 -nostdlib -lgcc -g -o ${KERNEL_BIN} $^

%.o: %.c ${HEADERS}
	i686-elf-gcc -ffreestanding -c -O2 -Wall -Wextra -o $@ $< 

%.o: %.asm
	nasm $< -felf32 -o $@

%.bin: %.asm
	nasm $< -felf32 -o $@

clean:
	rm -fr *.bin *.o kernel/*.o boot/*.o

# running
run:
	${RUN_CMD}

build-run: kernel.bin
	${RUN_CMD}

debug:
	${RUN_CMD} -s -S
