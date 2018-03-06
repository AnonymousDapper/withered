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

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]

#![no_std]

extern crate rlibc;
extern crate multiboot2;

#[macro_use]
mod vga;

mod memory;

use memory::FrameAllocator;

#[lang = "eh_personality"]
extern fn eh_personality() {

}

#[no_mangle]
#[lang = "panic_fmt"]
pub extern fn rust_begin_panic(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
  error!("Kernel panic in {}:{} :\n    {}", file, line, fmt);

  loop { }
}

#[no_mangle]
pub extern fn kmain(mbt_info: usize) -> ! {
  vga::clear_screen();
  vga::fg_color(vga::Color::Pink);

  println!("< WITHERED >");
  log!("Started OS init");


  let boot_info = unsafe { multiboot2::load(mbt_info) };
  debug!("Loaded boot info");
  let mem_map_tag = boot_info.memory_map_tag().expect("No memory map tag");

  debug!("Memory Map:");
  for area in mem_map_tag.memory_areas() {
    debug!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
  }

  let elf_sect_tag = boot_info.elf_sections_tag().expect("No elf sections tag");

  debug!("Kernel sections:");
  for section in elf_sect_tag.sections() {
    debug!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", section.addr, section.size, section.flags);
  }

  let kernel_start = elf_sect_tag.sections().map(|s| s.addr).min().unwrap();
  let kernel_end = elf_sect_tag.sections().map(|s| s.addr + s.size).max().unwrap();
  debug!("Kernel start: 0x{:x}, Kernel end: 0x{:x}", kernel_start, kernel_end);

  let mb_start = mbt_info;
  let mb_end = mb_start + (boot_info.total_size as usize);
  debug!("MB start: 0x{:x}, MB end: 0x{:x}", mb_start, mb_end);

  let mut frame_allocator = memory::AreaFrameAllocator::new(kernel_start as usize, kernel_end as usize, mb_start, mb_end, mem_map_tag.memory_areas());

  for i in 0.. {
    if let None = frame_allocator.allocate_frame() {
      debug!("Allocated {} memory frames, ({}Mib)", i, (i * 4096) / (1024 * 1024));
      break;
    }
  }


  loop { }
}

