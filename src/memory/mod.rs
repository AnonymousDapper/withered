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

mod frame_alloc;
mod paging;

pub use self::frame_alloc::AreaFrameAllocator;

use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
  number: usize
}

impl Frame {
  fn containing_address(addr: usize) -> Frame {
    Frame {
      number: addr / PAGE_SIZE
    }
  }

  fn start_address(&self) -> PhysicalAddress {
    self.number * PAGE_SIZE
  }

  fn clone(&self) -> Frame {
    Frame {
      number: self.number
    }
  }
}

pub trait FrameAllocator {
  fn allocate_frame(&mut self) -> Option<Frame>;
  fn deallocate_frame(&mut self, frame: Frame);
}