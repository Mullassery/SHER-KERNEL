// SHER Driver Runtime: Sandbox Enforcement
// Security isolation for driver execution

use sher_common::{ObjectId, Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// SANDBOX POLICIES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Permissive,  // Minimal restrictions (native SHER drivers)
    Restricted,  // Standard restrictions (most Linux drivers)
    Strict,      // Maximum restrictions (untrusted/new drivers)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallPolicy {
    pub security_level: SecurityLevel,
    pub allowed_syscalls: Vec<String>,
    pub blocked_syscalls: Vec<String>,
    pub restricted_syscalls: Vec<RestrictedSyscall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedSyscall {
    pub name: String,
    pub allowed_args: Vec<u32>,  // Whitelisted argument values
    pub rate_limit: u32,          // Calls per second
}

impl Default for SyscallPolicy {
    fn default() -> Self {
        SyscallPolicy {
            security_level: SecurityLevel::Restricted,
            allowed_syscalls: vec![
                "read".to_string(),
                "write".to_string(),
                "open".to_string(),
                "close".to_string(),
                "ioctl".to_string(),
                "poll".to_string(),
                "mmap".to_string(),
                "munmap".to_string(),
            ],
            blocked_syscalls: vec![
                "ptrace".to_string(),
                "process_vm_readv".to_string(),
                "process_vm_writev".to_string(),
                "syslog".to_string(),
                "kexec_load".to_string(),
            ],
            restricted_syscalls: vec![
                RestrictedSyscall {
                    name: "fork".to_string(),
                    allowed_args: vec![],
                    rate_limit: 0,
                },
                RestrictedSyscall {
                    name: "exec".to_string(),
                    allowed_args: vec![],
                    rate_limit: 0,
                },
            ],
        }
    }
}

// ============================================================================
// NAMESPACE ISOLATION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Namespace {
    Pid,
    Network,
    Ipc,
    Uts,
    MountPoint,
    User,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamespacePolicy {
    pub enabled_namespaces: Vec<Namespace>,
    pub share_network: bool,
    pub share_ipc: bool,
}

// ============================================================================
// FILE DESCRIPTOR POLICY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDescriptorPolicy {
    pub max_file_descriptors: u32,
    pub allowed_file_paths: Vec<String>,
    pub blocked_file_paths: Vec<String>,
    pub read_only_paths: Vec<String>,
}

impl Default for FileDescriptorPolicy {
    fn default() -> Self {
        FileDescriptorPolicy {
            max_file_descriptors: 256,
            allowed_file_paths: vec![
                "/sys/bus/pci/*".to_string(),
                "/dev/mem".to_string(),
                "/dev/kmem".to_string(),
            ],
            blocked_file_paths: vec![
                "/etc/shadow".to_string(),
                "/root/*".to_string(),
            ],
            read_only_paths: vec![
                "/sys/*".to_string(),
                "/proc/*".to_string(),
            ],
        }
    }
}

// ============================================================================
// SANDBOX POLICY ENFORCEMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub driver_id: ObjectId,
    pub security_level: SecurityLevel,
    pub syscall_policy: SyscallPolicy,
    pub namespace_policy: NamespacePolicy,
    pub fd_policy: FileDescriptorPolicy,
    pub memory_limit_bytes: u64,
    pub enabled: bool,
}

impl SandboxPolicy {
    pub fn new(driver_id: ObjectId, level: SecurityLevel) -> Self {
        SandboxPolicy {
            driver_id,
            security_level: level,
            syscall_policy: SyscallPolicy::default(),
            namespace_policy: NamespacePolicy::default(),
            fd_policy: FileDescriptorPolicy::default(),
            memory_limit_bytes: 256 * 1024 * 1024,
            enabled: true,
        }
    }

    pub fn check_syscall(&self, syscall: &str) -> Result<bool> {
        if !self.enabled {
            return Ok(true);
        }

        if self.syscall_policy.blocked_syscalls.contains(&syscall.to_string()) {
            return Err(Error::AllocationFailed(format!("Syscall {} blocked by policy", syscall)));
        }

        if self.syscall_policy.allowed_syscalls.contains(&syscall.to_string()) {
            return Ok(true);
        }

        // Check if in restricted list
        if let Some(restricted) = self.syscall_policy.restricted_syscalls.iter().find(|r| r.name == syscall) {
            if restricted.rate_limit == 0 {
                return Err(Error::AllocationFailed(format!("Syscall {} not allowed", syscall)));
            }
            return Ok(true);
        }

        // Default deny
        Err(Error::AllocationFailed(format!("Syscall {} not whitelisted", syscall)))
    }

