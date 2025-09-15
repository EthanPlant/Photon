use crate::memory::addr::PhysAddr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntryType {
    Free,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    Bad,
    BootloaderReclaimable,
    ExecutableAndModules,
    Framebuffer,
}

impl From<limine::memory_map::EntryType> for EntryType {
    fn from(value: limine::memory_map::EntryType) -> Self {
        match value {
            limine::memory_map::EntryType::USABLE => Self::Free,
            limine::memory_map::EntryType::RESERVED => Self::Reserved,
            limine::memory_map::EntryType::ACPI_RECLAIMABLE => Self::AcpiReclaimable,
            limine::memory_map::EntryType::ACPI_NVS => Self::AcpiNvs,
            limine::memory_map::EntryType::BAD_MEMORY => Self::Bad,
            limine::memory_map::EntryType::BOOTLOADER_RECLAIMABLE => Self::BootloaderReclaimable,
            limine::memory_map::EntryType::EXECUTABLE_AND_MODULES => Self::ExecutableAndModules,
            limine::memory_map::EntryType::FRAMEBUFFER => Self::Framebuffer,
            _ => unreachable!("Invalid memory type from Limine"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    base: PhysAddr,
    length: u64,
    entry_type: EntryType,
}

impl MemoryMapEntry {
    pub const fn zeroed() -> Self {
        Self {
            base: PhysAddr::null(),
            length: 0,
            entry_type: EntryType::Reserved,
        }
    }

    pub fn base(&self) -> PhysAddr {
        self.base
    }

    pub fn len(&self) -> u64 {
        self.length
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry_type
    }
}

impl From<limine::memory_map::Entry> for MemoryMapEntry {
    fn from(value: limine::memory_map::Entry) -> Self {
        Self {
            base: PhysAddr::new(value.base),
            length: value.length,
            entry_type: value.entry_type.into(),
        }
    }
}
