// vga module
// Copyright AnonymousDapper 2018

extern crate volatile;
extern crate spin;

use core::fmt;
use core::ptr::Unique;

use self::volatile::Volatile;

use self::spin::Mutex;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGrey = 7,
  DarkGrey = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15
}

// color code information
#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
  const fn new(fg: Color, bg: Color) -> ColorCode {
    ColorCode((bg as u8) << 4 | (fg as u8))
  }
}

// character object
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
  ascii_char: u8,
  color: ColorCode
}

// text buffer
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
  column_pos: usize,
  fg_color: Color,
  bg_color: Color,
  buffer: Unique<Buffer>
}

// writing to screen
impl Writer {
  fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => self.new_line(),
      //128 ... 143 => self.fg_color = byte - 128,
      //144 ... 159 => self.bg_color = byte - 144,
      byte => {
        if self.column_pos >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_pos;

        let color = ColorCode::new(self.fg_color, self.bg_color);
        self.buffer().chars[row][col].write(ScreenChar {
          ascii_char: byte,
          color: color
        });

        self.column_pos += 1;
      }
    }
  }

  fn buffer(&mut self) -> &mut Buffer {
    unsafe {
      self.buffer.as_mut()
    }
  }

  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let buffer = self.buffer();
        let character = buffer.chars[row][col].read();
        buffer.chars[row - 1][col].write(character);
      }
    }

    self.clear_row(BUFFER_HEIGHT - 1);
    self.column_pos = 0;
  }

  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_char: b' ',
      color: ColorCode::new(self.fg_color, self.bg_color)
    };

    for col in 0..BUFFER_WIDTH {
      self.buffer().chars[row][col].write(blank);
    }
  }
}

// builtin display impl
impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    //let mut get_color = false;
    //let mut is_fg = true;

    for byte in s.bytes() {
      /*match byte {
        27 => { get_color = true; is_fg = true; },
        48 ... 57 | 97 ... 102 => {
          if get_color {
            self.write_byte(byte + 128);
          } else {
            self.write_byte(byte);
          };
        },
        _ => self.write_byte(byte)
      }*/
      self.write_byte(byte);
    }

    Ok(())
  }
}

// macros and publics
macro_rules! print {
  ($($arg:tt)*) => ({
    $crate::vga::print(format_args!($($arg)*));
  });
}

macro_rules! println {
  ($fmt:expr) => (print!(concat!($fmt, "\n")));
  ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

// public interface
pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
  column_pos: 0,
  fg_color: Color::LightGrey,
  bg_color: Color::Black,
  buffer: unsafe { Unique::new_unchecked(0xB8000 as *mut _) }
});

// clear screen
pub fn clear_screen() {
  for _ in 0..BUFFER_HEIGHT {
    println!("");
  }
}

// print
pub fn print(args: fmt::Arguments) {
  use core::fmt::Write;
  WRITER.lock().write_fmt(args).unwrap();
}


pub fn fg_color(color: Color) {
  WRITER.lock().fg_color = color;
}

pub fn bg_color(color: Color) {
  WRITER.lock().bg_color = color;
}