pub use generated::*;
pub use memory_map::{MemoryRegion, MemoryRegionType};
use utils::prelude::Bit;

use crate::multiboot::ctypes::c_void;
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;

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

pub fn parse(magic: u32, multiboot: *mut c_void) -> Result<*mut multiboot_info, &'static str> {
    match (magic, multiboot) {
        (magic, _) if magic != MULTIBOOT_BOOTLOADER_MAGIC => Err("missing multiboot magic"),
        (_, ptr) if ptr.is_null() => Err("null pointer to multiboot info"),
        (_, multiboot) => Ok(multiboot as *mut multiboot_info),
    }
}

pub struct CommandLine<'multiboot> {
    start: *mut u8,
    phantom: PhantomData<&'multiboot ()>,
}

impl<'multiboot> CommandLine<'multiboot> {
    pub fn init(multiboot: &multiboot_info) -> Option<Self> {
        if multiboot.flags.bit(2) {
            Some(Self {
                start: multiboot.cmdline as *mut u8,
                phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

impl Display for CommandLine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut ptr = self.start;
        // safety: field is initialized from multiboot when bit 2 is set, and lifetime ensures this
        // doesn't outlive the data
        unsafe {
            while *ptr != 0x00 {
                write!(f, "{}", *ptr as char)?;
                ptr = ptr.add(1);
            }
        }

        Ok(())
    }
}

pub fn log_command_line(multiboot: &multiboot_info) {
    match CommandLine::init(multiboot) {
        None => log::info!("no command line args given"),
        Some(args) => log::info!("command line: '{}'", args),
    }
}