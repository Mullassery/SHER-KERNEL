//! Phase 13: Security Audit Framework
//!
//! Production hardening with:
//! - Input validation framework
//! - Memory safety checks
//! - Capability enforcement
//! - Audit logging
//! - Threat modeling

use sher_common::ObjectId;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug)]
pub struct SecurityEvent {
    pub event_id: ObjectId,
    pub timestamp: u64,
    pub event_type: String,
    pub source: String,
    pub threat_level: ThreatLevel,
    pub description: String,
    pub remediation: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InputValidator {
    rules: HashMap<String, ValidationRule>,
}

#[derive(Clone, Debug)]
pub struct ValidationRule {
    name: String,
    max_length: Option<usize>,
    min_length: Option<usize>,
    allowed_chars: Option<String>,
    reject_patterns: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MemorySafetyCheck {
    allocation_id: ObjectId,
    base_address: u64,
    size_bytes: usize,
    is_allocated: bool,
    guard_pages: u32,
}

#[derive(Clone, Debug)]
pub struct CapabilityValidator {
    capabilities: HashMap<ObjectId, Vec<String>>,
    expiration_times: HashMap<ObjectId, u64>,
}

pub struct SecurityAudit {
    events: Vec<SecurityEvent>,
    input_validator: InputValidator,
    memory_checks: Vec<MemorySafetyCheck>,
    capability_validator: CapabilityValidator,
    threat_score: u32,
}

impl InputValidator {
    pub fn new() -> Self {
        InputValidator {
            rules: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    pub fn validate(&self, rule_name: &str, input: &str) -> sher_common::Result<()> {
        if let Some(rule) = self.rules.get(rule_name) {
            if let Some(max_len) = rule.max_length {
                if input.len() > max_len {
                    return Err(sher_common::Error::Security(
                        "Input exceeds maximum length".to_string(),
                    ));
                }
            }

            if let Some(min_len) = rule.min_length {
                if input.len() < min_len {
                    return Err(sher_common::Error::Security(
                        "Input below minimum length".to_string(),
                    ));
                }
            }

            if let Some(ref allowed) = rule.allowed_chars {
                if !input.chars().all(|c| allowed.contains(c)) {
                    return Err(sher_common::Error::Security(
                        "Input contains forbidden characters".to_string(),
                    ));
                }
            }

            for pattern in &rule.reject_patterns {
                if input.contains(pattern) {
                    return Err(sher_common::Error::Security(
                        "Input matches rejection pattern".to_string(),
                    ));
                }
            }

            Ok(())
        } else {
            Err(sher_common::Error::Security(
                "Validation rule not found".to_string(),
            ))
        }
    }
}

impl Default for CapabilityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityValidator {
    pub fn new() -> Self {
        CapabilityValidator {
            capabilities: HashMap::new(),
            expiration_times: HashMap::new(),
        }
    }

    pub fn grant_capability(&mut self, subject: ObjectId, capability: String, expiration: u64) {
        self.capabilities
            .entry(subject)
            .or_default()
            .push(capability);

        self.expiration_times.insert(subject, expiration);
    }

    pub fn has_capability(&self, subject: &ObjectId, capability: &str, current_time: u64) -> bool {
        if let Some(expiration) = self.expiration_times.get(subject) {
            if current_time > *expiration {
                return false;
            }
        } else {
            return false;
        }

        self.capabilities
            .get(subject)
            .map(|caps| caps.contains(&capability.to_string()))
            .unwrap_or(false)
    }

    pub fn revoke_capability(&mut self, subject: &ObjectId, capability: &str) -> bool {
        if let Some(caps) = self.capabilities.get_mut(subject) {
            if let Some(pos) = caps.iter().position(|c| c == capability) {
                caps.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn expire_capabilities(&mut self, current_time: u64) -> u32 {
        let mut expired_count = 0;
        let expired_subjects: Vec<_> = self
            .expiration_times
            .iter()
            .filter(|(_, &exp_time)| current_time > exp_time)
            .map(|(subj, _)| *subj)
            .collect();

        for subject in expired_subjects {
            self.capabilities.remove(&subject);
            self.expiration_times.remove(&subject);
            expired_count += 1;
        }

        expired_count
    }
}

impl SecurityAudit {
    pub fn new() -> Self {
        SecurityAudit {
            events: Vec::new(),
            input_validator: InputValidator::new(),
            memory_checks: Vec::new(),
            capability_validator: CapabilityValidator::new(),
            threat_score: 0,
        }
    }

    pub fn log_event(&mut self, event: SecurityEvent) {
        if event.threat_level == ThreatLevel::Critical {
            self.threat_score += 50;
        } else if event.threat_level == ThreatLevel::High {
            self.threat_score += 20;
        } else if event.threat_level == ThreatLevel::Medium {
            self.threat_score += 5;
        }

        self.events.push(event);
    }

    pub fn register_memory_check(&mut self, check: MemorySafetyCheck) {
        self.memory_checks.push(check);
    }

    pub fn validate_memory_access(&self, address: u64) -> sher_common::Result<()> {
        for check in &self.memory_checks {
            if check.is_allocated {
                let end = check.base_address + check.size_bytes as u64;
                if address >= check.base_address && address < end {
                    return Ok(());
                }
            }
        }
        Err(sher_common::Error::Security(
            "Invalid memory access".to_string(),
        ))
    }

    /// Look up which registered allocation (by id) owns `address`, if any.
    /// Uses `MemorySafetyCheck::allocation_id`, closing the loop on that
    /// field actually being read somewhere.
    pub fn owning_allocation(&self, address: u64) -> Option<ObjectId> {
        self.memory_checks.iter().find_map(|check| {
            let end = check.base_address + check.size_bytes as u64;
            if check.is_allocated && address >= check.base_address && address < end {
                Some(check.allocation_id)
            } else {
                None
            }
        })
    }

    /// Count how many bytes of guard-page protection currently surround
    /// active allocations — a coarse indicator used in `security_status`.
    pub fn total_guard_pages(&self) -> u32 {
        self.memory_checks
            .iter()
            .filter(|c| c.is_allocated)
            .map(|c| c.guard_pages)
            .sum()
    }

    /// Validate a piece of input against a named rule registered on the
    /// embedded [`InputValidator`]. Delegates rather than duplicating logic.
    pub fn validate_input(&self, rule_name: &str, input: &str) -> sher_common::Result<()> {
        self.input_validator.validate(rule_name, input)
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.input_validator.add_rule(rule);
    }

    /// Grant a capability via the embedded [`CapabilityValidator`].
    pub fn grant_capability(&mut self, subject: ObjectId, capability: String, expiration: u64) {
        self.capability_validator
            .grant_capability(subject, capability, expiration);
    }

    /// Enforce a capability check via the embedded [`CapabilityValidator`],
    /// logging a security event on denial.
    pub fn enforce_capability(
        &mut self,
        subject: ObjectId,
        capability: &str,
        current_time: u64,
    ) -> sher_common::Result<()> {
        if self
            .capability_validator
            .has_capability(&subject, capability, current_time)
        {
            Ok(())
        } else {
            self.log_event(SecurityEvent {
                event_id: ObjectId::new(),
                timestamp: current_time,
                event_type: "capability_denied".to_string(),
                source: subject.to_string(),
                threat_level: ThreatLevel::Medium,
                description: format!("subject lacks capability '{capability}'"),
                remediation: None,
            });
            Err(sher_common::Error::Security(format!(
                "subject {subject} lacks capability '{capability}'"
            )))
        }
    }

    pub fn get_threat_score(&self) -> u32 {
        self.threat_score
    }

    pub fn get_events(&self) -> &[SecurityEvent] {
        &self.events
    }

    pub fn critical_event_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| e.threat_level == ThreatLevel::Critical)
            .count()
    }

    pub fn security_status(&self) -> String {
        if self.threat_score > 100 {
            "CRITICAL".to_string()
        } else if self.threat_score > 50 {
            "HIGH".to_string()
        } else if self.threat_score > 20 {
            "MEDIUM".to_string()
        } else {
            "SECURE".to_string()
        }
    }
}

impl Default for SecurityAudit {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_audit_creation() {
        let audit = SecurityAudit::new();
        assert_eq!(audit.get_threat_score(), 0);
        assert_eq!(audit.critical_event_count(), 0);
    }

    #[test]
    fn test_input_validator_length_check() {
        let mut validator = InputValidator::new();
        let rule = ValidationRule {
            name: "test_rule".to_string(),
            max_length: Some(10),
            min_length: Some(1),
            allowed_chars: None,
            reject_patterns: vec![],
        };

        validator.add_rule(rule);

        assert!(validator.validate("test_rule", "valid").is_ok());
        assert!(validator.validate("test_rule", "").is_err());
        assert!(validator.validate("test_rule", "this_is_too_long").is_err());
    }

    #[test]
    fn test_input_validator_character_check() {
        let mut validator = InputValidator::new();
        let rule = ValidationRule {
            name: "alphanum".to_string(),
            max_length: None,
            min_length: None,
            allowed_chars: Some("abcdefghijklmnopqrstuvwxyz0123456789".to_string()),
            reject_patterns: vec![],
        };

        validator.add_rule(rule);

        assert!(validator.validate("alphanum", "valid123").is_ok());
        assert!(validator.validate("alphanum", "invalid!").is_err());
    }

    #[test]
    fn test_input_validator_pattern_rejection() {
        let mut validator = InputValidator::new();
        let rule = ValidationRule {
            name: "no_sql".to_string(),
            max_length: None,
            min_length: None,
            allowed_chars: None,
            reject_patterns: vec!["DROP".to_string(), "DELETE".to_string()],
        };

        validator.add_rule(rule);

        assert!(validator.validate("no_sql", "SELECT * FROM users").is_ok());
        assert!(validator.validate("no_sql", "DROP TABLE users").is_err());
    }

    #[test]
    fn test_capability_grant_and_check() {
        let mut cv = CapabilityValidator::new();
        let subject = ObjectId::new();
        let cap = "read_file".to_string();

        cv.grant_capability(subject.clone(), cap.clone(), 1000);

        assert!(cv.has_capability(&subject, "read_file", 500));
        assert!(!cv.has_capability(&subject, "read_file", 1001));
    }

    #[test]
    fn test_capability_revocation() {
        let mut cv = CapabilityValidator::new();
        let subject = ObjectId::new();

        cv.grant_capability(subject.clone(), "read_file".to_string(), 1000);
        assert!(cv.has_capability(&subject, "read_file", 500));

        let revoked = cv.revoke_capability(&subject, "read_file");
        assert!(revoked);
        assert!(!cv.has_capability(&subject, "read_file", 500));
    }

    #[test]
    fn test_capability_expiration() {
        let mut cv = CapabilityValidator::new();
        let subj1 = ObjectId::new();
        let subj2 = ObjectId::new();

        cv.grant_capability(subj1.clone(), "cap1".to_string(), 500);
        cv.grant_capability(subj2.clone(), "cap2".to_string(), 1500);

        let expired = cv.expire_capabilities(1000);
        assert_eq!(expired, 1);

        assert!(!cv.has_capability(&subj1, "cap1", 1000));
        assert!(cv.has_capability(&subj2, "cap2", 1200));
    }

    #[test]
    fn test_security_event_logging() {
        let mut audit = SecurityAudit::new();
        let event = SecurityEvent {
            event_id: ObjectId::new(),
            timestamp: 12345,
            event_type: "unauthorized_access".to_string(),
            source: "client_1".to_string(),
            threat_level: ThreatLevel::High,
            description: "Attempted access to restricted resource".to_string(),
            remediation: Some("Revoke access token".to_string()),
        };

        audit.log_event(event);
        assert_eq!(audit.get_threat_score(), 20);
        assert_eq!(audit.get_events().len(), 1);
    }

    #[test]
    fn test_critical_threat_scoring() {
        let mut audit = SecurityAudit::new();

        for i in 0..3 {
            let event = SecurityEvent {
                event_id: ObjectId::new(),
                timestamp: 12345 + i,
                event_type: "critical_violation".to_string(),
                source: format!("source_{}", i),
                threat_level: ThreatLevel::Critical,
                description: "Critical security violation".to_string(),
                remediation: Some("Immediate shutdown".to_string()),
            };
            audit.log_event(event);
        }

        assert_eq!(audit.get_threat_score(), 150);
        assert_eq!(audit.security_status(), "CRITICAL");
    }

    #[test]
    fn test_memory_safety_check() {
        let mut audit = SecurityAudit::new();
        let check = MemorySafetyCheck {
            allocation_id: ObjectId::new(),
            base_address: 0x1000,
            size_bytes: 4096,
            is_allocated: true,
            guard_pages: 1,
        };

        audit.register_memory_check(check);

        assert!(audit.validate_memory_access(0x1500).is_ok());
        assert!(audit.validate_memory_access(0x5000).is_err());
    }

    #[test]
    fn test_threat_level_assessment() {
        let events = vec![
            (ThreatLevel::Low, 0),
            (ThreatLevel::Medium, 5),
            (ThreatLevel::High, 20),
            (ThreatLevel::Critical, 50),
        ];

        for (level, expected_score) in events {
            let mut audit = SecurityAudit::new();
            let event = SecurityEvent {
                event_id: ObjectId::new(),
                timestamp: 12345,
                event_type: "test".to_string(),
                source: "test".to_string(),
                threat_level: level,
                description: "test".to_string(),
                remediation: None,
            };

            audit.log_event(event);
            assert_eq!(audit.get_threat_score(), expected_score);
        }
    }

    #[test]
    fn test_comprehensive_security_scenario() {
        let mut audit = SecurityAudit::new();
        let mut validator = InputValidator::new();

        let rule = ValidationRule {
            name: "api_input".to_string(),
            max_length: Some(256),
            min_length: Some(1),
            allowed_chars: None,
            reject_patterns: vec!["../".to_string(), "NULL".to_string()],
        };

        validator.add_rule(rule);

        assert!(validator.validate("api_input", "safe_input").is_ok());

        let event = SecurityEvent {
            event_id: ObjectId::new(),
            timestamp: 12345,
            event_type: "input_validation_passed".to_string(),
            source: "api_endpoint".to_string(),
            threat_level: ThreatLevel::Low,
            description: "Malicious input detected and rejected".to_string(),
            remediation: None,
        };

        audit.log_event(event);
        assert_eq!(audit.security_status(), "SECURE");
    }

    #[test]
    fn test_audit_trail_completeness() {
        let mut audit = SecurityAudit::new();

        for i in 0..5 {
            let event = SecurityEvent {
                event_id: ObjectId::new(),
                timestamp: 12345 + i,
                event_type: format!("event_{}", i),
                source: format!("source_{}", i),
                threat_level: ThreatLevel::Low,
                description: format!("Event {}", i),
                remediation: None,
            };

            audit.log_event(event);
        }

        assert_eq!(audit.get_events().len(), 5);
        assert!(audit.get_events()[0].timestamp < audit.get_events()[4].timestamp);
    }

    #[test]
    fn test_audit_delegates_input_validation() {
        let mut audit = SecurityAudit::new();
        audit.add_validation_rule(ValidationRule {
            name: "username".to_string(),
            max_length: Some(10),
            min_length: None,
            allowed_chars: None,
            reject_patterns: vec![],
        });

        assert!(audit.validate_input("username", "short").is_ok());
        assert!(audit
            .validate_input("username", "way_too_long_name")
            .is_err());
    }

    #[test]
    fn test_audit_capability_enforcement_logs_denial() {
        let mut audit = SecurityAudit::new();
        let subject = ObjectId::new();

        assert!(audit.enforce_capability(subject, "admin", 100).is_err());
        assert_eq!(audit.get_events().len(), 1);
        assert_eq!(audit.get_events()[0].event_type, "capability_denied");

        audit.grant_capability(subject, "admin".to_string(), 1000);
        assert!(audit.enforce_capability(subject, "admin", 100).is_ok());
    }

    #[test]
    fn test_owning_allocation_and_guard_pages() {
        let mut audit = SecurityAudit::new();
        let alloc_id = ObjectId::new();
        audit.register_memory_check(MemorySafetyCheck {
            allocation_id: alloc_id,
            base_address: 0x1000,
            size_bytes: 256,
            is_allocated: true,
            guard_pages: 2,
        });

        assert_eq!(audit.owning_allocation(0x1050), Some(alloc_id));
        assert_eq!(audit.owning_allocation(0x9999), None);
        assert_eq!(audit.total_guard_pages(), 2);
    }
}
