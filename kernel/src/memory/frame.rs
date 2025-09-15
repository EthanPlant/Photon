use core::marker::PhantomData;

use crate::memory::addr::PhysAddr;

pub trait FrameSize: Copy + Eq + Ord {
    const SIZE: u64;
    const SIZE_STR: &'static str;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameSize4K;

impl FrameSize for FrameSize4K {
    const SIZE: u64 = 4096;
    const SIZE_STR: &'static str = "4 KiB";
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Frame<S: FrameSize = FrameSize4K> {
    start: PhysAddr,
    size: PhantomData<S>,
}

impl<S: FrameSize> Frame<S> {
    pub fn containing_addr(addr: PhysAddr) -> Self {
        Self {
            start: addr
                .align_down(S::SIZE)
                .expect("S::SIZE is a valid power of 2"),
            size: PhantomData,
        }
    }

    pub fn start_addr(self) -> PhysAddr {
        self.start
    }
}

impl<S: FrameSize> core::fmt::Debug for Frame<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Frame[{}]({:?})",
            S::SIZE_STR,
            self.start_addr()
        ))
    }
}
