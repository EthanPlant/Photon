//! # Address Abstraction
//!
//! This module provides type-safe abstractions for memory addresses in the kernel.
//!
//! The purpose of these types is to prevent accidental mixing of address spaces,
//! enforce proper alignment, and provide utilities for memory management.

/// Represents errors that can occur when manipulating memory addresses.
#[derive(Debug, Clone, Copy)]
pub enum AddrError {
    /// The requested alignment is invalid (alignment must be a power of two).
    InvalidAlignment,
}

/// A type-safe wrapper around a 64-bit **physical memory address**.
///
/// Prevents accidental mixing with virtual addresses or other integers and provides
/// alignment utilities. Supports low-level kernel memory management.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(u64);

impl PhysAddr {
    // Create a new `PhysAddr` from a raw `u64`.
    pub fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Aligns the address **down** to the nearest multiple of `align`.
    ///
    /// Returns an error if `align` is not a power-of-two.
    pub fn align_down(self, align: u64) -> Result<Self, AddrError> {
        if !align.is_power_of_two() {
            return Err(AddrError::InvalidAlignment);
        }

        let mask = align - 1;
        if self.0 & mask == 0 {
            Ok(self)
        } else {
            Ok(Self((self.0 | mask) + 1))
        }
    }
}

impl core::fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:x}", self.0))
            .finish()
    }
}
