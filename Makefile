SRC_DIR  = src
INC_DIR  = include
OBJ_DIR  = obj
BIN_DIR  = bin
BOOT_DIR = boot

ISO_DIR  = $(BIN_DIR)/isofiles
GRUB_DIR = $(ISO_DIR)/boot/grub
GRUB_CFG = $(GRUB_DIR)/grub.cfg


SOURCES_C   = $(shell find $(SRC_DIR) -type f -name '*.c')
SOURCES_ASM = $(shell find $(BOOT_DIR) -type f -name '*.asm')
HEADERS     = $(shell find $(INC_DIR) -type f -name '*.h')
OBJ_C       = $(patsubst %.c, $(OBJ_DIR)/%.o, $(notdir $(SOURCES_C)))
OBJ_ASM     = $(patsubst %.asm, $(OBJ_DIR)/%.o, $(notdir $(SOURCES_ASM)))
OBJ         = $(OBJ_C) $(OBJ_ASM)

KERNEL = $(BIN_DIR)/kernel.bin
ISO = $(BIN_DIR)/domeos.iso

RUN_CMD  = qemu-system-x86_64 -cdrom $(ISO) -monitor stdio -d cpu_reset,int -D qemu-logfile
NASM_CMD = nasm $< -felf64 -i $(BOOT_DIR)/ -o $@

CFLAGS = -ffreestanding -mcmodel=large -mno-red-zone -mno-mmx -mno-sse -mno-sse2 -O0 -Wall -Wextra -Iinclude -g
CC = x86_64-elf-gcc
LD = x86_64-elf-ld
LDFLAGS = -n --gc-sections -Tlinker.ld -g

VPATH = $(shell find $(SRC_DIR) $(INC_DIR) -type d)

# default
.PHONY: all
all: $(ISO)

# building
$(KERNEL): $(OBJ)
	$(LD) $(LDFLAGS) -o $@ $^

$(OBJ_DIR)/%.o: %.c | build_dirs
	$(CC) $(CFLAGS) -c -o $@ $<

$(OBJ_DIR)/%.o: $(BOOT_DIR)/%.asm | build_dirs
	$(NASM_CMD)

# iso
$(GRUB_CFG):
	mkdir -p $(GRUB_DIR)
	echo -e "set timeout=0\nset default=0\nmenuentry \"domeos\" {\nmultiboot2 /boot/$(notdir $(KERNEL))\nboot\n}" > $@

$(ISO): $(KERNEL) $(GRUB_CFG)
	cp $(KERNEL) $(GRUB_DIR)/../
	grub-mkrescue -o $@ $(ISO_DIR)

# phonies
.PHONY: build_dirs
build_dirs:
	@mkdir -p $(OBJ_DIR) $(BIN_DIR)

.PHONY: clean
clean:
	rm -rf $(BIN_DIR) $(OBJ_DIR)

# running
.PHONY: run-only
run-only:
	$(RUN_CMD)

.PHONY: run
run: $(ISO)
	$(RUN_CMD)

.PHONY: debug
debug:
	$(RUN_CMD) -s -S
