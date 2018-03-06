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

use memory::{Frame, FrameAllocator};

use multiboot2::{MemoryAreaIter, MemoryArea};

pub struct AreaFrameAllocator {
  next_free_frame: Frame,
  current_area: Option<&'static MemoryArea>,
  areas: MemoryAreaIter,
  kernel_start: Frame,
  kernel_end: Frame,
  mb_start: Frame,
  mb_end: Frame
}

impl FrameAllocator for AreaFrameAllocator {
  fn allocate_frame(&mut self) -> Option<Frame> {
    if let Some(area) = self.current_area {
      // return a frame with same address if free
      let frame = Frame {
        number: self.next_free_frame.number
      };

      let current_area_last_frame = { // last frame of current area
        let addr = area.base_addr + area.length - 1;
        Frame::containing_address(addr as usize)
      };

      if frame > current_area_last_frame { // have we used all of our area?
        self.choose_next_area(); // yes

      } else if frame >= self.kernel_start && frame <= self.kernel_end { // frame is in use by kernel
        self.next_free_frame = Frame {
          number: self.kernel_end.number + 1
        };

      } else if frame >= self.mb_start && frame <= self.mb_end { // frame is in use by multiboot
        self.next_free_frame = Frame {
          number: self.mb_end.number + 1
        };

      } else {
        self.next_free_frame.number += 1; // frame unused, return it
        return Some(frame);
      }

      self.allocate_frame() // frame not valid, try another

    } else {

      None // no free frames
    }
  }

  fn deallocate_frame(&mut self, _frame: Frame) {
    unimplemented!()
  }
}

impl AreaFrameAllocator {
  pub fn new(kernel_start: usize, kernel_end: usize, mb_start: usize, mb_end: usize, memory_areas: MemoryAreaIter) -> AreaFrameAllocator {
    let mut allocator = AreaFrameAllocator {
      next_free_frame: Frame::containing_address(0),
      current_area: None,
      areas: memory_areas,
      kernel_start: Frame::containing_address(kernel_start),
      kernel_end: Frame::containing_address(kernel_end),
      mb_start: Frame::containing_address(mb_start),
      mb_end: Frame::containing_address(mb_end)
    };

    allocator.choose_next_area();
    allocator
  }

  fn choose_next_area(&mut self) {
    self.current_area = self.areas.clone().filter(|area| {
      let addr = area.base_addr + area.length - 1;
      Frame::containing_address(addr as usize) >= self.next_free_frame
    }).min_by_key(|area| area.base_addr); // find lowest free frame

    if let Some(area) = self.current_area {
      let start_frame = Frame::containing_address(area.base_addr as usize);
      if self.next_free_frame < start_frame {
        self.next_free_frame = start_frame
      }
    }
  }
}