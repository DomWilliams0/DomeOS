use bitfield::Bit;
use crate::multiboot::ctypes::c_void;
pub use generated::*;

/// Stolen from libc::unix
#[allow(warnings)]
mod ctypes {
    pub use core::ffi::c_void;
    pub type c_schar = i8;
    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_float = f32;
    pub type c_double = f64;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
    pub type intmax_t = i64;
    pub type uintmax_t = u64;

    pub type size_t = usize;
    pub type ptrdiff_t = isize;
    pub type intptr_t = isize;
    pub type uintptr_t = usize;
    pub type ssize_t = isize;
}

/// Autogenerated from multiboot.h
#[allow(warnings)]
mod generated;
mod memory_map;

pub use memory_map::{MemoryRegion, MemoryRegionType, MemoryRegions};

pub fn parse(magic: u32, multiboot: *mut c_void) -> Result<*mut multiboot_info, &'static str> {
    match (magic, multiboot) {
        (magic, _) if magic != MULTIBOOT_BOOTLOADER_MAGIC => Err("missing multiboot magic"),
        (_, ptr) if ptr.is_null() => Err("null pointer to multiboot info"),
        (_, multiboot) => Ok(multiboot as *mut multiboot_info),
    }
}

pub fn print_commandline(multiboot: &multiboot_info) {
    use crate::{print, println};

    if multiboot.flags.bit(2) {
        print!("command line: '");
        unsafe {
            let mut ptr = multiboot.cmdline as *mut u8;
            loop {
                if *ptr == 0x00 {
                    break;
                }
                print!("{}", *ptr as char);
                ptr = ptr.add(1);
            }
        }
        println!("'");
    }
}