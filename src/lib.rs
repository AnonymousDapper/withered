#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]

#![no_std]

extern crate rlibc;

#[macro_use]
mod vga;

#[lang = "eh_personality"]
extern fn eh_personality() {

}

#[no_mangle]
#[lang = "panic_fmt"]
pub extern fn rust_begin_panic() -> ! {
  loop {}
}

#[no_mangle]
pub extern fn kmain() -> ! {
  vga::clear_screen();
  vga::fg_color(vga::Color::Yellow);

  println!("< WITHERED >");

  vga::fg_color(vga::Color::LightGrey);
  println!("[INFO] Started OS init");

  loop { }
}