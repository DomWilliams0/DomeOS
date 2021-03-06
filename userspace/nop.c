
void do_illegal() {
	asm("cli");
}

int g_things_ro[100] = {1};

int g_things[100] = {3};

int _start() {
	int things[100] = {2};
	for (int i = 0; i < 100; i++) {
		things[i] = 0x55555555;
		g_things[i] = 0x66666666;
	}

	//do_illegal();

	while (1);
}
