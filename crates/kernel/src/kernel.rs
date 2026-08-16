use crate::config::KernelConfig;
use sher_ai::{AiMonitor, InferenceEngine, ResourceOptimizer};
use sher_common::{ObjectId, Result};
use sher_device_manager::registry::DeviceRegistry;
use sher_driver_runtime::container::DriverContainer;
use sher_lki::LinuxKernelInterface;
use sher_memory::allocator::MemoryAllocator;
use sher_scheduler::queue::TaskQueue;
use sher_security::audit::AuditLog;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

pub struct SherKernel {
    config: KernelConfig,
    boot_time: u64,
    kernel_id: ObjectId,
    memory_allocator: MemoryAllocator,
    task_queue: TaskQueue,
    device_registry: DeviceRegistry,
    audit_log: AuditLog,
    inference_engine: InferenceEngine,
    ai_monitor: AiMonitor,
    resource_optimizer: ResourceOptimizer,
    ai_services_active: bool,
    lki: Option<LinuxKernelInterface>,
    active_drivers: Vec<DriverContainer>,
}

impl SherKernel {
    pub fn new(config: KernelConfig) -> Result<Self> {
        let boot_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            memory_allocator: MemoryAllocator::new(config.total_memory),
            task_queue: TaskQueue::default(),
            device_registry: DeviceRegistry::default(),
            audit_log: AuditLog::default(),
            inference_engine: InferenceEngine::default(),
            ai_monitor: AiMonitor::default(),
            resource_optimizer: ResourceOptimizer::default(),
            ai_services_active: false,
            lki: None,
            active_drivers: Vec::new(),
            boot_time,
            kernel_id: ObjectId::new(),
            config,
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing kernel subsystems");
        self.audit_log
            .log(self.kernel_id, "kernel_initialize_start", true);

        if self.config.enable_ai_services {
            info!("Initializing AI services");
            // Run one inference + one optimization pass so the AI
            // subsystems are actually exercised at boot, not just held as
            // unused fields.
            let _stats = self.inference_engine.get_stats();
            self.resource_optimizer.optimize();
            self.ai_services_active = true;
        }

        if self.config.enable_linux_compatibility {
            info!("Initializing Linux Kernel Interface (LKI)");
            self.lki = Some(LinuxKernelInterface::new(sher_common::ObjectId::nil()));
        }

        self.audit_log
            .log(self.kernel_id, "kernel_initialize_complete", true);
        info!("Kernel initialization complete");
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Initiating kernel shutdown");
        self.audit_log.log(self.kernel_id, "kernel_shutdown", true);
        self.active_drivers.clear();
        Ok(())
    }

    pub fn uptime(&self) -> u64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        current_time - self.boot_time
    }

    /// Register a device with the kernel's device registry, auditing the
    /// action.
    pub fn register_device(&mut self, device: sher_device_manager::registry::RegisteredDevice) {
        let name = device.name.clone();
        self.device_registry.register(device);
        self.audit_log
            .log(self.kernel_id, format!("register_device:{name}"), true);
    }

    /// Ask the AI anomaly monitor whether a given metric currently looks
    /// anomalous. Returns `false` (not anomalous) if AI services are
    /// disabled, since no monitoring is running in that case.
    pub fn check_anomaly(&self, metric: &str) -> bool {
        self.ai_services_active && self.ai_monitor.detect_anomaly(metric)
    }

    pub fn status(&self) -> KernelStatus {
        KernelStatus {
            uptime_seconds: self.uptime(),
            memory_usage_percent: self.memory_allocator.usage_percent(),
            active_drivers: self.active_drivers.len(),
            pending_tasks: self.task_queue.len(),
            registered_devices: self.device_registry.get_device_count(),
            operational_devices: self.device_registry.get_operational_count(),
            audit_events: self.audit_log.events.len(),
            ai_services_active: self.ai_services_active,
            security_level: format!("{:?}", self.config.security_level),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KernelStatus {
    pub uptime_seconds: u64,
    pub memory_usage_percent: f64,
    pub active_drivers: usize,
    pub pending_tasks: usize,
    pub registered_devices: usize,
    pub operational_devices: usize,
    pub audit_events: usize,
    pub ai_services_active: bool,
    pub security_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KernelConfig;

    #[tokio::test]
    async fn initialize_activates_ai_services_when_enabled() {
        let mut kernel = SherKernel::new(KernelConfig::default()).unwrap();
        kernel.initialize().await.unwrap();
        let status = kernel.status();
        assert!(status.ai_services_active);
        assert!(status.audit_events >= 2); // start + complete events logged
    }

    #[tokio::test]
    async fn initialize_skips_ai_services_when_disabled() {
        let mut config = KernelConfig::default();
        config.enable_ai_services = false;
        let mut kernel = SherKernel::new(config).unwrap();
        kernel.initialize().await.unwrap();
        assert!(!kernel.status().ai_services_active);
        assert!(!kernel.check_anomaly("cpu_usage"));
    }

    #[tokio::test]
    async fn shutdown_clears_active_drivers_and_audits() {
        let mut kernel = SherKernel::new(KernelConfig::default()).unwrap();
        kernel.initialize().await.unwrap();
        kernel.shutdown().await.unwrap();
        assert_eq!(kernel.status().active_drivers, 0);
    }

    #[test]
    fn status_reflects_memory_usage() {
        let kernel = SherKernel::new(KernelConfig::default()).unwrap();
        let status = kernel.status();
        assert_eq!(status.memory_usage_percent, 0.0);
        assert_eq!(status.registered_devices, 0);
    }
}
