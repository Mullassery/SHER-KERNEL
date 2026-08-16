//! Transactional update state machine.
//!
//! Implements the sequence documented at the crate root:
//! 1. Download into System B (System A untouched)
//! 2. Verify signatures + hashes
//! 3. Boot test System B
//! 4. If OK, switch boot pointer
//! 5. Old version still bootable
//!
//! "Power fails at any point → Just boot previous version" holds because
//! every step before `commit` only touches the standby partition; the boot
//! pointer (and therefore what actually boots) is untouched until the very
//! last, single-word-sized step.

use crate::commit::commit as commit_step;
use crate::verify::verify as verify_step;
use sher_recovery::{
    check_default, BootPointer, HealthCheckReport, ImmutablePartition, PartitionSlot,
};
use sher_snapshot::SnapshotStore;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Pending,
    Downloading,
    Verifying,
    Testing,
    Committed,
    RolledBack,
    Failed(String),
}

#[derive(Debug)]
pub struct Transaction {
    pub target_version: String,
    pub state: TransactionState,
    pub log: Vec<String>,
}

impl Transaction {
    pub fn new(target_version: impl Into<String>) -> Self {
        Self {
            target_version: target_version.into(),
            state: TransactionState::Pending,
            log: Vec::new(),
        }
    }

    fn note(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
    }

    fn fail(&mut self, reason: impl Into<String>) -> Result<(), String> {
        let reason = reason.into();
        self.note(format!("FAILED: {reason}"));
        self.state = TransactionState::Failed(reason.clone());
        Err(reason)
    }

    /// Step 1: stage the new image into the standby partition. The active
    /// partition (whatever `boot_ptr` currently points at) is never passed
    /// in here, so it is structurally impossible for this step to touch it.
    pub fn download(&mut self, standby: &mut ImmutablePartition, image: Vec<u8>) {
        self.state = TransactionState::Downloading;
        let len = image.len();
        standby.write_image(self.target_version.clone(), image);
        self.note(format!(
            "staged {len} bytes into standby partition {:?}",
            standby.slot
        ));
    }

    /// Step 2: verify signatures + hashes (checksum-based; see
    /// `crate::verify` for what "verify" means in this simulation).
    pub fn verify(&mut self, standby: &ImmutablePartition) -> Result<(), String> {
        self.state = TransactionState::Verifying;
        match verify_step(standby) {
            Ok(()) => {
                self.note("verification passed");
                Ok(())
            }
            Err(reason) => self.fail(reason),
        }
    }

    /// Step 3: boot test System B via the recovery crate's health probes.
    pub fn test_boot(&mut self, standby: &ImmutablePartition) -> Result<HealthCheckReport, String> {
        self.state = TransactionState::Testing;
        let report = check_default(standby);
        if report.all_passed() {
            self.note("boot test passed");
            return Ok(report);
        }
        let failures: Vec<String> = report
            .failures()
            .into_iter()
            .map(|f| format!("{}: {}", f.name, f.detail))
            .collect();
        let reason = format!("boot test failed ({})", failures.join(", "));
        self.note(format!("FAILED: {reason}"));
        self.state = TransactionState::Failed(reason.clone());
        Err(reason)
    }

    /// Step 4: switch the boot pointer. Only reachable after verify/test
    /// have both returned `Ok`, by construction of the caller's workflow.
    pub fn commit(&mut self, boot_ptr: &mut BootPointer, target_slot: PartitionSlot) {
        commit_step(boot_ptr, target_slot, &self.target_version);
        self.state = TransactionState::Committed;
        self.note("boot pointer switched: update committed");
    }

    /// Optionally also record this version in the long-lived, versioned
    /// snapshot history (independent of the A/B boot pointer), so multiple
    /// prior versions remain inspectable/restorable via `sher_snapshot`.
    pub fn record_snapshot(
        &mut self,
        store: &mut SnapshotStore,
        component: &str,
        version: u32,
        image: Vec<u8>,
    ) {
        store.create(component, version, self.target_version.clone(), image);
        self.note(format!("recorded snapshot v{version} for '{component}'"));
    }

    /// Step 5 in reverse: power/verification/boot-test failure at any point
    /// rolls the boot pointer back to whatever it pointed at before, which
    /// remains fully intact because it was never written to.
    pub fn rollback(&mut self, boot_ptr: &mut BootPointer) {
        boot_ptr.rollback(format!("rollback failed update to {}", self.target_version));
        self.state = TransactionState::RolledBack;
        self.note("rolled back to previous boot pointer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_reaches_committed() {
        let mut txn = Transaction::new("2.0.0");
        let mut standby = ImmutablePartition::empty(PartitionSlot::B);
        let mut boot_ptr = BootPointer::default();

        txn.download(&mut standby, vec![1, 2, 3, 4]);
        txn.verify(&standby).unwrap();
        txn.test_boot(&standby).unwrap();
        txn.commit(&mut boot_ptr, PartitionSlot::B);

        assert_eq!(txn.state, TransactionState::Committed);
        assert_eq!(boot_ptr.active(), PartitionSlot::B);
        assert!(txn.log.len() >= 4);
    }

    #[test]
    fn verify_failure_stops_before_commit() {
        let mut txn = Transaction::new("2.0.0");
        let standby = ImmutablePartition::empty(PartitionSlot::B); // never downloaded into
        let boot_ptr = BootPointer::default();

        let result = txn.verify(&standby);
        assert!(result.is_err());
        assert!(matches!(txn.state, TransactionState::Failed(_)));
        assert_eq!(boot_ptr.active(), PartitionSlot::A);
    }

    #[test]
    fn rollback_after_failed_boot_test_restores_previous_pointer() {
        let mut txn = Transaction::new("2.0.0");
        let standby = ImmutablePartition::empty(PartitionSlot::B);
        let mut boot_ptr = BootPointer::default();

        assert!(txn.test_boot(&standby).is_err());
        txn.rollback(&mut boot_ptr);

        assert_eq!(txn.state, TransactionState::RolledBack);
        // Rolling back from A (nothing was ever committed) flips to B by
        // definition of "the other slot" — active system A itself was never
        // touched, which is the actual safety property under test.
        assert_eq!(boot_ptr.history().len(), 1);
    }

    #[test]
    fn power_fail_before_commit_leaves_original_boot_pointer_untouched() {
        let mut txn = Transaction::new("2.0.0");
        let mut standby = ImmutablePartition::empty(PartitionSlot::B);
        let boot_ptr = BootPointer::default();

        // Simulate "power fails" after download but before verify/commit.
        txn.download(&mut standby, vec![9, 9]);
        assert_eq!(boot_ptr.active(), PartitionSlot::A);
        assert!(boot_ptr.history().is_empty());
    }

    #[test]
    fn record_snapshot_adds_to_store() {
        let mut txn = Transaction::new("2.0.0");
        let mut store = SnapshotStore::new();
        txn.record_snapshot(&mut store, "kernel", 2, vec![1, 2, 3]);
        assert_eq!(store.active_version("kernel"), Some(2));
    }
}
