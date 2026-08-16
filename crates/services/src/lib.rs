//! SHER Optional Services
//! Server boot: never loads display, audio
//! Workstation: loads filesystem, networking, display
//! Headless: minimal services only
//!
//! [`manager::ServiceManager`] implements the real lazy-loading policy
//! described above. The per-kind modules below (`filesystem`, `networking`,
//! `storage`, `display`, `audio`) are marker types identifying each service;
//! this crate does not implement a real filesystem, network stack, display
//! server, or audio pipeline — those live in their own subsystem crates.

pub mod audio;
pub mod display;
pub mod filesystem;
pub mod manager;
pub mod networking;
pub mod storage;

pub use manager::{Profile, ServiceKind, ServiceManager, ServiceState};
