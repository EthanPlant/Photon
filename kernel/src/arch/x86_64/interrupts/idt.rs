use core::{arch::asm, ptr::addr_of};

use spin::Mutex;

use crate::arch::x86_64::{
    PrivilegeLevel,
    gdt::{KERNEL_CODE_SELECTOR, SegmentSelector},
};

const INTERRUPT_GATE: u8 = 0x0e;

const IDT_ENTRIES: usize = 256;

pub static IDT: Mutex<Idt> = Mutex::new(Idt::new());

#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct Idt {
    pub entries: [IdtEntry; IDT_ENTRIES],
}

impl Idt {
    const fn new() -> Self {
        Self {
            entries: [IdtEntry::EMPTY; IDT_ENTRIES],
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

impl IdtDescriptor {
    fn new(size: u16, offset: u64) -> Self {
        Self { size, offset }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct IdtEntryAttributes(u8);

impl IdtEntryAttributes {
    const fn new(privelege: PrivilegeLevel) -> Self {
        Self(1 << 7 | (privelege as u8) | INTERRUPT_GATE)
    }

    const fn kernel() -> Self {
        Self::new(PrivilegeLevel::Kernel)
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct IdtEntry {
    offset_low: u16,
    selector: SegmentSelector,
    ist: u8,
    attributes: IdtEntryAttributes,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    // Safety: While this appears to violate the safety condition of `new()`. This *is* safe as
    // we're setting the offset pointer to a null pointer, which will cause a page-fault if we ever jump to it.
    // This is guaranteed to be well-defined behaviour and is intentionally what we want as an IDT entry being empty
    // imples the interrupt hasn't been registered to the IDT.
    const EMPTY: Self = unsafe { Self::new(0, KERNEL_CODE_SELECTOR, IdtEntryAttributes::kernel()) };

    /// Creates a new IDT entry struct
    ///
    /// # Safety
    ///
    /// `offset` must be the memory address of a valid interrupt handler function,
    /// or else undefined behaviour will occur when this interrupt is triggered.
    const unsafe fn new(
        offset: usize,
        selector: SegmentSelector,
        attributes: IdtEntryAttributes,
    ) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        // We explicitely want truncation to occur in these casts
        Self {
            offset_low: offset as u16,
            selector,
            ist: 0,
            attributes,
            offset_middle: (offset >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            reserved: 0,
        }
    }
}

pub fn init() {
    // The IDT size will always be 4096 bytes
    #[allow(clippy::cast_possible_truncation)]
    let idt_descriptor = IdtDescriptor::new(
        (core::mem::size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
        addr_of!(IDT.lock().entries) as u64,
    );

    log::debug!("IDT Descriptor: {idt_descriptor:x?}");

    // Safety: The IDT is guaranteed to be valid
    unsafe {
        load_idt(&idt_descriptor);
    }
}

unsafe fn load_idt(descriptor: &IdtDescriptor) {
    unsafe {
        asm!("lidt [{}]", in(reg) descriptor, options(nostack));
    }
}
