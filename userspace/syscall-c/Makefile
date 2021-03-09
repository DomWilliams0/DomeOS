syscall-c.exe: syscall.obj
	$(CC) $(CFLAGS) -o $@ $^

syscall.obj: syscall.s
	nasm $^ -f win64

nop.exe: nop.c
	$(CC) $(CFLAGS) -o $@ $^
