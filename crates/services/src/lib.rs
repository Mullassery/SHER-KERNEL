//! SHER Optional Services
//! Server boot: never loads display, audio
//! Workstation: loads filesystem, networking, display
//! Headless: minimal services only

pub mod filesystem;
pub mod networking;
pub mod storage;
pub mod display;
pub mod audio;
