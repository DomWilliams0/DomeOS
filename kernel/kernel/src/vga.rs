use core::fmt;
use core::mem::MaybeUninit;

use core::ops::{Deref, DerefMut};
use log::*;
use utils::memory::address::VirtualAddress;
use volatile::Volatile;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const VGA_ADDR: usize = 0xb8000;

type VGABuffer = [[Volatile<ScreenChar>; WIDTH]; HEIGHT];

/// Must be initialized with `init` before any printing is done
static mut SCREEN: MaybeUninit<spin::Mutex<Screen>> = MaybeUninit::uninit();
static mut SCREEN_INIT: bool = false;

#[allow(unused)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    character: u8,
    color: u8,
}

pub struct Screen {
    buffer: &'static mut VGABuffer,
    foreground: Color,
    background: Color,
    x: usize,
    y: usize,
}

pub fn set_colors(fg: Color, bg: Color) -> ColorGuard {
    let mut vga = get();
    let guard = ColorGuard {
        fg: vga.foreground,
        bg: vga.background,
    };

    vga.set_colors(fg, bg);
    guard
}

pub fn init(fg: Color, bg: Color) {
    debug_assert!(!is_initialized());

    unsafe {
        SCREEN
            .as_mut_ptr()
            .write(spin::Mutex::new(Screen::with_colors(fg, bg)));
        SCREEN_INIT = true;
    }
}

pub fn get<'a>() -> spin::MutexGuard<'a, Screen> {
    unsafe { SCREEN.assume_init_mut().lock() }
}

pub fn is_initialized() -> bool {
    unsafe { SCREEN_INIT }
}

fn fill<FG, BG>(fg: FG, bg: BG)
where
    FG: Into<Option<Color>>,
    BG: Into<Option<Color>>,
{
    let mut screen = get();
    screen.set_colors(fg, bg);
    screen.clear()
}

pub fn set_error_colors() {
    get().set_colors(Color::White, Color::Red);
}

/// # Safety
/// New address must be writable and the start of the physical VGA buffer
pub unsafe fn move_vga_buffer(new_addr: VirtualAddress) {
    get().buffer = &mut *(new_addr.0 as *mut VGABuffer);
}

impl ScreenChar {
    fn new(c: u8, fg: Color, bg: Color) -> Self {
        Self {
            character: c,
            color: color_as_byte(fg, bg),
        }
    }
}

impl Deref for ScreenChar {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

fn color_as_byte(fg: Color, bg: Color) -> u8 {
    fg as u8 | ((bg as u8) << 4)
}

impl Screen {
    fn with_colors(fg: Color, bg: Color) -> Self {
        let mut s = Self {
            buffer: unsafe { &mut *(VGA_ADDR as *mut VGABuffer) },
            foreground: fg,
            background: bg,
            x: 0,
            y: 0,
        };
        debug!(
            "initialized vga with foreground {:?} and background {:?}",
            fg, bg
        );
        s.clear();
        s
    }

    fn set_colors<FG, BG>(&mut self, fg: FG, bg: BG)
    where
        FG: Into<Option<Color>>,
        BG: Into<Option<Color>>,
    {
        if let Some(fg) = fg.into() {
            self.foreground = fg
        }
        if let Some(bg) = bg.into() {
            self.background = bg
        }
    }

    fn screen_char(&self, c: u8) -> ScreenChar {
        ScreenChar::new(c, self.foreground, self.background)
    }

    fn clear(&mut self) {
        let sc = self.screen_char(b' ');
        let len = WIDTH * HEIGHT;
        let buf: *mut ScreenChar = self.buffer as *mut VGABuffer as *mut ScreenChar;

        unsafe {
            let slice = core::slice::from_raw_parts_mut(buf, len);
            for c in slice {
                *c = sc;
            }
        }

        self.x = 0;
        self.y = 0;
    }

    fn scroll_down(&mut self) {
        // move all rows up
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let v = self.buffer[row][col].read();
                self.buffer[row - 1][col].write(v);
            }
        }

        // clear bottom row
        let blank = self.screen_char(b' ');
        for col in 0..WIDTH {
            self.buffer[HEIGHT - 1][col].write(blank);
        }

        if self.y > 0 {
            self.y -= 1;
        }
    }

    fn new_line(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y >= HEIGHT {
            self.scroll_down();
        }
    }

    fn write_byte(&mut self, b: u8) {
        // TODO limited to ascii for now
        let new_line = if b == b'\n' {
            true
        } else {
            let b = match b {
                0x20..=0x7e => b,
                _ => b'?',
            };

            let sc = self.screen_char(b);
            self.buffer[self.y][self.x].write(sc);
            self.x += 1;
            self.x >= WIDTH
        };

        // wrap
        if new_line {
            self.new_line();
        }
    }

    fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

pub struct ColorGuard {
    fg: Color,
    bg: Color,
}

impl Drop for ColorGuard {
    fn drop(&mut self) {
        get().set_colors(self.fg, self.bg);
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_raw_print(format_args!($($arg)*)));
}

#[doc(hidden)]
/// Use println!() and print!()
pub fn _raw_print(args: fmt::Arguments) {
    use fmt::Write;
    get().write_fmt(args).unwrap();
}

impl fmt::Write for Screen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
