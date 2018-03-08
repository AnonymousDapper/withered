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

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use memory::FrameAllocator;

use memory::paging::ENTRY_COUNT;
use memory::paging::entry::{Entry, EntryFlags};

pub const P4: *mut Table<Level4> = 0xFFFFFFFF_FFFFF000 as *mut _; // special virtual address for P4 table

pub trait TableLevel { }
pub trait HierarchicalLevel: TableLevel {
  type NextLevel: TableLevel;
}

pub enum Level4 { }
pub enum Level3 { }
pub enum Level2 { }
pub enum Level1 { }

impl TableLevel for Level4 { }
impl TableLevel for Level3 { }
impl TableLevel for Level2 { }
impl TableLevel for Level1 { }

impl HierarchicalLevel for Level4 {
  type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
  type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
  type NextLevel = Level1;
}

pub struct Table<L: TableLevel> {
  entries: [Entry; ENTRY_COUNT],
  level: PhantomData<L>
}

impl<L> Table<L> where L: TableLevel {
  pub fn zero(&mut self) {
    for entry in self.entries.iter_mut() {
      entry.set_unused();
    }
  }
}

impl<L> Table<L> where L: HierarchicalLevel {
  fn next_table_address(&self, index: usize) -> Option<usize> {
    let entry_flags = self[index].flags();

    if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE) {
      let table_address = self as *const _ as usize;
      Some((table_address << 9) | (index << 12))

    } else {
      None
    }
  }

  pub fn next_table<'a>(&'a self, index: usize) -> Option<&'a Table<L::NextLevel>> {
    self.next_table_address(index).map(|addr| unsafe { &*(addr as *const _) })
  }

  pub fn next_table_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Table<L::NextLevel>> {
    self.next_table_address(index).map(|addr| unsafe { &mut *(addr as *mut _) })
  }

  pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut Table<L::NextLevel> where A: FrameAllocator {
    if self.next_table(index).is_none() {
      assert!(!self.entries[index].flags().contains(EntryFlags::HUGE_PAGE), "mapping code does not support huge pages");

      let frame = allocator.allocate_frame().expect("no frames available");
      self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
      self.next_table_mut(index).unwrap().zero();
    }

    self.next_table_mut(index).unwrap()
  }
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
  type Output = Entry;

  fn index(&self, index: usize) -> &Entry {
    &self.entries[index]
  }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
  fn index_mut(&mut self, index: usize) -> &mut Entry {
    &mut self.entries[index]
  }
}