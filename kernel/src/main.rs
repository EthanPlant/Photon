#![no_std]
#![no_main]
#![feature(step_trait)]
#![feature(allocator_api)]
#![warn(clippy::pedantic)]

use dummy_alloc::DummyAllocator;
use limine::{
    BaseRevision,
    request::{
        FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
    },
};

extern crate alloc;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(2);

#[used]
#[unsafe(link_section = ".requests")]
static MEM_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

/// We need the `alloc` crate for the bootstrap allocator which requires a global allocator to be defined.
/// Since we don't have a heap allocator yet, we just use [`DummyAllocator`](dummy_alloc::DummyAllocator)
/// for now.
#[global_allocator]
static GLOBAL_ALLOC: DummyAllocator = DummyAllocator;

mod arch;
mod drivers;
mod logger;
mod memory;

/// Kernel main function.
///
/// This function is called after the architecture-specific
/// initialization is complete to perform non-architecture-specific
/// setup and enter the main kernel loop.
///
/// # Panics
///
/// This function panics if the base revision is unsupported by the bootloader.
pub fn kmain() -> ! {
    log::debug!("Dropped into kmain!");
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().next()
    {
        for i in 0..100_u64 {
            // Calculate the pixel offset using the framebuffer information we obtained above.
            // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
            let pixel_offset = i * framebuffer.pitch() + i * 4;

            // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
            unsafe {
                // This is only temp code and we don't really care about clippy lints here
                #[allow(clippy::pedantic)]
                framebuffer
                    .addr()
                    .add(pixel_offset as usize)
                    .cast::<u32>()
                    .write(0xFFFF_FFFF);
            };
        }
    }

    arch::enable_interrupts();

    arch::halt()
}

/// Panic handler for the kernel.
///
/// This function is called when a panic occurs in the kernel.
/// It halts the CPU to prevent further execution.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!(
        "KERNEL PANIC: {} - {}",
        info.location().unwrap(),
        info.message()
    );
    arch::halt()
}
