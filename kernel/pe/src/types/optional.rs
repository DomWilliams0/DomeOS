/// PE32+ only
#[derive(Debug)]
#[repr(C)]
pub struct OptionalHeader {
    magic: u16,

    major_linker_version: u8,
    minor_linker_version: u8,
    size_of_code: u32,
    size_of_initialized_data: u32,
    size_of_uninitialized_data: u32,
    address_of_entry_point: u32,
    base_of_code: u32,

    // missing base_of_data on PE32+
    image_base: u64,
    section_alignment: u32,
    file_alignment: u32,
    major_os_version: u16,
    minor_os_version: u16,
    major_image_version: u16,
    minor_image_version: u16,
    major_subsystem_version: u16,
    minor_subsystem_version: u16,
    _win32_version_value: u32,

    size_of_image: u32,
    size_of_headers: u32,
    checksum: u32,
    subsystem: u16,
    dll_characteristics: u16,

    size_of_stack_reserve: u64,
    size_of_stack_commit: u64,
    size_of_heap_reserve: u64,
    size_of_heap_commit: u64,

    _loader_flags: u32,
    number_of_data_directories: u32,
}

impl OptionalHeader {
    pub fn image_type(&self) -> Result<(), u16> {
        if self.magic == 0x20B {
            Ok(())
        } else {
            Err(self.magic)
        }
    }
}
