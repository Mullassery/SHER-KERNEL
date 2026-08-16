//! Syscall Hardening and Filtering
//!
//! Attack surface reduction through:
//! - Syscall whitelisting
//! - Parameter validation
//! - Return value checking
//! - Suspicious pattern detection
//! - Rate limiting

use sher_common::{ObjectId, Result};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SyscallType {
    Read,
    Write,
    Open,
    Close,
    Stat,
    Mmap,
    Mprotect,
    Brk,
    Exit,
    Fork,
    Execve,
    Unknown(usize),
}

#[derive(Clone, Debug)]
pub struct SyscallPolicy {
    pub allowed_syscalls: HashSet<SyscallType>,
    pub max_calls_per_second: usize,
    pub validate_parameters: bool,
    pub validate_return_values: bool,
}

impl Default for SyscallPolicy {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert(SyscallType::Read);
        allowed.insert(SyscallType::Write);
        allowed.insert(SyscallType::Close);
        allowed.insert(SyscallType::Exit);

        SyscallPolicy {
            allowed_syscalls: allowed,
            max_calls_per_second: 10000,
            validate_parameters: true,
            validate_return_values: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SyscallAudit {
    pub syscall: SyscallType,
    pub count: u64,
    pub blocked: u64,
    pub parameter_violations: u64,
    pub return_value_violations: u64,
}

pub struct SyscallHardener {
    policy: SyscallPolicy,
    audits: HashMap<ObjectId, HashMap<SyscallType, SyscallAudit>>,
    call_times: HashMap<ObjectId, Vec<std::time::Instant>>,
}

impl SyscallHardener {
    pub fn new(policy: SyscallPolicy) -> Self {
        SyscallHardener {
            policy,
            audits: HashMap::new(),
            call_times: HashMap::new(),
        }
    }

    pub fn validate_syscall(&mut self, driver_id: &ObjectId, syscall: SyscallType) -> Result<()> {
        if !self.policy.allowed_syscalls.contains(&syscall) {
            self.record_blocked(driver_id, syscall.clone());
            return Err(sher_common::Error::Security(
                "Syscall not whitelisted".to_string(),
            ));
        }

        if !self.check_rate_limit(driver_id)? {
            return Err(sher_common::Error::Security(
                "Syscall rate limit exceeded".to_string(),
            ));
        }

        self.record_call(driver_id, syscall);
        Ok(())
    }

    pub fn validate_parameters(&self, _syscall: &SyscallType, params: &[u64]) -> Result<()> {
        if !self.policy.validate_parameters {
            return Ok(());
        }

        for param in params {
            if *param == 0 {
                return Err(sher_common::Error::Security(
                    "Null pointer parameter".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn validate_return_value(&self, _syscall: &SyscallType, return_value: i64) -> Result<()> {
        if !self.policy.validate_return_values {
            return Ok(());
        }

        if return_value < -130 {
            return Err(sher_common::Error::Security(
                "Invalid return value (out of range)".to_string(),
            ));
        }

        Ok(())
    }

    pub fn add_allowed_syscall(&mut self, syscall: SyscallType) {
        self.policy.allowed_syscalls.insert(syscall);
    }

    pub fn remove_allowed_syscall(&mut self, syscall: SyscallType) {
        self.policy.allowed_syscalls.remove(&syscall);
    }

    pub fn get_audit(&self, driver_id: &ObjectId, syscall: &SyscallType) -> Option<SyscallAudit> {
        self.audits
            .get(driver_id)
            .and_then(|audits| audits.get(syscall))
            .cloned()
    }

    pub fn get_driver_audit(&self, driver_id: &ObjectId) -> HashMap<SyscallType, SyscallAudit> {
        self.audits.get(driver_id).cloned().unwrap_or_default()
    }

    fn record_call(&mut self, driver_id: &ObjectId, syscall: SyscallType) {
        let audit = self
            .audits
            .entry(*driver_id)
            .or_default()
            .entry(syscall)
            .or_insert_with(|| SyscallAudit {
                syscall: SyscallType::Unknown(0),
                count: 0,
                blocked: 0,
                parameter_violations: 0,
                return_value_violations: 0,
            });

        audit.count += 1;
    }

    fn record_blocked(&mut self, driver_id: &ObjectId, syscall: SyscallType) {
        let audit = self
            .audits
            .entry(*driver_id)
            .or_default()
            .entry(syscall)
            .or_insert_with(|| SyscallAudit {
                syscall: SyscallType::Unknown(0),
                count: 0,
                blocked: 0,
                parameter_violations: 0,
                return_value_violations: 0,
            });

        audit.blocked += 1;
    }

    fn check_rate_limit(&mut self, driver_id: &ObjectId) -> Result<bool> {
        let now = std::time::Instant::now();
        let one_sec_ago = now - std::time::Duration::from_secs(1);

        let times = self.call_times.entry(*driver_id).or_default();

        times.retain(|&t| t > one_sec_ago);

        if times.len() >= self.policy.max_calls_per_second {
            return Ok(false);
        }

        times.push(now);
        Ok(true)
    }

    pub fn reset(&mut self, driver_id: &ObjectId) {
        self.audits.remove(driver_id);
        self.call_times.remove(driver_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_policy_default() {
        let policy = SyscallPolicy::default();
        assert!(policy.allowed_syscalls.contains(&SyscallType::Read));
    }

    #[test]
    fn test_allowed_syscall() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);
        let driver_id = ObjectId::new();

        let result = hardener.validate_syscall(&driver_id, SyscallType::Read);
        assert!(result.is_ok());
    }

    #[test]
    fn test_blocked_syscall() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);
        let driver_id = ObjectId::new();

        let result = hardener.validate_syscall(&driver_id, SyscallType::Fork);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation() {
        let policy = SyscallPolicy::default();
        let hardener = SyscallHardener::new(policy);

        let result = hardener.validate_parameters(&SyscallType::Read, &[0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_return_value_validation() {
        let policy = SyscallPolicy::default();
        let hardener = SyscallHardener::new(policy);

        let result = hardener.validate_return_value(&SyscallType::Read, -5000);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_syscall() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);

        hardener.add_allowed_syscall(SyscallType::Fork);
        let driver_id = ObjectId::new();
        let result = hardener.validate_syscall(&driver_id, SyscallType::Fork);

        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_syscall() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);
        let driver_id = ObjectId::new();

        hardener.remove_allowed_syscall(SyscallType::Read);
        let result = hardener.validate_syscall(&driver_id, SyscallType::Read);

        assert!(result.is_err());
    }

    #[test]
    fn test_audit_tracking() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);
        let driver_id = ObjectId::new();

        let _ = hardener.validate_syscall(&driver_id, SyscallType::Read);
        let audit = hardener.get_audit(&driver_id, &SyscallType::Read);

        assert!(audit.is_some());
    }

    #[test]
    fn test_reset() {
        let policy = SyscallPolicy::default();
        let mut hardener = SyscallHardener::new(policy);
        let driver_id = ObjectId::new();

        let _ = hardener.validate_syscall(&driver_id, SyscallType::Read);
        hardener.reset(&driver_id);

        let audit = hardener.get_audit(&driver_id, &SyscallType::Read);
        assert!(audit.is_none());
    }
}
