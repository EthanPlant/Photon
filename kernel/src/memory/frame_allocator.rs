//! # Frame Allocator
//!
//! This module implements a simple **bump frame allocator** for physical memory
//! using the memory map provided by the Limine bootloader. It allocates fixed-size
//! frames by incrementally "bumping" through usable memory regions without tracking
//! freed frames (i.e. it cannot deallocate).
//!
//! ## Overview
//!
//! - Uses [`limine::memory_map`] to discover usable memory regions.
//! - Allocates frames of a fixed size (default: 4 KiB).
//! - Designed for early kernel initialization where a simple allocator is sufficient.
//! - Non-deallocating: memory can only be "freed" by resetting the entire allocator.
//!
//! ## Example
//!
//! ```rust
//! use crate::memory::frame_allocator;
//!
//! fn example() {
//!     let mut allocator = frame_allocator();
//!     let frame = allocator.allocate_frame().unwrap();
//!     log::info!("Allocated frame: {:?}", frame);
//! }
//! ```

use core::marker::PhantomData;

use limine::memory_map::{Entry, EntryType};
use spin::{Mutex, MutexGuard, Once};

use crate::{
    MEM_MAP_REQUEST,
    memory::{
        addr::{AddrError, PhysAddr},
        mem_map::{self, mmap_iter},
    },
};

/// A global singleton holding the [`BumpFrameAllocator`] wrapped in a [`Mutex`].
///
/// Initialized via [`init()`].
static FRAME_ALLOCATOR: Once<Mutex<BumpFrameAllocator>> = Once::new();

/// Errors that can occur during frame allocation.
#[derive(Debug, Clone, Copy)]
pub enum FrameAllocatorError {
    /// The frame size is invalid. Frame sizes must be a power of 2 to ensure proper address alignment
    InvalidFrameSize,
    /// No more free frames are available
    NoFreeFrames,
}

/// Represents a compile-time constant frame size.
///
/// Implemented by types such as [`FrameSize4K`]. Used by [`Frame`] and [`FrameAllocator`] to track the size of their frames.
pub trait FrameSize {
    /// Size of each frame in bytes.
    const SIZE: u64;
    /// Human-readable string describing the frame size (e.g. `"4 KiB"`).
    const SIZE_STR: &str;
}

/// Marker type for 4 KiB frames (the default page size on `x86_64`).
pub struct FrameSize4K;

impl FrameSize for FrameSize4K {
    const SIZE: u64 = 4096;
    const SIZE_STR: &str = "4 KiB";
}

#[derive(Clone)]
/// Represents a single frame of physical memory.
///
/// This is a typed handle to a frame of size `S::SIZE`. The [`start_addr()`]
/// gives its physical address.
pub struct Frame<S: FrameSize> {
    start_addr: PhysAddr,
    size: PhantomData<S>,
}

impl<S: FrameSize> Frame<S> {
    /// Creates a frame containing the given physical address, aligning
    /// it down to the start of its frame boundary.
    fn containing(addr: PhysAddr) -> Result<Self, AddrError> {
        Ok(Self {
            start_addr: addr.align_down(S::SIZE)?,
            size: PhantomData,
        })
    }

