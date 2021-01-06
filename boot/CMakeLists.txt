project(DomeOS_bootloader
        LANGUAGES ASM_NASM)

enable_language(ASM_NASM)
set(SRCS
    boot.asm
    boot.h
    gdt.asm
    isr.asm
    long_mode.asm
    multiboot.asm
    multiboot.h)
set(CMAKE_ASM_NASM_OBJECT_FORMAT elf64 PARENT_SCOPE)
set(CMAKE_ASM_NASM_LINK_EXECUTABLE "ld <LINK_FLAGS> <OBJECTS> -o <TARGET> <LINK_LIBRARIES>" PARENT_SCOPE)
add_compile_options(-I ${CMAKE_CURRENT_SOURCE_DIR}/)

add_library(${PROJECT_NAME} OBJECT ${SRCS})
