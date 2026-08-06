//! Memory Safety Auditing and Validation
//!
//! Comprehensive memory safety checks including:
//! - Bounds checking
//! - Use-after-free detection
//! - Double-free prevention
//! - Memory leak tracking
//! - Alignment validation
//! - Overflow detection

use std::collections::{HashMap, HashSet};
use sher_common::{ObjectId, Result};

#[derive(Clone, Debug)]
pub struct MemorySafetyAudit {
    pub allocation_id: ObjectId,
    pub size: usize,
    pub alignment: usize,
    pub freed: bool,
    pub use_count: usize,
}

#[derive(Clone, Debug)]
pub struct AuditResult {
    pub passed: bool,
    pub total_checks: usize,
    pub failed_checks: usize,
    pub issues: Vec<String>,
}

pub struct MemorySafetyValidator {
    allocations: HashMap<ObjectId, MemorySafetyAudit>,
    freed_pointers: HashSet<ObjectId>,
    use_after_free_attempts: usize,
    double_free_attempts: usize,
}

impl MemorySafetyValidator {
    pub fn new() -> Self {
        MemorySafetyValidator {
            allocations: HashMap::new(),
            freed_pointers: HashSet::new(),
            use_after_free_attempts: 0,
            double_free_attempts: 0,
        }
    }

    pub fn register_allocation(&mut self, id: ObjectId, size: usize, alignment: usize) -> Result<()> {
        if size == 0 {
            return Err(sher_common::Error::Memory("Zero-size allocation not allowed".to_string()));
        }

        if !Self::is_valid_alignment(alignment) {
            return Err(sher_common::Error::Memory("Invalid alignment".to_string()));
        }

        self.allocations.insert(id.clone(), MemorySafetyAudit {
            allocation_id: id,
            size,
            alignment,
            freed: false,
            use_count: 0,
        });

        Ok(())
    }

    pub fn record_use(&mut self, id: &ObjectId) -> Result<()> {
        if self.freed_pointers.contains(id) {
            self.use_after_free_attempts += 1;
            return Err(sher_common::Error::Memory("Use-after-free detected".to_string()));
        }

        if let Some(audit) = self.allocations.get_mut(id) {
            if audit.freed {
                self.use_after_free_attempts += 1;
                return Err(sher_common::Error::Memory("Use-after-free detected".to_string()));
            }
            audit.use_count += 1;
            Ok(())
        } else {
            Err(sher_common::Error::Memory("Unknown allocation".to_string()))
        }
    }

    pub fn record_free(&mut self, id: &ObjectId) -> Result<()> {
        if self.freed_pointers.contains(id) {
            self.double_free_attempts += 1;
            return Err(sher_common::Error::Memory("Double-free detected".to_string()));
        }

        if let Some(audit) = self.allocations.get_mut(id) {
            if audit.freed {
                self.double_free_attempts += 1;
                return Err(sher_common::Error::Memory("Double-free detected".to_string()));
            }
            audit.freed = true;
            self.freed_pointers.insert(id.clone());
            Ok(())
        } else {
            Err(sher_common::Error::Memory("Unknown allocation".to_string()))
        }
    }

    pub fn check_bounds(&self, id: &ObjectId, offset: usize, size: usize) -> Result<()> {
        if let Some(audit) = self.allocations.get(id) {
            if offset + size > audit.size {
                return Err(sher_common::Error::Memory("Buffer overflow detected".to_string()));
            }
            Ok(())
        } else {
            Err(sher_common::Error::Memory("Unknown allocation".to_string()))
        }
    }

    pub fn audit(&self) -> AuditResult {
        let mut issues = Vec::new();
        let mut failed_checks = 0;

        for (id, audit) in &self.allocations {
            if audit.freed && audit.use_count > 0 {
                issues.push(format!("Potential use-after-free: {:?}", id));
                failed_checks += 1;
            }

            if !Self::is_valid_alignment(audit.alignment) {
                issues.push(format!("Invalid alignment for {:?}", id));
                failed_checks += 1;
            }

            if audit.size == 0 {
                issues.push(format!("Zero-size allocation: {:?}", id));
                failed_checks += 1;
            }
        }

        let total_checks = self.allocations.len() * 3 + self.use_after_free_attempts + self.double_free_attempts;

        AuditResult {
            passed: failed_checks == 0,
            total_checks,
            failed_checks,
            issues,
        }
    }

    pub fn get_leaked_allocations(&self) -> Vec<(ObjectId, usize)> {
        self.allocations.iter()
            .filter(|(_, audit)| !audit.freed)
            .map(|(id, audit)| (id.clone(), audit.size))
            .collect()
    }

    pub fn get_use_after_free_count(&self) -> usize {
        self.use_after_free_attempts
    }

    pub fn get_double_free_count(&self) -> usize {
        self.double_free_attempts
    }

    fn is_valid_alignment(alignment: usize) -> bool {
        alignment > 0 && alignment.is_power_of_two()
    }

    pub fn reset(&mut self) {
        self.allocations.clear();
        self.freed_pointers.clear();
        self.use_after_free_attempts = 0;
        self.double_free_attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation_registration() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let result = validator.register_allocation(id.clone(), 256, 8);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_size_allocation() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let result = validator.register_allocation(id, 0, 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_use_after_free_detection() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let _ = validator.register_allocation(id.clone(), 256, 8);
        let _ = validator.record_free(&id);
        let result = validator.record_use(&id);

        assert!(result.is_err());
        assert_eq!(validator.get_use_after_free_count(), 1);
    }

    #[test]
    fn test_double_free_detection() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let _ = validator.register_allocation(id.clone(), 256, 8);
        let _ = validator.record_free(&id);
        let result = validator.record_free(&id);

        assert!(result.is_err());
        assert_eq!(validator.get_double_free_count(), 1);
    }

    #[test]
    fn test_bounds_checking() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let _ = validator.register_allocation(id.clone(), 256, 8);
        let result = validator.check_bounds(&id, 200, 100);

        assert!(result.is_err());
    }

    #[test]
    fn test_audit() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let _ = validator.register_allocation(id, 256, 8);
        let audit = validator.audit();

        assert!(audit.passed);
    }

    #[test]
    fn test_leaked_allocations() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let _ = validator.register_allocation(id, 256, 8);
        let leaked = validator.get_leaked_allocations();

        assert_eq!(leaked.len(), 1);
    }

    #[test]
    fn test_invalid_alignment() {
        let mut validator = MemorySafetyValidator::new();
        let id = ObjectId::new();

        let result = validator.register_allocation(id, 256, 7);
        assert!(result.is_err());
    }
}
