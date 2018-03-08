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

use memory::Frame;

pub struct Entry(u64);

impl Entry {
  pub fn is_unused(&self) -> bool {
    self.0 == 0
  }

  pub fn set_unused(&mut self) {
    self.0 = 0;
  }

  pub fn flags(&self) -> EntryFlags {
    EntryFlags::from_bits_truncate(self.0)
  }

  pub fn pointed_frame(&self) -> Option<Frame> {
    if self.flags().contains(EntryFlags::PRESENT) {
      Some(Frame::containing_address(self.0 as usize & 0x000FFFFF_FFFFF000)) // mask bits 12-51
    } else {
      None
    }
  }

  pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
    assert!(frame.start_address() & !0x000FFFFF_FFFFF000 == 0);
    self.0 = (frame.start_address() as u64) | flags.bits();
  }
}

bitflags! {
  pub struct EntryFlags: u64 {
    const PRESENT         = 1 << 0; // page in memory
    const WRITABLE        = 1 << 1; // allowed to write
    const USER_ACCESSIBLE = 1 << 2; // if unset, only kernel can access
    const WRITE_THROUGH   = 1 << 3; // writes go directly to memory
    const NO_CACHE        = 1 << 4; // no cache used
    const ACCESSED        = 1 << 5; // bit is set by cpu when page is used
    const DIRTY           = 1 << 6; // bit is set by cpu when is written to
    const HUGE_PAGE       = 1 << 7; // 0 in P1, P4 makes 1GiB in P3, and 2MiB in P2
    const GLOBAL          = 1 << 8; // page not flushed from cache on address space switch
    const NO_EXECUTE      = 1 << 63; // forbid executing code on page
  }
}