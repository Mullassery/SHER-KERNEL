//! Step 4 of the update sequence: switch boot pointer. This is the only
//! irreversible-looking step, but because it is just a pointer flip over an
//! untouched previous partition, `sher_recovery::BootPointer::rollback` can
//! always undo it — "old version still bootable".

use sher_recovery::{BootPointer, PartitionSlot};

pub fn commit(boot_ptr: &mut BootPointer, target: PartitionSlot, version: &str) {
    boot_ptr.switch_to(target, format!("commit update to {version}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_switches_active_partition() {
        let mut boot_ptr = BootPointer::default();
        assert_eq!(boot_ptr.active(), PartitionSlot::A);
        commit(&mut boot_ptr, PartitionSlot::B, "2.0.0");
        assert_eq!(boot_ptr.active(), PartitionSlot::B);
        assert_eq!(boot_ptr.history().len(), 1);
    }
}
