use crate::cursor::Reinterpret;
use crate::error::PeResult;
use crate::PeError;

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
    pub number_of_data_directories: u32,
}

impl OptionalHeader {
    pub fn image_type(&self) -> Result<(), u16> {
        if self.magic == 0x20B {
            Ok(())
        } else {
            Err(self.magic)
        }
    }

    /// Error if not aligned
    pub fn image_base(&self) -> PeResult<u64> {
        ensure_multiple(self.image_base, 0x10000_u64)
    }

    pub fn entry_point(&self) -> Option<u64> {
        if self.address_of_entry_point == 0 {
            None
        } else {
            Some(self.address_of_entry_point as u64)
        }
    }

    /// Error if not aligned
    pub fn size_of_image(&self) -> PeResult<usize> {
        ensure_multiple(self.size_of_image, self.section_alignment).map(|sz| sz as usize)
    }

    /// Error if not aligned
    pub fn size_of_headers(&self) -> PeResult<usize> {
        ensure_multiple(self.size_of_headers, self.file_alignment).map(|sz| sz as usize)
    }
}

fn ensure_multiple(val: impl Into<u64>, multiple: impl Into<u64>) -> PeResult<u64> {
    let val = val.into();
    let multiple = multiple.into();

    if is_multiple_of(val, multiple) {
        Ok(val)
    } else {
        Err(PeError::Unaligned {
            expected: multiple,
            value: val,
        })
    }
}

fn is_multiple_of(val: u64, multiple: u64) -> bool {
    debug_assert!(multiple.is_power_of_two());

    let mask = !(multiple - 1);
    (val & mask) == val
}

// safety: raw PE type
unsafe impl Reinterpret for OptionalHeader {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_of() {
        let base = 0x10000;
        assert!(is_multiple_of(base, base));
        assert!(is_multiple_of(base * 2, base));
        assert!(is_multiple_of(base * 50, base));
        assert!(!is_multiple_of(base + 5, base));
    }
}
