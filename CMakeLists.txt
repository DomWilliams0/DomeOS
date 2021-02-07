cmake_minimum_required(VERSION 3.9)
project(DomeOS
        VERSION 0.1)

add_subdirectory(boot)
add_subdirectory(kernel)

add_executable(${PROJECT_NAME} $<TARGET_OBJECTS:DomeOS_bootloader>)
set_property(TARGET ${PROJECT_NAME} PROPERTY ENABLE_EXPORTS 1)
target_link_options(${PROJECT_NAME} PRIVATE -n --gc-sections -T${CMAKE_CURRENT_SOURCE_DIR}/linker.ld -g)
target_link_libraries(${PROJECT_NAME} DomeOS_kernel)

# iso
set(ISO_DIR  ${CMAKE_CURRENT_BINARY_DIR}/isofiles)
set(GRUB_DIR ${ISO_DIR}/boot/grub)
set(GRUB_CFG ${GRUB_DIR}/grub.cfg)
set(ISO_BIN  ${ISO_DIR}/${PROJECT_NAME}.iso)
file(GENERATE
     OUTPUT ${GRUB_CFG}
     CONTENT "\
     set timeout=0
     set default=0
     menuentry \"domeos\" {
         multiboot /boot/$<TARGET_FILE_BASE_NAME:${PROJECT_NAME}> some args in grub.cfg
         boot
     }")

add_custom_command(OUTPUT ${ISO_BIN}
                   COMMENT "Creating ISO with grub"
                   COMMAND ${CMAKE_COMMAND} -E make_directory ${GRUB_DIR}
                   COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}> ${GRUB_DIR}/../
                   COMMAND ${CMAKE_COMMAND} -E remove -f ${ISO_BIN}
                   COMMAND grub-mkrescue -o ${ISO_BIN} ${ISO_DIR}
                   BYPRODUCTS ${GRUB_DIR} ${ISO_DIR}
                   DEPENDS ${GRUB_CFG} ${PROJECT_NAME})

add_custom_target(iso DEPENDS ${ISO_BIN})

function(mk_run name comment extra_flags)
    add_custom_target(${name}
                      COMMENT "${comment}"
                      COMMAND qemu-system-x86_64 -cdrom ${ISO_BIN} -monitor stdio -serial file:serial.log
                      -d cpu_reset,int -D qemu-logfile -no-reboot ${extra_flags}
                      BYPRODUCTS serial.log qemu-logfile
                      DEPENDS ${ISO_BIN})
endfunction()

mk_run(run "Running qemu" "")
mk_run(debug "Debugging qemu" "-s;-S")
