#![feature(lang_items)]
#![feature(pointer_methods)]
#![no_std]

#[lang = "eh_personality"]
extern fn eh_personality() {

}

#[lang = "panic_fmt"]
extern fn rust_begin_panic() -> ! {
  loop {}
}

#[no_mangle]
pub extern fn kmain() -> ! {
  unsafe {
    let vga = 0xB8000 as *mut u64;

    *vga = 0x2F592F412F4B2F4F;
  };

  loop { }
}