//! Boot pointer: tracks which partition slot boots next and which one is
//! currently active, with an audit trail of every switch. Switching the
//! pointer is the entire "rollback" operation in the A/B scheme — no data
//! is copied, so it is effectively instant and always leaves the previous
//! system bootable.

use crate::partition::PartitionSlot;

#[derive(Debug, Clone)]
pub struct BootSwitch {
    pub from: PartitionSlot,
    pub to: PartitionSlot,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct BootPointer {
    active: PartitionSlot,
    history: Vec<BootSwitch>,
}

impl BootPointer {
    pub fn new(initial: PartitionSlot) -> Self {
        Self {
            active: initial,
            history: Vec::new(),
        }
    }

    pub fn active(&self) -> PartitionSlot {
        self.active
    }

    /// Switch the boot pointer to `target`, recording why. No-op (but still
    /// recorded) if `target` is already active.
    pub fn switch_to(&mut self, target: PartitionSlot, reason: impl Into<String>) {
        let from = self.active;
        self.history.push(BootSwitch {
            from,
            to: target,
            reason: reason.into(),
        });
        self.active = target;
    }

    /// Instant rollback: switch back to whatever the previous slot was.
    pub fn rollback(&mut self, reason: impl Into<String>) {
        let target = self.active.other();
        self.switch_to(target, reason);
    }

    pub fn history(&self) -> &[BootSwitch] {
        &self.history
    }
}

impl Default for BootPointer {
    fn default() -> Self {
        Self::new(PartitionSlot::A)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_slot_a() {
        let ptr = BootPointer::default();
        assert_eq!(ptr.active(), PartitionSlot::A);
        assert!(ptr.history().is_empty());
    }

    #[test]
    fn switch_to_updates_active_and_records_history() {
        let mut ptr = BootPointer::default();
        ptr.switch_to(PartitionSlot::B, "update verified");
        assert_eq!(ptr.active(), PartitionSlot::B);
        assert_eq!(ptr.history().len(), 1);
        assert_eq!(ptr.history()[0].from, PartitionSlot::A);
        assert_eq!(ptr.history()[0].to, PartitionSlot::B);
    }

    #[test]
    fn rollback_flips_to_other_slot() {
        let mut ptr = BootPointer::default();
        ptr.switch_to(PartitionSlot::B, "update");
        ptr.rollback("boot test failed");
        assert_eq!(ptr.active(), PartitionSlot::A);
        assert_eq!(ptr.history().len(), 2);
        assert_eq!(ptr.history()[1].reason, "boot test failed");
    }
}
