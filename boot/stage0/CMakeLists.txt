cmake_minimum_required(VERSION 3.9)
project(DomeOS_bootloader_stage0)

set(STAGE0 stage0.bin)
set(FLOPPY floppy.img)
set(SRCS ${CMAKE_CURRENT_SOURCE_DIR}/stage0.asm)

set(CMAKE_ASM_NASM_OBJECT_FORMAT bin)
enable_language(ASM_NASM)

add_custom_command(OUTPUT ${STAGE0}
                   COMMENT "Compiling stage0"
                   COMMAND nasm -f bin ${SRCS} -o ${STAGE0}) # TODO multiple srcs?

add_custom_command(OUTPUT ${FLOPPY}
                   COMMENT "Creating floppy image"
                   COMMAND dd if=/dev/zero of=${FLOPPY} count=1440 bs=1k
                   COMMAND mkfs.msdos ${FLOPPY}
                   COMMAND dd if=${STAGE0} of=${FLOPPY}
                   DEPENDS ${STAGE0})

add_custom_target(${PROJECT_NAME}
                  DEPENDS ${FLOPPY})

