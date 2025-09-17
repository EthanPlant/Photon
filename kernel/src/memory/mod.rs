//! Memory management module.
//!
//! This module provides abstractions and utilities for managing physical and virtual memory
//! within the kernel. It includes memory mapping, allocation, and other low-level memory
//! operations required for kernel functionality.
//!
//! Submodules:
//! - [`addr`]: Abstraction around physical and virtual addresses
//! - [`frame_allocator`] - Handles allocating and deallocating frames of physical memory.
//! - [`mem_map`]: Handles memory mapping and related operations.

mod addr;
pub mod frame_allocator;
pub mod mem_map;