    // Returns the starting physical address of this frame.
    pub fn start_addr(&self) -> PhysAddr {
        self.start_addr
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

/// Generic trait for frame allocators.
///
/// # Safety
///
/// Implementors must ensure that:
/// - Frames that are returned by [`allocate_frame()`](FrameAllocator::allocate_frame) must be free and not in use by anything
/// - Any allocated frame must be valid until deallocated or the allocator is dropped.
pub unsafe trait FrameAllocator<S: FrameSize = FrameSize4K> {
    /// Allocate a single frame of size `S::SIZE`.
    ///
    /// # Errors
    ///
    /// This function returns [`FrameAllocatorError::NoFreeFrames`] if all free memory has been exhausted.
    fn allocate_frame(&mut self) -> Result<Frame<S>, FrameAllocatorError>;
    /// Deallocate a previously allocated frame.
    ///
    /// # Safety
    ///
    /// - `frame` must point to a valid frame that was allocated by this allocator
    /// - `frame` must no longer be in use
    #[allow(dead_code)] // Currently nothing deallocates a frame
    unsafe fn deallocate_frame(&mut self, frame: Frame<S>);
}

/// A simple bump allocator for physical frames.
///
/// Allocates frames by linearly advancing through memory regions discovered
/// in the memory map. When the current region is exhausted, it moves to the next
/// [`EntryType::USABLE`] region.
///
/// This allocator **does not support deallocation**
pub struct BumpFrameAllocator<S: FrameSize = FrameSize4K> {
    current_base: u64,
    current_end: u64,
    size: PhantomData<S>,
}

impl<S: FrameSize> BumpFrameAllocator<S> {
    // Create a new bump frame allocator using the first usable memory region.
    ///
    /// # Panics
    ///
    /// Panics if no usable memory regions are reported by the bootloader.
    pub fn new() -> Self {
        // Find the first free entry
        let first_entry = mmap_iter()
            .find(|entry| entry.entry_type == EntryType::USABLE)
            .expect("At least one free region of memory should be present");

        log::debug!(
            "First free entry {:x?} ({:?} bytes)",
            first_entry.base,
            first_entry.length
        );

        Self {
            current_base: first_entry.base,
            current_end: first_entry.base + first_entry.length,
            size: PhantomData,
        }
    }

    fn find_next(&self) -> Result<Entry, FrameAllocatorError> {
        mmap_iter()
            .filter(|entry| entry.base > self.current_end)
            .find(|entry| entry.entry_type == EntryType::USABLE)
            .ok_or(FrameAllocatorError::NoFreeFrames)
    }
}

unsafe impl<S: FrameSize> FrameAllocator<S> for BumpFrameAllocator<S> {
    fn allocate_frame(&mut self) -> Result<Frame<S>, FrameAllocatorError> {
        // First check if there's enough space in the current memory map entry for this frame
        if self.current_base + S::SIZE <= self.current_end {
            let addr = PhysAddr::new(self.current_base);
            self.current_base += S::SIZE;
            log::debug!("Allocating frame with address {addr:?}");
            return Frame::containing(addr).map_err(|_| FrameAllocatorError::InvalidFrameSize);
        }

        // Find next usable entry if current is exhausted
        let next_entry = self.find_next()?;

        log::debug!(
            "Next free entry {:x} ({})",
            next_entry.base,
            next_entry.length
        );

        let addr = PhysAddr::new(next_entry.base);
        self.current_base = next_entry.base + S::SIZE;
        self.current_end = next_entry.base + next_entry.length;

        Frame::containing(addr).map_err(|_| FrameAllocatorError::InvalidFrameSize)
    }

    unsafe fn deallocate_frame(&mut self, _frame: Frame<S>) {
        unimplemented!("Cannot deallocate with a bump allocator");
    }
}

/// Initializes the global frame allocator.
///
/// Must be called **once during early kernel initialization** before any calls
/// to [`frame_allocator()`]. This function:
///
/// - Initializes the memory map subsystem via [`mem_map::init()`].
/// - Constructs a global [`BumpFrameAllocator`].
///
/// If the frame allocator has already been initialized, This function does nothing.
///
/// # Panics
///
/// This function panics if no memory map was recieved from the bootloader
pub fn init() {
    mem_map::init(
        MEM_MAP_REQUEST
            .get_response()
            .expect("Should have recieved memory map from Limine"),
    );
    FRAME_ALLOCATOR.call_once(|| Mutex::new(BumpFrameAllocator::new()));
}

/// Returns a locked reference to the global [`BumpFrameAllocator`].
///
/// This function blocks if another thread currently holds the lock.
///
/// # Panics
///
/// Panics if [`init()`] has not yet been called.
pub fn frame_allocator() -> MutexGuard<'static, BumpFrameAllocator> {
    FRAME_ALLOCATOR
        .get()
        .expect("Frame allocator is initialized")
        .lock()
}
