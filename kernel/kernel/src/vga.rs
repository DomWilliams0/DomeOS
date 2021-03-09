use core::fmt;

use crate::spinlock::SpinLock;
use common::*;
use core::ops::{Deref, DerefMut};

use common::InitializedGlobalChecked;
use memory::VirtualAddress;
use volatile::Volatile;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const VGA_ADDR: usize = 0xb8000;

type VgaBuffer = [[Volatile<ScreenChar>; WIDTH]; HEIGHT];

/// Must be initialized with `init` before any printing is done
static mut SCREEN: InitializedGlobalChecked<SpinLock<Screen>> = InitializedGlobalChecked::uninit();

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
    buffer: &'static mut VgaBuffer,
    foreground: Color,
    background: Color,
    x: usize,
    y: usize,
}

pub struct VgaGuard<'a> {
    screen: spin::MutexGuard<'a, Screen>,

    /// (fg, bg) to revert to on drop
    original: Option<(Color, Color)>,
}

pub fn init(fg: Color, bg: Color) {
    let mut screen = Screen {
        buffer: unsafe { &mut *(VGA_ADDR as *mut VgaBuffer) },
        foreground: fg,
        background: bg,
        x: 0,
        y: 0,
    };
    debug!(
        "initialized vga with foreground {:?} and background {:?}",
        fg, bg
    );
    screen.clear();

    unsafe {
        SCREEN.init(SpinLock::new(screen));
    }
}

pub fn get<'a>() -> VgaGuard<'a> {
    let screen = unsafe { SCREEN.get().lock() };
    let colors = screen.colors();

    VgaGuard {
        screen,
        original: Some(colors),
    }
}

pub fn try_get<'a>() -> Option<VgaGuard<'a>> {
    let screen = unsafe { SCREEN.get().try_lock() };

    screen.map(|screen| {
        let colors = screen.colors();

        VgaGuard {
            screen,
            original: Some(colors),
        }
    })
}

pub fn is_initialized() -> bool {
    unsafe { SCREEN.is_initialized() }
}

impl ScreenChar {
    fn new(c: u8, fg: Color, bg: Color) -> Self {
        Self {
            character: c,
            color: color_as_byte(fg, bg),
        }
    }
}

fn color_as_byte(fg: Color, bg: Color) -> u8 {
    fg as u8 | ((bg as u8) << 4)
}

impl Screen {
    pub fn set_colors(&mut self, fg: impl Into<Option<Color>>, bg: impl Into<Option<Color>>) {
        if let Some(fg) = fg.into() {
            self.foreground = fg
        }
        if let Some(bg) = bg.into() {
            self.background = bg
        }
    }

    fn colors(&self) -> (Color, Color) {
        (self.foreground, self.background)
    }

    fn screen_char(&self, c: u8) -> ScreenChar {
        ScreenChar::new(c, self.foreground, self.background)
    }

    fn clear(&mut self) {
        let sc = self.screen_char(b' ');
        let len = WIDTH * HEIGHT;
        let buf: *mut ScreenChar = self.buffer as *mut VgaBuffer as *mut ScreenChar;

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

    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        fmt::Write::write_fmt(self, args).unwrap();
    }

    /// # Safety
    /// New address must be writable and the start of the physical VGA buffer
    pub unsafe fn move_vga_buffer(&mut self, new_addr: VirtualAddress) {
        self.buffer = &mut *(new_addr.0 as *mut VgaBuffer);
    }
}

impl Drop for VgaGuard<'_> {
    fn drop(&mut self) {
        if let Some((fg, bg)) = self.original {
            self.screen.set_colors(fg, bg);
        }
    }
}

impl Deref for VgaGuard<'_> {
    type Target = Screen;

    fn deref(&self) -> &Self::Target {
        &self.screen
    }
}

impl DerefMut for VgaGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.screen
    }
}

impl fmt::Write for Screen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
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
