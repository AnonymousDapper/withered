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

mod entry;
mod table;

use memory::{PAGE_SIZE, Frame, FrameAllocator};

use self::entry::EntryFlags;

use self::table::{P4, Table, Level4};

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

// basically a Frame, but virtual instead of physical
#[derive(Debug, Clone, Copy)]
pub struct Page {
  number: usize
}

impl Page {
  pub fn containing_address(address: VirtualAddress) -> Page {
    assert!(address < 0x0000_8000_0000_0000 || address >= 0xFFFF_8000_0000_0000, "invalid page address: 0x{:x}", address);
    Page {
      number: address / PAGE_SIZE
    }
  }

  fn start_address(&self) -> usize {
    self.number * PAGE_SIZE
  }

  fn p4_index(&self) -> usize {
    (self.number >> 27) & 0x1FF
  }

  fn p3_index(&self) -> usize {
    (self.number >> 18) & 0x1FF
  }

  fn p2_index(&self) -> usize {
    (self.number >> 9) & 0x1FF
  }

  fn p1_index(&self) -> usize {
    (self.number >> 0) & 0x1FF
  }
}

pub struct ActivePageTable {
  p4: &'static mut Table<Level4>
}

impl ActivePageTable {
  pub unsafe fn new() -> ActivePageTable {
    ActivePageTable {
      p4: unsafe { &mut *(P4 as *mut Table<Level4>) }
    }
  }

  fn p4(&self) -> &Table<Level4> {
    unsafe { &*self.p4 }
  }

  fn p4_mut(&mut self) -> &mut Table<Level4> {
    unsafe { &mut *self.p4}
  }

  pub fn translate(&self, vt_addr: VirtualAddress) -> Option<PhysicalAddress> {
    let offset = vt_addr % PAGE_SIZE;
    self.translate_page(Page::containing_address(vt_addr)).map(|frame| frame.number * PAGE_SIZE + offset)
  }

  fn translate_page(&self, page: Page) -> Option<Frame> {

    let p3 = self.p4().next_table(page.p4_index());

    let huge_page = || {
      p3.and_then(|p3| {
        let p3_entry = &p3[page.p3_index()];

        if let Some(start_frame) = p3_entry.pointed_frame() { // 1GiB page
          if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
            assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0); // address must be 1Gib aligned
            return Some(Frame {
              number: start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index()
            });
          }
        }

        if let Some(p2) = p3.next_table(page.p3_index()) {
          let p2_entry = &p2[page.p2_index()];

          if let Some(start_frame) = p2_entry.pointed_frame() { // 2 MiB page?
            assert!(start_frame.number & ENTRY_COUNT == 0); // address must be 2MiB aligned
            return Some(Frame {
              number: start_frame.number + page.p1_index()
            });
          }
        }

        None
      })
    };

    p3.and_then(|p3| p3.next_table(page.p3_index()))
      .and_then(|p2| p2.next_table(page.p2_index()))
      .and_then(|p1| p1[page.p1_index()].pointed_frame())
      .or_else(huge_page)
  }

  pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
    let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
    let mut p2 = p3.next_table_create(page.p3_index(), allocator);
    let mut p1 = p2.next_table_create(page.p2_index(), allocator);

    assert!(p1[page.p1_index()].is_unused());
    p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
  }

  pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
    let frame = allocator.allocate_frame().expect("out of memory");
    self.map_to(page, frame, flags, allocator)
  }

  fn unmap<A>(&mut self, page: Page, allocator: &mut A) where A: FrameAllocator {
    assert!(self.translate(page.start_address()).is_some());

    let p1 = self.p4_mut()
      .next_table_mut(page.p4_index())
      .and_then(|p3| p3.next_table_mut(page.p3_index()))
      .and_then(|p2| p2.next_table_mut(page.p2_index()))
      .expect("mapping code does not support huge pages");

    let frame = p1[page.p1_index()].pointed_frame().unwrap();
    p1[page.p1_index()].set_unused();

    use x86_64::instructions::tlb;
    use x86_64::VirtualAddress;
    tlb::flush(VirtualAddress(page.start_address()));

    //allocator.deallocate_frame(frame);
  }
}

pub struct InactivePageTable {
  p4_frame: Frame
}

impl InactivePageTable {
  pub fn new(frame: Frame) -> InactivePageTable {
    // TODO zero and r-map
    InactivePageTable {
      p4_frame: frame
    }
  }
}