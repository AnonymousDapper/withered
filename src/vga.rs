/*

MIT License

Copyright (c) 2018 AnonymousDapper

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

extern crate volatile;
extern crate spin;

use core::fmt;

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
  Red = 4, // Error
  Magenta = 5,
  Brown = 6,
  LightGrey = 7, // Log
  DarkGrey = 8, // Debug
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14, // Warn
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
  buffer: &'static mut Buffer
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
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_char: byte,
          color: color
        });

        self.column_pos += 1;
      }
    }
  }

  fn new_line(&mut self) {
    for row in 1..BUFFER_HEIGHT {
      for col in 0..BUFFER_WIDTH {
        let character = self.buffer.chars[row][col].read();
        self.buffer.chars[row - 1][col].write(character);
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
      self.buffer.chars[row][col].write(blank);
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

#[allow(unused_macros)]
macro_rules! print {
  ($($arg:tt)*) => ({
    $crate::vga::print(format_args!($($arg)*));
  });
}

#[allow(unused_macros)]
macro_rules! println {
  ($fmt:expr) => (print!(concat!($fmt, "\n")));
  ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[allow(unused_macros)]
macro_rules! debug {
  ($fmt:expr) => ($crate::vga::print_color($crate::vga::Color::DarkGrey, format_args!(concat!("[DEBUG] ", concat!($fmt, "\n")))));
  ($fmt:expr, $($arg:tt)*) => ($crate::vga::print_color($crate::vga::Color::DarkGrey, format_args!(concat!("[DEBUG] ", concat!($fmt, "\n")), $($arg)*)));
}

#[allow(unused_macros)]
macro_rules! log {
  ($fmt:expr) => ($crate::vga::print_color($crate::vga::Color::LightGrey, format_args!(concat!("[INFO] ", concat!($fmt, "\n")))));
  ($fmt:expr, $($arg:tt)*) => ($crate::vga::print_color($crate::vga::Color::LightGrey, format_args!(concat!("[INFO] ", concat!($fmt, "\n")), $($arg)*)));
}

#[allow(unused_macros)]
macro_rules! warn {
  ($fmt:expr) => ($crate::vga::print_color($crate::vga::Color::Yellow, format_args!(concat!("[WARN] ", concat!($fmt, "\n")))));
  ($fmt:expr, $($arg:tt)*) => ($crate::vga::print_color($crate::vga::Color::Yellow, format_args!(concat!("[WARN] ", concat!($fmt, "\n")), $($arg)*)));
}

#[allow(unused_macros)]
macro_rules! error {
  ($fmt:expr) => ($crate::vga::print_color($crate::vga::Color::Red, format_args!(concat!("[ERROR] ", concat!($fmt, "\n")))));
  ($fmt:expr, $($arg:tt)*) => ($crate::vga::print_color($crate::vga::Color::Red, format_args!(concat!("[ERROR] ", concat!($fmt, "\n")), $($arg)*)));
}

// public interface
lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_pos: 0,
    fg_color: Color::LightGrey,
    bg_color: Color::Black,
    buffer: unsafe { &mut *(0xB8000 as *mut Buffer) }
  });
}

// clear screen
pub fn clear_screen() {
  for _ in 0..BUFFER_HEIGHT {
    println!("");
  }
}

// print

#[allow(dead_code)]
pub fn print(args: fmt::Arguments) {
  use core::fmt::Write;
  WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
pub fn print_color(color: Color, args: fmt::Arguments) {
  use core::fmt::Write;
  let mut writer = WRITER.lock();
  writer.fg_color = color;
  writer.write_fmt(args).unwrap();
}


#[allow(dead_code)]
pub fn fg_color(color: Color) {
  WRITER.lock().fg_color = color;
}

#[allow(dead_code)]
pub fn bg_color(color: Color) {
  WRITER.lock().bg_color = color;
}