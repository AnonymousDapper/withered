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
mod temp_page;
mod mapper;

use multiboot2::BootInformation;

use core::ops::{Deref, DerefMut};

use memory::{PAGE_SIZE, Frame, FrameAllocator};

use self::entry::EntryFlags;

// use self::table::{P4, Table, Level4};

use self::temp_page::TemporaryPage;

use self::mapper::Mapper;

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

pub struct InactivePageTable {
  p4_frame: Frame
}

impl InactivePageTable {
  pub fn new(frame: Frame, active_table: &mut ActivePageTable, temp_page: &mut TemporaryPage) -> InactivePageTable {
    {
      debug!("new inactive: {:?}", frame);
      let table = temp_page.map_table_frame(frame.clone(), active_table);

      // zero and r-map
      table.zero();
      table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
    }

    // unmap current active table
    temp_page.unmap(active_table);

    InactivePageTable {
      p4_frame: frame
    }
  }
}

pub struct ActivePageTable {
  mapper: Mapper
}

impl Deref for ActivePageTable {
  type Target = Mapper;

  fn deref(&self) -> &Mapper {
    &self.mapper
  }
}

impl DerefMut for ActivePageTable {
  fn deref_mut(&mut self) -> &mut Mapper {
    &mut self.mapper
  }
}

impl ActivePageTable {
  unsafe fn new() -> ActivePageTable {
    ActivePageTable {
      mapper: Mapper::new()
    }
  }

  pub fn with<F>(&mut self, table: &mut InactivePageTable, temp_page: &mut temp_page::TemporaryPage, f: F) where F: FnOnce(&mut Mapper) {
    use x86_64::instructions::tlb;
    use x86_64::registers::control_regs;

    {
      let backup = Frame::containing_address(control_regs::cr3().0 as usize);

      // map temp page to current p4
      debug!("cr3 {:?}", backup);                                        // [[ TODO:: Find erroneous extra call to map ]] //
      let p4_table = temp_page.map_table_frame(backup.clone(), self);

      debug!("loading r-map");
      // overwrite r-mapping
      self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
      tlb::flush_all();

      debug!("running context fn");
      f(self);

      // restore r-mapping
      p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
      tlb::flush_all()
    }

    temp_page.unmap(self);
  }

  pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
    use x86_64::PhysicalAddress;
    use x86_64::registers::control_regs;

    let old_table = InactivePageTable {
      p4_frame: Frame::containing_address(control_regs::cr3().0 as usize)
    };

    unsafe { control_regs::cr3_write(PhysicalAddress(new_table.p4_frame.start_address() as u64)); }

    old_table
  }
}

// remap whole kernel, yay
pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) where A: FrameAllocator {
  let mut temporary_page = temp_page::TemporaryPage::new(Page {
    number: 0xDEADBEEF // ^.^
  }, allocator);

  let mut active_table = unsafe {ActivePageTable::new() };
  let mut new_table = {
    let frame = allocator.allocate_frame().expect("no more frames");
    debug!("New: {:?}", frame);
    InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
  };

  active_table.with(&mut new_table, &mut temporary_page, |mapper| {
    let elf_sections_tag = boot_info.elf_sections_tag().expect("ELF sections tag required");

    for section in elf_sections_tag.sections() {
      use self::entry::EntryFlags;

      if !section.is_allocated() {
        continue; // section is not loaded
      }

      assert!(section.start_address() % PAGE_SIZE == 0, "sections must be page aligned");

      debug!("Mapping section at 0x{:x}, size: 0x{:x}", section.addr, section.size);

      let flags = EntryFlags::from_elf_section_flags(section);

      let start_frame = Frame::containing_address(section.start_address());
      let end_frame = Frame::containing_address(section.end_address() - 1);
      for frame in Frame::range_inclusive(start_frame, end_frame) {
        mapper.identity_map(frame, flags, allocator);
      }
    }

    let vga_buffer_frame = Frame::containing_address(0xB8000); // map vga buffer
    mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator);

    let mb_start = Frame::containing_address(boot_info.start_address()); // map multiboot info structure
    let mb_end = Frame::containing_address(boot_info.end_address() - 1);
    for frame in Frame::range_inclusive(mb_start, mb_end) {
      mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
    }
  });

  let old_table = active_table.switch(new_table);

  let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
  active_table.unmap(old_p4_page, allocator);

}