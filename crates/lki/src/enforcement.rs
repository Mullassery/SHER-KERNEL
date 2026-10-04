// SHER LKI: Security Enforcement
// Enforces capability grants and security policies

use crate::security::{Capability, CapabilityManager, SecurityLevel, SecurityPolicy};
use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

// ============================================================================
// SECURITY CONTEXT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub context_id: ObjectId,
    pub driver_id: ObjectId,
    pub policy: SecurityPolicy,
    pub capability_manager: CapabilityManager,
    pub operation_count: u64,
    pub denied_operations: u64,
    pub approved_operations: u64,
}

impl SecurityContext {
    pub fn new(driver_id: ObjectId, policy: SecurityPolicy) -> Self {
        SecurityContext {
            context_id: ObjectId::new(),
            driver_id,
            policy,
            capability_manager: CapabilityManager::new(),
            operation_count: 0,
            denied_operations: 0,
            approved_operations: 0,
        }
    }

    /// Check if operation is allowed
    pub fn check_operation(
        &mut self,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<bool> {
        self.operation_count += 1;

        match self.policy.security_level {
            SecurityLevel::Unrestricted => {
                self.approved_operations += 1;
                Ok(true)
            }
            SecurityLevel::Permissive => {
                if self.capability_manager.has_capability(
                    self.driver_id,
                    capability,
                    current_time_ms,
                ) {
                    self.approved_operations += 1;
                    Ok(true)
                } else {
                    self.denied_operations += 1;
                    Err(Error::Driver("Capability not granted".to_string()))
                }
            }
            SecurityLevel::Balanced | SecurityLevel::Strict | SecurityLevel::Critical => {
                if self.capability_manager.has_capability(
                    self.driver_id,
                    capability,
                    current_time_ms,
                ) {
                    self.approved_operations += 1;
                    Ok(true)
                } else {
                    self.denied_operations += 1;
                    Err(Error::Driver("Capability not granted".to_string()))
                }
            }
        }
    }

    /// Check with reauthentication requirement
    pub fn check_with_reauthentication(
        &mut self,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<bool> {
        self.operation_count += 1;

        match self
            .capability_manager
            .check_with_reauth(self.driver_id, capability, current_time_ms)
        {
            Ok(needs_reauth) => {
                self.approved_operations += 1;
                Ok(needs_reauth)
            }
            Err(_) => {
                self.denied_operations += 1;
                Err(Error::Driver("Capability not granted".to_string()))
            }
        }
    }

    /// Get denial rate
    pub fn denial_rate(&self) -> f64 {
        if self.operation_count == 0 {
            0.0
        } else {
            (self.denied_operations as f64 / self.operation_count as f64) * 100.0
        }
    }
}

// ============================================================================
// ENFORCER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SecurityEnforcer {
    pub contexts: HashMap<ObjectId, SecurityContext>,
    pub total_checks: u64,
    pub total_denials: u64,
    pub critical_denials: u64,
}

impl SecurityEnforcer {
    pub fn new() -> Self {
        SecurityEnforcer::default()
    }

    /// Register driver with security context
    pub fn register_driver(
        &mut self,
        driver_id: ObjectId,
        policy: SecurityPolicy,
    ) -> Result<ObjectId> {
        let context = SecurityContext::new(driver_id, policy);
        let context_id = context.context_id;
        self.contexts.insert(context_id, context);
        Ok(context_id)
    }

    /// Unregister driver
    pub fn unregister_driver(&mut self, context_id: ObjectId) -> Result<()> {
        if self.contexts.remove(&context_id).is_some() {
            Ok(())
        } else {
            Err(Error::Driver("Context not found".to_string()))
        }
    }

