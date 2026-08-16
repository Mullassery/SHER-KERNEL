//! Linux compatibility layer: translates a known set of Linux kernel API
//! names to the SHER primitive that would handle them.
//!
//! This is a real, tested lookup/dispatch table — it is *not* a Linux
//! syscall ABI implementation or binary-compatible ELF loader; it exists so
//! a higher layer (e.g. `sher_lki`, which has the fuller translation
//! implementation) can consult a canonical name→target mapping. Load only
//! if a Linux driver is actually encountered, per the crate root docs.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SherTarget {
    MemoryAllocator,
    DmaManager,
    InterruptController,
    DeviceRegistry,
    BusHierarchy,
}

fn translation_table() -> HashMap<&'static str, SherTarget> {
    let mut table = HashMap::new();
    table.insert("kmalloc", SherTarget::MemoryAllocator);
    table.insert("kzalloc", SherTarget::MemoryAllocator);
    table.insert("vmalloc", SherTarget::MemoryAllocator);
    table.insert("kfree", SherTarget::MemoryAllocator);
    table.insert("vfree", SherTarget::MemoryAllocator);
    table.insert("dma_alloc_coherent", SherTarget::DmaManager);
    table.insert("request_irq", SherTarget::InterruptController);
    table.insert("free_irq", SherTarget::InterruptController);
    table.insert("enable_irq", SherTarget::InterruptController);
    table.insert("disable_irq", SherTarget::InterruptController);
    table.insert("pci_driver_register", SherTarget::DeviceRegistry);
    table.insert("pci_device_register", SherTarget::DeviceRegistry);
    table.insert("bus_register", SherTarget::BusHierarchy);
    table.insert("bus_add_device", SherTarget::BusHierarchy);
    table.insert("bus_add_driver", SherTarget::BusHierarchy);
    table
}

/// Translate a Linux kernel API name to the SHER subsystem that implements
/// it. Returns `None` for unknown/unsupported names.
pub fn translate(linux_api_name: &str) -> Option<SherTarget> {
    translation_table().get(linux_api_name).copied()
}

pub fn is_supported(linux_api_name: &str) -> bool {
    translate(linux_api_name).is_some()
}

pub fn supported_apis() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = translation_table().into_keys().collect();
    names.sort_unstable();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_memory_apis_translate_to_memory_allocator() {
        assert_eq!(translate("kmalloc"), Some(SherTarget::MemoryAllocator));
        assert_eq!(translate("kfree"), Some(SherTarget::MemoryAllocator));
    }

    #[test]
    fn known_irq_apis_translate_to_interrupt_controller() {
        assert_eq!(
            translate("request_irq"),
            Some(SherTarget::InterruptController)
        );
    }

    #[test]
    fn unknown_api_returns_none() {
        assert_eq!(translate("totally_made_up_syscall"), None);
        assert!(!is_supported("totally_made_up_syscall"));
    }

    #[test]
    fn supported_apis_are_sorted_and_nonempty() {
        let apis = supported_apis();
        assert!(!apis.is_empty());
        let mut sorted = apis.clone();
        sorted.sort_unstable();
        assert_eq!(apis, sorted);
    }
}
