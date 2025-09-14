#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct MemoryMapEntry {
    base: u64,
    length: u64,
    entry_type: EntryType,
}

impl From<limine::memory_map::Entry> for MemoryMapEntry {
    fn from(value: limine::memory_map::Entry) -> Self {
        Self {
            base: value.base,
            length: value.length,
            entry_type: value.entry_type.into(),
        }
    }
}

pub fn parse_mem_map(mem_map_resp: &limine::response::MemoryMapResponse) {
    for entry in mem_map_resp.entries() {
        let entry = MemoryMapEntry::from(**entry);
        log::debug!("Memory Map Entry: {entry:?}");
    }
}
