//! Inter-process communication: named mailboxes with FIFO delivery. Real
//! in-process message passing (not a syscall-level IPC mechanism — this
//! process has no other processes to talk to, so it models the primitive a
//! real kernel would expose above raw syscalls).
//!
//! Each mailbox is a **lock-free, bounded ring buffer**
//! ([`crossbeam_queue::ArrayQueue`], a well-established lock-free bounded
//! queue), not a `VecDeque` behind `&mut self`: `send`/`receive` take `&self`
//! and can be called concurrently from multiple threads without a mutex.
//! Payloads are [`Arc<[u8]>`] rather than an owned `Vec<u8>`, so passing a
//! framebuffer or input-event buffer from producer to consumer is an O(1)
//! refcount clone, not a byte-for-byte copy — the zero-copy half of
//! "formalize IPC ... to pass framebuffers/input events safely without
//! kernel-space memory overhead". Bounded capacity (rather than the
//! previous unbounded `VecDeque`) is what makes this an actual *ring*
//! buffer and caps per-mailbox memory overhead; a full mailbox reports
//! [`Error::Ipc`] instead of growing without limit.

use crossbeam_queue::ArrayQueue;
use sher_common::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Capacity used by [`IpcBus::register_mailbox`] when no explicit capacity
/// is given. Framebuffer/high-throughput mailboxes should call
/// [`IpcBus::register_mailbox_with_capacity`] instead with a size matched to
/// their producer/consumer cadence.
pub const DEFAULT_MAILBOX_CAPACITY: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub from: String,
    pub payload: Arc<[u8]>,
}

#[derive(Default)]
pub struct IpcBus {
    mailboxes: HashMap<String, Arc<ArrayQueue<Message>>>,
}

impl IpcBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a mailbox with [`DEFAULT_MAILBOX_CAPACITY`].
    pub fn register_mailbox(&mut self, name: impl Into<String>) {
        self.register_mailbox_with_capacity(name, DEFAULT_MAILBOX_CAPACITY);
    }

    /// Register a mailbox with an explicit ring-buffer capacity — e.g. a
    /// framebuffer channel that only ever needs to hold the latest one or
    /// two frames, or a high-volume input-event channel that wants more
    /// headroom than the default.
    pub fn register_mailbox_with_capacity(&mut self, name: impl Into<String>, capacity: usize) {
        self.mailboxes
            .entry(name.into())
            .or_insert_with(|| Arc::new(ArrayQueue::new(capacity.max(1))));
    }

    /// Enqueue a message without blocking. Lock-free: safe to call
    /// concurrently from multiple producer threads sharing the same
    /// `IpcBus` (or a clone of one mailbox's handle via
    /// [`IpcBus::mailbox_handle`]).
    pub fn send(
        &self,
        to: &str,
        from: impl Into<String>,
        payload: impl Into<Arc<[u8]>>,
    ) -> Result<()> {
        let mailbox = self
            .mailboxes
            .get(to)
            .ok_or_else(|| Error::Ipc(format!("no such mailbox: {to}")))?;
        mailbox
            .push(Message {
                from: from.into(),
                payload: payload.into(),
            })
            .map_err(|_| Error::Ipc(format!("mailbox full: {to}")))
    }

    /// Receive the oldest pending message for `mailbox`, if any. Lock-free:
    /// safe to call concurrently with `send` and with other `receive`
    /// calls on the same mailbox.
    pub fn receive(&self, mailbox: &str) -> Option<Message> {
        self.mailboxes.get(mailbox)?.pop()
    }

    pub fn pending_count(&self, mailbox: &str) -> usize {
        self.mailboxes.get(mailbox).map(|q| q.len()).unwrap_or(0)
    }

    pub fn capacity(&self, mailbox: &str) -> Option<usize> {
        self.mailboxes.get(mailbox).map(|q| q.capacity())
    }

    /// A cheaply-cloneable handle to one mailbox's underlying ring buffer,
    /// for a producer/consumer pair that wants to bypass the by-name lookup
    /// (and doesn't need the rest of `IpcBus`) on a hot path — e.g. a
    /// dedicated framebuffer-delivery thread.
    pub fn mailbox_handle(&self, mailbox: &str) -> Option<Arc<ArrayQueue<Message>>> {
        self.mailboxes.get(mailbox).cloned()
    }
}

pub fn initialize() -> Result<IpcBus> {
    Ok(IpcBus::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::thread;

    #[test]
    fn send_to_unregistered_mailbox_errors() {
        let bus = IpcBus::new();
        assert!(bus.send("driver-a", "kernel", vec![1]).is_err());
    }

    #[test]
    fn send_and_receive_is_fifo() {
        let mut bus = IpcBus::new();
        bus.register_mailbox("driver-a");
        bus.send("driver-a", "kernel", vec![1]).unwrap();
        bus.send("driver-a", "kernel", vec![2]).unwrap();

        let first = bus.receive("driver-a").unwrap();
        assert_eq!(&*first.payload, &[1][..]);
        let second = bus.receive("driver-a").unwrap();
        assert_eq!(&*second.payload, &[2][..]);
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

    #[test]
    fn mailbox_is_a_bounded_ring_buffer_not_unbounded() {
        let mut bus = IpcBus::new();
        bus.register_mailbox_with_capacity("small", 2);
        bus.send("small", "kernel", vec![1]).unwrap();
        bus.send("small", "kernel", vec![2]).unwrap();

        let err = bus.send("small", "kernel", vec![3]).unwrap_err();
        assert!(err.to_string().contains("mailbox full"));
        assert_eq!(bus.pending_count("small"), 2);
    }

    #[test]
    fn send_and_receive_are_zero_copy() {
        // `payload` is the exact same allocation on both sides of the
        // channel -- proven by pointer identity, not just equal contents --
        // which is what makes this zero-copy rather than "copies a Vec but
        // the bytes happen to match".
        let mut bus = IpcBus::new();
        bus.register_mailbox("frame");

        let framebuffer: Arc<[u8]> = Arc::from(vec![0u8; 4096]);
        let sent_ptr = Arc::as_ptr(&framebuffer);
        bus.send("frame", "gpu", framebuffer).unwrap();

        let received = bus.receive("frame").unwrap();
        assert_eq!(Arc::as_ptr(&received.payload), sent_ptr);
    }

    #[test]
    fn send_and_receive_work_through_shared_ref_across_threads() {
        // `send`/`receive` take `&self`: a single `IpcBus` (behind an Arc,
        // as it would be shared between real producer/consumer threads) can
        // serve concurrent producers and a concurrent consumer without any
        // external Mutex -- the "lock-free" half of the requirement.
        let mut bus = IpcBus::new();
        bus.register_mailbox_with_capacity("events", 1024);
        let bus = Arc::new(bus);

        const PRODUCERS: usize = 8;
        const PER_PRODUCER: usize = 100;
        let barrier = Arc::new(Barrier::new(PRODUCERS));

        let handles: Vec<_> = (0..PRODUCERS)
            .map(|p| {
                let bus = Arc::clone(&bus);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    for i in 0..PER_PRODUCER {
                        let payload: Arc<[u8]> = Arc::from(vec![p as u8, i as u8]);
                        loop {
                            if bus
                                .send("events", format!("producer-{p}"), payload.clone())
                                .is_ok()
                            {
                                break;
                            }
                            thread::yield_now();
                        }
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let mut received = 0;
        while bus.receive("events").is_some() {
            received += 1;
        }
        assert_eq!(received, PRODUCERS * PER_PRODUCER);
    }
}
