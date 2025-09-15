//! # Bootstrap Allocator
//!
//! This module provides a simple, thread-safe bootstrap memory allocator for early kernel memory management.
//! It is intended for use before a full-featured memory allocator is available.
//!
//! ## Overview
//!
//! The `BootstrapAlloc` struct manages a mutable reference to a memory map (typically provided by the bootloader).
//! It allocates memory by carving out aligned regions from available memory map entries, but does **not** support deallocation.
//! The allocator is protected by a spinlock (`spin::Mutex`) to ensure safe concurrent access during early boot stages.
//!
//! The `BootstrapAllocRef` type is a lightweight reference wrapper that implements the [`Allocator`](core::alloc::Allocator) trait,
//! allowing it to be used with Rust's allocation APIs where an allocator is required.
//!
//! ## Limitations
//!
//! - **No deallocation:** Memory allocated by this allocator cannot be freed.
//! - **Intended for early boot:** Use only until a more sophisticated allocator is initialized.
//! - **Alignment:** All allocations are aligned to the requested alignment or the frame size, whichever is greater.
//!
//! ## Usage
//!
//! ```rust
//! let mut memory_map: &mut [MemoryMapEntry] = /* provided by bootloader */;
//! let bootstrap_alloc = BootstrapAlloc::new(memory_map);
//! let alloc_ref = BootstrapAllocRef::new(&bootstrap_alloc);
//! // Use `alloc_ref` as an allocator
//! ```
//!
//! ## Safety
//!
//! This allocator assumes exclusive access to the provided memory map and does not track individual allocations.
//! Use with caution and only during early initialization.

#![allow(dead_code)]

use core::{marker::PhantomData, ptr::NonNull};

use alloc::alloc::{AllocError, Allocator};
use spin::Mutex;

use crate::memory::{
    frame::{FrameSize, FrameSize4K},
    mem_map::{EntryType, MemoryMapEntry},
};

/// A simple, thread-safe bootstrap memory allocator for early kernel memory management.
///
/// `BootstrapAlloc` manages a mutable reference to a memory map (typically provided by the bootloader)
/// and allocates memory by carving out aligned regions from available memory map entries.
/// It is protected by a spinlock to ensure safe concurrent access during early boot stages.
/// This allocator does **not** support deallocation and is intended for use only until a full-featured
/// memory allocator is initialized.
///
/// # Type Parameters
/// - `S`: The frame size type, defaults to `FrameSize4K`.
#[derive(Debug)]
pub struct BootstrapAlloc<S: FrameSize = FrameSize4K> {
    pub memory_map: Mutex<&'static mut [MemoryMapEntry]>,
    size: PhantomData<S>,
}

impl<S: FrameSize> BootstrapAlloc<S> {
    /// Creates a new `BootstrapAlloc` using the provided memory map.
    ///
    /// # Example
    /// ```
    /// use crate::memory::bootstrap::BootstrapAlloc;
    /// use crate::memory::mem_map::MemoryMapEntry;
    ///
    /// // Assume `memory_map` is provided by the bootloader
    /// let memory_map: &'static mut [MemoryMapEntry] = /* ... */;
    /// let allocator = BootstrapAlloc::new(memory_map);
    /// ```
    pub fn new(memory_map: &'static mut [MemoryMapEntry]) -> Self {
        Self {
            memory_map: Mutex::new(memory_map),
            size: PhantomData,
        }
    }

    /// Allocates a block of memory of at least `size` bytes, aligned to the frame size.
    ///
    /// Returns a mutable pointer to the allocated memory, or `None` if there is no more free area in memory.
    ///
    /// # Example
    /// ```
    /// use crate::memory::bootstrap::BootstrapAlloc;
    /// use crate::memory::mem_map::MemoryMapEntry;
    ///
    /// let memory_map: &'static mut [MemoryMapEntry] = /* ... */;
    /// let allocator = BootstrapAlloc::new(memory_map);
    /// let ptr = allocator.allocate(4096);
    /// assert!(ptr.is_some());
    /// ```
    fn allocate(&self, size: u64) -> Option<*mut u8> {
        let aligned_size = align_up(size, S::SIZE);

        for range in self
            .memory_map
            .lock()
            .iter_mut()
            .filter(|range| range.entry_type() == EntryType::Free)
        {
            if range.length >= aligned_size {
                let addr = range.base;
                range.base += aligned_size;
                range.length -= aligned_size;
                return Some(addr.as_hhdm_virt().as_mut_ptr());
            }
        }

        None
    }
}

// A lightweight reference wrapper for [`BootstrapAlloc`] that implements the [`Allocator`](core::alloc::Allocator) trait.
///
/// `BootstrapAllocRef` allows the bootstrap allocator to be used with Rust's allocation APIs that require an allocator.
/// It provides safe, shared access to the underlying `BootstrapAlloc` instance.
///
/// # Type Parameters
/// - `'a`: Lifetime of the referenced `BootstrapAlloc`.
/// - `S`: The frame size type, defaults to `FrameSize4K`.
#[derive(Debug, Copy, Clone)]
pub struct BootstrapAllocRef<'a, S: FrameSize = FrameSize4K> {
    inner: &'a BootstrapAlloc<S>,
}

impl<'a, S: FrameSize> BootstrapAllocRef<'a, S> {
    /// Creates a new `BootstrapAllocRef` from a reference to a `BootstrapAlloc`.
    ///
    /// # Example
    /// ```
    /// use crate::memory::bootstrap::{BootstrapAlloc, BootstrapAllocRef};
    /// use crate::memory::mem_map::MemoryMapEntry;
    ///
    /// let memory_map: &'static mut [MemoryMapEntry] = /* ... */;
    /// let allocator = BootstrapAlloc::new(memory_map);
    /// let alloc_ref = BootstrapAllocRef::new(&allocator);
    /// ```
    pub fn new(inner: &'a BootstrapAlloc<S>) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying `BootstrapAlloc`.
    ///
    /// # Example
    /// ```
    /// use crate::memory::bootstrap::{BootstrapAlloc, BootstrapAllocRef};
    /// use crate::memory::mem_map::MemoryMapEntry;
    ///
    /// let memory_map: &'static mut [MemoryMapEntry] = /* ... */;
    /// let allocator = BootstrapAlloc::new(memory_map);
    /// let alloc_ref = BootstrapAllocRef::new(&allocator);
    /// let inner = alloc_ref.get_inner();
    /// ```
    fn get_inner(&self) -> &BootstrapAlloc<S> {
        self.inner
    }
}

// Safety: The bootstrap allocator enforces the following invariants:
// - The allocator only allocates memory that's marked as usable by the bootloader
// - Allocated memory always remains in use and is unusable by anything else
unsafe impl<S: FrameSize> Allocator for BootstrapAllocRef<'_, S> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, alloc::alloc::AllocError> {
        let aligned_size = align_up(layout.size() as u64, layout.align() as u64);
        let inner = self.get_inner();
        let ptr = inner.allocate(aligned_size);
        if ptr.is_none() {
            return Err(AllocError);
        }

        let ptr = ptr.unwrap();
        assert!(!ptr.is_null());
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        #[allow(clippy::cast_possible_truncation)]
        // We only run on 64-bit architectures, so usize is always 64-bits
        Ok(NonNull::slice_from_raw_parts(ptr, aligned_size as usize))
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        unimplemented!("Bootstrap allocator cannot deallocate")
    }
}

fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());

    let mask = align - 1;
    if addr & mask == 0 {
        addr
    } else {
        (addr | mask) + 1
    }
}
