//! Inter-process communication: named mailboxes with FIFO delivery. Real
//! in-process message passing (not a syscall-level IPC mechanism — this
//! process has no other processes to talk to, so it models the primitive a
//! real kernel would expose above raw syscalls).

use sher_common::{Error, Result};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub from: String,
    pub payload: Vec<u8>,
}

#[derive(Default)]
pub struct IpcBus {
    mailboxes: HashMap<String, VecDeque<Message>>,
}

impl IpcBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_mailbox(&mut self, name: impl Into<String>) {
        self.mailboxes.entry(name.into()).or_default();
    }

    pub fn send(&mut self, to: &str, from: impl Into<String>, payload: Vec<u8>) -> Result<()> {
        let mailbox = self
            .mailboxes
            .get_mut(to)
            .ok_or_else(|| Error::Unknown(format!("no such mailbox: {to}")))?;
        mailbox.push_back(Message {
            from: from.into(),
            payload,
        });
        Ok(())
    }

    /// Receive the oldest pending message for `mailbox`, if any.
    pub fn receive(&mut self, mailbox: &str) -> Option<Message> {
        self.mailboxes.get_mut(mailbox)?.pop_front()
    }

    pub fn pending_count(&self, mailbox: &str) -> usize {
        self.mailboxes.get(mailbox).map(VecDeque::len).unwrap_or(0)
    }
}

pub fn initialize() -> Result<IpcBus> {
    Ok(IpcBus::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_to_unregistered_mailbox_errors() {
        let mut bus = IpcBus::new();
        assert!(bus.send("driver-a", "kernel", vec![1]).is_err());
    }

    #[test]
    fn send_and_receive_is_fifo() {
        let mut bus = IpcBus::new();
        bus.register_mailbox("driver-a");
        bus.send("driver-a", "kernel", vec![1]).unwrap();
        bus.send("driver-a", "kernel", vec![2]).unwrap();

        let first = bus.receive("driver-a").unwrap();
        assert_eq!(first.payload, vec![1]);
        let second = bus.receive("driver-a").unwrap();
        assert_eq!(second.payload, vec![2]);
        assert!(bus.receive("driver-a").is_none());
    }

    #[test]
    fn pending_count_tracks_queue_depth() {
        let mut bus = IpcBus::new();
        bus.register_mailbox("x");
        bus.send("x", "y", vec![]).unwrap();
        assert_eq!(bus.pending_count("x"), 1);
        bus.receive("x");
        assert_eq!(bus.pending_count("x"), 0);
    }
}
