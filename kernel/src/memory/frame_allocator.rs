use core::marker::PhantomData;

use crate::memory::{
    frame::{Frame, FrameSize, FrameSize4K},
    mem_map::{EntryType, MemoryMapEntry},
};

static mut ENTRIES_BUF: [MemoryMapEntry; 64] = [MemoryMapEntry::zeroed(); 64];

pub unsafe trait FrameAllocator<S: FrameSize> {
    fn allocate_frame(&mut self) -> Option<Frame<S>>;
    unsafe fn deallocate_frame(&self, frame: Frame<S>);
}

#[derive(Debug, Clone)]
pub struct BumpFrameAllocator<S: FrameSize = FrameSize4K> {
    mem_map: &'static [MemoryMapEntry],
    next: usize,
    size: PhantomData<S>,
}

impl<S: FrameSize> BumpFrameAllocator<S> {
    pub fn init(mem_map: &limine::response::MemoryMapResponse) -> Self {
        let entries = mem_map.entries();
        let count = entries.len();

        for (i, entry) in entries.iter().enumerate() {
            // Safety: A data-race cannot occur as this is the only thing running
            unsafe {
                ENTRIES_BUF[i] = MemoryMapEntry::from(**entry);
            }
        }

        // Safety: A data-race cannot occur as this is the only thing running
        let slice = unsafe { &ENTRIES_BUF[..count] };

        Self {
            mem_map: slice,
            next: 0,
            size: PhantomData,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = Frame<S>> {
        self.mem_map
            .iter()
            .filter(|entry| entry.entry_type() == EntryType::Free)
            .map(|range| range.base..range.base + range.length)
            .flat_map(|range| range.step_by(S::SIZE as usize))
            .map(Frame::containing_addr)
    }
}

unsafe impl<S: FrameSize> FrameAllocator<S> for BumpFrameAllocator<S> {
    fn allocate_frame(&mut self) -> Option<Frame<S>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }

    unsafe fn deallocate_frame(&self, _frame: Frame<S>) {
        unimplemented!("Deallocation is invalid for a bump allocator")
    }
}
