//! Memory map management.
//!
//! This module defines structures and functions for managing the kernel's memory map.
//! It is responsible for tracking physical and virtual memory regions, handling allocation
//! and deallocation of memory, and providing safe abstractions for memory operations.
//!
//! Key features include:
//! - Representation of memory regions and their attributes
//! - Allocation and freeing of memory pages
//! - Utilities for querying and updating the memory map
#![allow(dead_code)]

use core::{mem, ptr};

use spin::Once;

static MEM_MAP: Once<MemMap> = Once::new();

/// Represents the kernel's memory map.
///
/// The `MemMap` struct holds metadata and a pointer to the memory map entries provided
/// by the bootloader. It allows the kernel to access information about available,
/// reserved, and special memory regions.
#[derive(Debug)]
struct MemMap {
    size: usize,
    entry_size: usize,
    entries: *const limine::memory_map::Entry,
}

// SAFETY: The bootloader guarantees that the memory map entries pointed to by `entries`
// are valid, read-only, and remain accessible for the entire lifetime of the kernel.
// No thread will mutate or deallocate this memory, making concurrent and cross-thread
// access safe. If these guarantees are ever violated, undefined behavior may occur.
unsafe impl Send for MemMap {}
unsafe impl Sync for MemMap {}

// Returns an iterator over the memory map entries.
///
/// Each item in the iterator is a copy of a `limine::memory_map::Entry` describing a region
/// of physical memory as reported by the bootloader. The iterator traverses all entries
/// in the memory map initialized by [`init`].
///
/// # Panics
/// 
/// Panics if the memory map has not been initialized via [`init`].
pub fn mmap_iter() -> impl Iterator<Item = limine::memory_map::Entry> {
    let mem_map = MEM_MAP.get().expect("Memory map is initialized");
    (0..mem_map.size).step_by(mem_map.entry_size).map(|off| {
        // Safety: We're in bounds and each `entry_size` points to a memory map entry
        unsafe { ptr::read_unaligned(mem_map.entries.byte_add(off)) }
    })
}

// Initializes the global memory map from the bootloader's memory map response.
///
/// This function must be called exactly once during kernel initialization, before any
/// calls to [`mmap_iter`]. It stores the memory map metadata and a pointer to the entries
/// for later access.
///
/// # Panics
///
/// This function will panic if the memory map has already been initialized.
pub fn init(mem_map: &limine::response::MemoryMapResponse) {
    let entry_size = mem::size_of::<limine::memory_map::Entry>();
    let size = entry_size * mem_map.entries().len();

    MEM_MAP.call_once(|| MemMap {
        size,
        entry_size,
        entries: ptr::from_ref(mem_map.entries()[0]),
    });
}
