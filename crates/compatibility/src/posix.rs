//! POSIX compatibility layer: translates a known set of POSIX syscall names
//! to the SHER primitive that would handle them.
//!
//! Real, tested lookup table — not a POSIX-compliant syscall implementation.
//! Load on first POSIX syscall, per the crate root docs.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SherTarget {
    ObjectManager,
    StorageSubsystem,
    NetworkingSubsystem,
    SchedulerSubsystem,
    MemoryAllocator,
}

fn translation_table() -> HashMap<&'static str, SherTarget> {
    let mut table = HashMap::new();
    table.insert("open", SherTarget::StorageSubsystem);
    table.insert("read", SherTarget::StorageSubsystem);
    table.insert("write", SherTarget::StorageSubsystem);
    table.insert("close", SherTarget::StorageSubsystem);
    table.insert("socket", SherTarget::NetworkingSubsystem);
    table.insert("bind", SherTarget::NetworkingSubsystem);
    table.insert("connect", SherTarget::NetworkingSubsystem);
    table.insert("fork", SherTarget::ObjectManager);
    table.insert("exec", SherTarget::ObjectManager);
    table.insert("exit", SherTarget::ObjectManager);
    table.insert("sched_yield", SherTarget::SchedulerSubsystem);
    table.insert("nanosleep", SherTarget::SchedulerSubsystem);
    table.insert("mmap", SherTarget::MemoryAllocator);
    table.insert("munmap", SherTarget::MemoryAllocator);
    table.insert("brk", SherTarget::MemoryAllocator);
    table
}

/// Translate a POSIX syscall name to the SHER subsystem that implements it.
pub fn translate(posix_call_name: &str) -> Option<SherTarget> {
    translation_table().get(posix_call_name).copied()
}

pub fn is_supported(posix_call_name: &str) -> bool {
    translate(posix_call_name).is_some()
}

pub fn supported_calls() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = translation_table().into_keys().collect();
    names.sort_unstable();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_calls_translate_to_storage() {
        assert_eq!(translate("open"), Some(SherTarget::StorageSubsystem));
        assert_eq!(translate("read"), Some(SherTarget::StorageSubsystem));
    }

    #[test]
    fn socket_calls_translate_to_networking() {
        assert_eq!(translate("socket"), Some(SherTarget::NetworkingSubsystem));
    }

    #[test]
    fn memory_calls_translate_to_memory_allocator() {
        assert_eq!(translate("mmap"), Some(SherTarget::MemoryAllocator));
    }

    #[test]
    fn unknown_call_is_unsupported() {
        assert!(!is_supported("not_a_real_syscall"));
    }
}
