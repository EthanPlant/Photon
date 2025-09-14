use core::{arch::asm, ptr::addr_of};

use crate::arch::x86_64::PrivilegeLevel;

const GDT_ENTRIES: usize = 3;

const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Kernel);
const KERNEL_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Kernel);

// We need to specify to the linker that this should be in the `.data` segment
// as otherwise the GDT will get put in `.rodata` which gets mapped to a readonly page
// and panics when the CPU attempts to write the accessed flag
#[unsafe(link_section = ".data.gdt")]
static GDT: [GdtEntry; GDT_ENTRIES] = [
    // Null descriptor
    GdtEntry::new(0, GdtEntryFlags::empty()),
    // Kernel code segment
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            | GdtAccessFlags::KERNEL
            | GdtAccessFlags::DESCRIPTOR_TYPE
            | GdtAccessFlags::EXECUTABLE
            | GdtAccessFlags::RW,
        GdtEntryFlags::LONG_MODE,
    ),
    // Kernel data segment
    GdtEntry::new(
        GdtAccessFlags::PRESENT
            | GdtAccessFlags::KERNEL
            | GdtAccessFlags::DESCRIPTOR_TYPE
            | GdtAccessFlags::RW,
        GdtEntryFlags::LONG_MODE,
    ),
];

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone)]
    struct GdtEntryFlags: u8 {
        const PROTECTED_MODE = 1 << 6;
        const LONG_MODE = 1 << 5;
    }
}

#[derive(Debug, Copy, Clone)]
struct GdtAccessFlags;

impl GdtAccessFlags {
    const RW: u8 = 1 << 1;
    const DESCRIPTOR_TYPE: u8 = 1 << 4;
    const EXECUTABLE: u8 = 1 << 3;
    const KERNEL: u8 = 0 << 5;
    const PRESENT: u8 = 1 << 7;
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    limit_high_flags: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn new(access: u8, flags: GdtEntryFlags) -> Self {
        Self {
            limit_low: 0x00,
            base_low: 0x00,
            base_middle: 0x00,
            access,
            limit_high_flags: flags.bits() & 0xF0,
            base_high: 0x00,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

impl GdtDescriptor {
    const fn new(size: u16, offset: u64) -> Self {
        Self { size, offset }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct SegmentSelector(u16);

impl SegmentSelector {
    const fn new(index: u16, privilege: PrivilegeLevel) -> Self {
        Self(index << 3 | (privilege as u16))
    }
}

pub fn init() {
    // Truncation is never possible here, as the GDT has a hard limit of 65536 bytes
    // which is the maximum value storable in a u16
    #[allow(clippy::cast_possible_truncation)]
    let descriptor = GdtDescriptor::new(
        (core::mem::size_of::<[GdtEntry; GDT_ENTRIES]>() - 1) as u16,
        addr_of!(GDT) as u64,
    );

    log::debug!("GDT Descriptor: {descriptor:x?}");

    // Safety: The GDT is valid
    unsafe {
        load_gdt(&descriptor);

        load_cs(KERNEL_CODE_SELECTOR);
        load_ds(KERNEL_DATA_SELECTOR);
        load_es(KERNEL_DATA_SELECTOR);
        load_fs(KERNEL_DATA_SELECTOR);
        load_fs(KERNEL_DATA_SELECTOR);
        load_gs(KERNEL_DATA_SELECTOR);
        load_ss(KERNEL_DATA_SELECTOR);
    }
}

unsafe fn load_gdt(descriptor: &GdtDescriptor) {
    unsafe {
        asm!("lgdt[{}]", in(reg) descriptor, options(nostack));
    }
}

unsafe fn load_cs(selector: SegmentSelector) {
    unsafe {
        asm!(
            "push {selector}", 
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            selector = in(reg) u64::from(selector.0), tmp=lateout(reg) _);
    }
}

unsafe fn load_ds(selector: SegmentSelector) {
    unsafe { asm!("mov ds, {0:x}", in(reg) selector.0) };
}

unsafe fn load_es(selector: SegmentSelector) {
    unsafe { asm!("mov es, {0:x}", in(reg) selector.0) };
}

unsafe fn load_fs(selector: SegmentSelector) {
    unsafe { asm!("mov fs, {0:x}", in(reg) selector.0) };
}

unsafe fn load_gs(selector: SegmentSelector) {
    unsafe { asm!("mov gs, {0:x}", in(reg) selector.0) };
}

unsafe fn load_ss(selector: SegmentSelector) {
    unsafe { asm!("mov ss, {0:x}", in(reg) selector.0) };
}
