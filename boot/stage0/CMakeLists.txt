cmake_minimum_required(VERSION 3.9)
project(DomeOS_bootloader_stage0)

set(STAGE0 stage0.bin)
set(DISK disk.img)
set(SRCS ${CMAKE_CURRENT_SOURCE_DIR}/stage0.asm)

set(CMAKE_ASM_NASM_OBJECT_FORMAT bin)
enable_language(ASM_NASM)

add_custom_command(OUTPUT ${STAGE0}
                   COMMENT "Compiling stage0"
                   COMMAND nasm -f bin ${SRCS} -o ${STAGE0}
                   DEPENDS ${SRCS}) # TODO multiple srcs?

add_custom_command(OUTPUT ${DISK}
                   COMMENT "Creating FAT12 disk with MBR"
                   COMMAND dd if=/dev/zero of=${DISK} count=10 bs=1M
                   # o: new dos partition table, np1\n\n: new partition at default place
                   # t1: change type to fat12, w: write
                   COMMAND bash -c "echo -e 'o\\nn\\np\\n1\\n\\n\\nt\\n1\\nw\\n' | fdisk ${DISK}"
                   VERBATIM
                   # write mbr to only 446 bytes so as not to overwrite partition table
                   COMMAND dd if=${STAGE0} of=${DISK} bs=446 count=1 conv=notrunc
                   DEPENDS ${STAGE0})

add_custom_target(run
                  COMMENT "Running qemu"
                  COMMAND qemu-system-x86_64 -hda ${DISK} -monitor stdio
                  DEPENDS ${DISK})

add_custom_target(${PROJECT_NAME}
                  DEPENDS ${DISK})

