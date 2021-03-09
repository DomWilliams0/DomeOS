CC = x86_64-w64-mingw32-gcc
CFLAGS = -ffreestanding -nostdlib -mconsole -Wl,--entry=_start

.PHONY: all
all: syscall.exe nop.exe syscall-c.exe

syscall.obj: syscall.s
	nasm $^ -f win64

syscall.exe: syscall.obj
	$(CC) $(CFLAGS) -o $@ $^

syscall-c.exe: syscall-c/syscall.c syscall-c/printf.c
	$(CC) $(CFLAGS) -o $@ $^ -Isyscall-c

nop.exe: nop.c
	$(CC) $(CFLAGS) -o $@ $^