    /// Enforce capability check
    pub fn enforce(
        &mut self,
        context_id: ObjectId,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<()> {
        self.total_checks += 1;

        if let Some(context) = self.contexts.get_mut(&context_id) {
            match context.check_operation(capability, current_time_ms) {
                Ok(_) => Ok(()),
                Err(e) => {
                    self.total_denials += 1;
                    if matches!(context.policy.security_level, SecurityLevel::Critical) {
                        self.critical_denials += 1;
                    }
                    Err(e)
                }
            }
        } else {
            self.total_denials += 1;
            Err(Error::Driver("Context not found".to_string()))
        }
    }

    /// Enforce with reauthentication
    pub fn enforce_with_reauth(
        &mut self,
        context_id: ObjectId,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<bool> {
        self.total_checks += 1;

        if let Some(context) = self.contexts.get_mut(&context_id) {
            match context.check_with_reauthentication(capability, current_time_ms) {
                Ok(needs_reauth) => Ok(needs_reauth),
                Err(e) => {
                    self.total_denials += 1;
                    if matches!(context.policy.security_level, SecurityLevel::Critical) {
                        self.critical_denials += 1;
                    }
                    Err(e)
                }
            }
        } else {
            self.total_denials += 1;
            Err(Error::Driver("Context not found".to_string()))
        }
    }

    /// Get driver context
    pub fn get_context(&self, context_id: ObjectId) -> Option<&SecurityContext> {
        self.contexts.get(&context_id)
    }

    /// Get mutable context (for policy updates)
    pub fn get_context_mut(&mut self, context_id: ObjectId) -> Option<&mut SecurityContext> {
        self.contexts.get_mut(&context_id)
    }

    /// Get enforcement statistics
    pub fn get_stats(&self) -> EnforcementStats {
        EnforcementStats {
            total_checks: self.total_checks,
            total_denials: self.total_denials,
            critical_denials: self.critical_denials,
            denial_rate: if self.total_checks == 0 {
                0.0
            } else {
                (self.total_denials as f64 / self.total_checks as f64) * 100.0
            },
            contexts: self.contexts.len() as u64,
        }
    }

    /// Find contexts with high denial rate
    pub fn find_suspicious_contexts(&self, threshold: f64) -> Vec<&SecurityContext> {
        self.contexts
            .values()
            .filter(|ctx| ctx.denial_rate() > threshold)
            .collect()
    }
}

// ============================================================================
// ENFORCEMENT STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementStats {
    pub total_checks: u64,
    pub total_denials: u64,
    pub critical_denials: u64,
    pub denial_rate: f64,
    pub contexts: u64,
}

// ============================================================================
// PERMISSION CHECKER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct PermissionChecker {
    pub checks_performed: u64,
    pub checks_passed: u64,
    pub checks_failed: u64,
    pub capability_cache: HashMap<(ObjectId, Capability), bool>,
}

impl PermissionChecker {
    pub fn new() -> Self {
        PermissionChecker::default()
    }

    /// Check permission with caching.
    ///
    /// Only **denied** outcomes are cached — a granted outcome is never
    /// cached and is always re-verified against the live, time-bounded
    /// capability grant on every call.
    ///
    /// This asymmetry is deliberate and security-relevant, not an
    /// optimization left on the table: this workspace's capability model
    /// is explicitly time-bounded with "no silent renewal" (see
    /// `CapabilityGrant`/`CapabilityManager` — every grant carries an
    /// `expires_at_ms` and `has_capability()` re-checks it against the
    /// caller-supplied `current_time_ms` on every call). The previous
    /// version of this function cached a granted (`true`) result
    /// unconditionally, keyed only on `(context_id, capability)` with no
    /// expiry or timestamp attached to the cache entry. That meant a
    /// capability granted once and cached would continue to read as
    /// "granted" from the cache forever — including long after the
    /// underlying `CapabilityGrant` expired or was explicitly revoked —
    /// because nothing invalidates the cache on a timer; only an explicit
    /// `clear_cache()`/`clear_all()` call (which nothing in this crate's
    /// enforcement path calls automatically) would remove the stale entry.
    /// That is a real authorization bypass of this project's own
    /// time-bounded-capability security model, reachable from this public
    /// API (`PermissionChecker` is re-exported from `sher_lki`). Caching
    /// only denials keeps the fast-path optimization for the safe
    /// direction (repeatedly denying a driver that has never been granted
    /// a capability is cheap and "fails secure" either way) while removing
    /// the unsafe direction (silently treating an expired/revoked grant as
    /// still valid).
    pub fn check(
        &mut self,
        enforcer: &mut SecurityEnforcer,
        context_id: ObjectId,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<()> {
        self.checks_performed += 1;

        // Only a cached *denial* can be trusted without re-checking time —
        // a cached grant cannot, since grants expire.
        if let Some(&cached) = self.capability_cache.get(&(context_id, capability)) {
            if !cached {
                self.checks_failed += 1;
                return Err(Error::Driver("Permission denied (cached)".to_string()));
            }
        }

        // Perform actual check (always, for a granted outcome — never
        // trust a stale cache entry for permission to proceed).
        match enforcer.enforce(context_id, capability, current_time_ms) {
            Ok(_) => {
                // Deliberately not cached as `true` — see doc comment above.
                self.capability_cache.remove(&(context_id, capability));
                self.checks_passed += 1;
                Ok(())
            }
            Err(e) => {
                self.capability_cache
                    .insert((context_id, capability), false);
                self.checks_failed += 1;
                Err(e)
            }
        }
    }

    /// Clear cache on privilege change
    pub fn clear_cache(&mut self, context_id: ObjectId) {
        self.capability_cache
            .retain(|(ctx_id, _), _| *ctx_id != context_id);
    }

    /// Clear all cache
    pub fn clear_all(&mut self) {
        self.capability_cache.clear();
    }

    /// Get permission statistics
    pub fn get_stats(&self) -> PermissionStats {
        PermissionStats {
            checks_performed: self.checks_performed,
            checks_passed: self.checks_passed,
            checks_failed: self.checks_failed,
            cache_size: self.capability_cache.len() as u64,
            success_rate: if self.checks_performed == 0 {
                100.0
            } else {
                (self.checks_passed as f64 / self.checks_performed as f64) * 100.0
            },
        }
    }
}

// ============================================================================
// PERMISSION STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStats {
    pub checks_performed: u64,
    pub checks_passed: u64,
    pub checks_failed: u64,
    pub cache_size: u64,
    pub success_rate: f64,
}