    pub fn check_file_access(&self, path: &str, write: bool) -> Result<bool> {
        if !self.enabled {
            return Ok(true);
        }

        // Check blocked paths
        for blocked in &self.fd_policy.blocked_file_paths {
            if path.starts_with(blocked.trim_end_matches('*')) {
                return Err(Error::AllocationFailed(format!("Access to {} blocked", path)));
            }
        }

        // Check read-only paths
        if write {
            for ro_path in &self.fd_policy.read_only_paths {
                if path.starts_with(ro_path.trim_end_matches('*')) {
                    return Err(Error::AllocationFailed(format!("Write to read-only path {}", path)));
                }
            }
        }

        // Check allowed paths
        for allowed in &self.fd_policy.allowed_file_paths {
            if path.starts_with(allowed.trim_end_matches('*')) {
                return Ok(true);
            }
        }

        // Default deny
        Err(Error::AllocationFailed(format!("Access to {} not allowed", path)))
    }
}

// ============================================================================
// SANDBOX MANAGER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SandboxManager {
    pub policies: HashMap<ObjectId, SandboxPolicy>,
    pub syscall_log: HashMap<ObjectId, Vec<SyscallEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallEntry {
    pub syscall: String,
    pub timestamp: u64,
    pub allowed: bool,
}

impl SandboxManager {
    pub fn new() -> Self {
        SandboxManager::default()
    }

    pub fn register_policy(&mut self, policy: SandboxPolicy) {
        let driver_id = policy.driver_id;
        self.policies.insert(driver_id, policy);
        self.syscall_log.insert(driver_id, Vec::new());
    }

    pub fn get_policy(&self, driver_id: ObjectId) -> Option<&SandboxPolicy> {
        self.policies.get(&driver_id)
    }

    pub fn check_syscall(&mut self, driver_id: ObjectId, syscall: &str) -> Result<bool> {
        if let Some(policy) = self.policies.get(&driver_id) {
            let allowed = policy.check_syscall(syscall).is_ok();

            // Log syscall
            if let Some(log) = self.syscall_log.get_mut(&driver_id) {
                log.push(SyscallEntry {
                    syscall: syscall.to_string(),
                    timestamp: 0,  // Would be set to current time
                    allowed,
                });
            }

            policy.check_syscall(syscall)
        } else {
            Err(Error::AllocationFailed(format!("Driver {} not found", driver_id)))
        }
    }

    pub fn get_syscall_log(&self, driver_id: ObjectId) -> Option<&Vec<SyscallEntry>> {
        self.syscall_log.get(&driver_id)
    }

    pub fn enable_sandbox(&mut self, driver_id: ObjectId) -> Result<()> {
        if let Some(policy) = self.policies.get_mut(&driver_id) {
            policy.enabled = true;
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!("Driver {} not found", driver_id)))
        }
    }

    pub fn disable_sandbox(&mut self, driver_id: ObjectId) -> Result<()> {
        if let Some(policy) = self.policies.get_mut(&driver_id) {
            policy.enabled = false;
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!("Driver {} not found", driver_id)))
        }
    }
}

// ============================================================================
// CAPABILITY ENFORCEMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub capabilities: HashSet<String>,
    pub denied_capabilities: HashSet<String>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        CapabilitySet {
            capabilities: HashSet::new(),
            denied_capabilities: HashSet::new(),
        }
    }

    pub fn grant(&mut self, capability: &str) {
        self.capabilities.insert(capability.to_string());
        self.denied_capabilities.remove(capability);
    }

    pub fn deny(&mut self, capability: &str) {
        self.denied_capabilities.insert(capability.to_string());
        self.capabilities.remove(capability);
    }

    pub fn has(&self, capability: &str) -> bool {
        self.capabilities.contains(capability) && !self.denied_capabilities.contains(capability)
    }

    pub fn check(&self, capability: &str) -> Result<()> {
        if self.has(capability) {
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!("Capability {} not granted", capability)))
        }
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}
