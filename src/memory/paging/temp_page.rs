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

use super::{Page, ActivePageTable, VirtualAddress};

use super::table::{Table, Level1};

use memory::{Frame, FrameAllocator};

struct TinyAllocator([Option<Frame>; 3]);

impl FrameAllocator for TinyAllocator {
  fn allocate_frame(&mut self) -> Option<Frame> {
    for frame_opt in &mut self.0 {
      if frame_opt.is_some() {
        return frame_opt.take();
      }
    }

    None
  }

  fn deallocate_frame(&mut self, frame: Frame) {
    for frame_opt in &mut self.0 {
      if frame_opt.is_none() {
        *frame_opt = Some(frame);
        return;
      }
    }

    panic!("Tiny allocator can only hold 3 frames");
  }
}

impl TinyAllocator {
  fn new<A>(allocator: &mut A) -> TinyAllocator where A: FrameAllocator {
    let mut f = || { let frame = allocator.allocate_frame(); debug!("tinyalloc {:?}", frame); frame };
    let frames = [f(), f(), f()];
    TinyAllocator(frames)
  }
}

pub struct TemporaryPage {
  page: Page,
  allocator: TinyAllocator
}

impl TemporaryPage {
  pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage where A: FrameAllocator {
    TemporaryPage {
      page: page,
      allocator: TinyAllocator::new(allocator)
    }
  }

  pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
    use super::entry::EntryFlags;

    debug!("map {:?} - {:?}", self.page, active_table.translate_page(self.page));

    //assert!(active_table.translate_page(self.page).is_none(), "temporary page is already mapped");
    active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
    self.page.start_address()
  }

  pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
    active_table.unmap(self.page, &mut self.allocator);
  }

  pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut Table<Level1> {
    unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
  }
}