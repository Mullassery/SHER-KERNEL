// SHER Driver Runtime: Linux Kernel Interface (LKI)
// Translates Linux kernel API calls to SHER primitives

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// LINUX KERNEL API TRANSLATION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinuxApiCall {
    // Memory API
    KMalloc,
    KZalloc,
    Vmalloc,
    Kfree,
    VFree,

    // Interrupt API
    RequestIrq,
    FreeIrq,
    EnableIrq,
    DisableIrq,

    // Device API
    PciDriverRegister,
    DeviceRegister,
    BusRegister,

    // IO API
    IORemap,
    IOUnmap,

    // Synchronization
    Mutex,
    Spinlock,

    // Workqueue
    ScheduleWork,
    CancelWork,

    // Module
    ModuleInit,
    ModuleExit,
}

impl LinuxApiCall {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinuxApiCall::KMalloc => "kmalloc",
            LinuxApiCall::KZalloc => "kzalloc",
            LinuxApiCall::Vmalloc => "vmalloc",
            LinuxApiCall::Kfree => "kfree",
            LinuxApiCall::VFree => "vfree",
            LinuxApiCall::RequestIrq => "request_irq",
            LinuxApiCall::FreeIrq => "free_irq",
            LinuxApiCall::EnableIrq => "enable_irq",
            LinuxApiCall::DisableIrq => "disable_irq",
            LinuxApiCall::PciDriverRegister => "pci_driver_register",
            LinuxApiCall::DeviceRegister => "device_register",
            LinuxApiCall::BusRegister => "bus_register",
            LinuxApiCall::IORemap => "ioremap",
            LinuxApiCall::IOUnmap => "iounmap",
            LinuxApiCall::Mutex => "mutex_lock",
            LinuxApiCall::Spinlock => "spin_lock",
            LinuxApiCall::ScheduleWork => "schedule_work",
            LinuxApiCall::CancelWork => "cancel_work",
            LinuxApiCall::ModuleInit => "module_init",
            LinuxApiCall::ModuleExit => "module_exit",
        }
    }
}

// ============================================================================
// SHER PRIMITIVE MAPPING
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SherPrimitive {
    // Memory
    MasterAllocate,
    MasterDeallocate,

    // Interrupts
    InterruptRegister,
    InterruptUnregister,
    InterruptEnable,
    InterruptDisable,

    // Devices
    DeviceRegistryAdd,
    DeviceRegistryUpdate,

    // IO
    MemoryMap,
    MemoryUnmap,

    // Synchronization
    LockAcquire,
    LockRelease,

    // Async
    WorkSchedule,
    WorkCancel,
}

impl SherPrimitive {
    pub fn as_str(&self) -> &'static str {
        match self {
            SherPrimitive::MasterAllocate => "master_allocate",
            SherPrimitive::MasterDeallocate => "master_deallocate",
            SherPrimitive::InterruptRegister => "interrupt_register",
            SherPrimitive::InterruptUnregister => "interrupt_unregister",
            SherPrimitive::InterruptEnable => "interrupt_enable",
            SherPrimitive::InterruptDisable => "interrupt_disable",
            SherPrimitive::DeviceRegistryAdd => "device_registry_add",
            SherPrimitive::DeviceRegistryUpdate => "device_registry_update",
            SherPrimitive::MemoryMap => "memory_map",
            SherPrimitive::MemoryUnmap => "memory_unmap",
            SherPrimitive::LockAcquire => "lock_acquire",
            SherPrimitive::LockRelease => "lock_release",
            SherPrimitive::WorkSchedule => "work_schedule",
            SherPrimitive::WorkCancel => "work_cancel",
        }
    }
}

// ============================================================================
// TRANSLATION MAPPING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMapping {
    pub linux_api: String,
    pub sher_primitive: String,
    pub validation_level: ValidationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    None,
    Basic,
    Strict,
}

// ============================================================================
// TRANSLATION ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct TranslationEngine {
    pub mappings: HashMap<String, (String, ValidationLevel)>,
    pub call_count: HashMap<String, u64>,
    pub error_count: HashMap<String, u64>,
}

impl TranslationEngine {
    pub fn new() -> Self {
        let mut engine = TranslationEngine::default();

        // Initialize default mappings
        engine.add_mapping(
            LinuxApiCall::KMalloc,
            SherPrimitive::MasterAllocate,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::Kfree,
            SherPrimitive::MasterDeallocate,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::Vmalloc,
            SherPrimitive::MasterAllocate,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::VFree,
            SherPrimitive::MasterDeallocate,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::RequestIrq,
            SherPrimitive::InterruptRegister,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::FreeIrq,
            SherPrimitive::InterruptUnregister,
            ValidationLevel::Strict,
        );
        engine.add_mapping(
            LinuxApiCall::PciDriverRegister,
            SherPrimitive::DeviceRegistryAdd,
            ValidationLevel::Basic,
        );

        engine
    }

    pub fn add_mapping(&mut self, linux_api: LinuxApiCall, sher: SherPrimitive, validation: ValidationLevel) {
        self.mappings.insert(
            linux_api.as_str().to_string(),
            (sher.as_str().to_string(), validation),
        );
    }

    pub fn translate(&self, linux_api: &str) -> Option<&String> {
        self.mappings
            .get(linux_api)
            .map(|(sher, _)| sher)
    }

    pub fn get_validation_level(&self, linux_api: &str) -> Option<ValidationLevel> {
        self.mappings
            .get(linux_api)
            .map(|(_, level)| *level)
    }

    pub fn record_call(&mut self, api: &str) {
        *self.call_count.entry(api.to_string()).or_insert(0) += 1;
    }

    pub fn record_error(&mut self, api: &str) {
        *self.error_count.entry(api.to_string()).or_insert(0) += 1;
    }

    pub fn get_call_count(&self, api: &str) -> u64 {
        self.call_count.get(api).copied().unwrap_or(0)
    }

    pub fn get_error_count(&self, api: &str) -> u64 {
        self.error_count.get(api).copied().unwrap_or(0)
    }

    pub fn get_total_calls(&self) -> u64 {
        self.call_count.values().sum()
    }

    pub fn get_total_errors(&self) -> u64 {
        self.error_count.values().sum()
    }

    pub fn get_success_rate(&self) -> f64 {
        let total = self.get_total_calls();
        if total == 0 {
            return 100.0;
        }
        let errors = self.get_total_errors();
        ((total - errors) as f64 / total as f64) * 100.0
    }

    pub fn list_mappings(&self) -> Vec<(&String, &String)> {
        self.mappings
            .iter()
            .map(|(linux, (sher, _))| (linux, sher))
            .collect()
    }
}
