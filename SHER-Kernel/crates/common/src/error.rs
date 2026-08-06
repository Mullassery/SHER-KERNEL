use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Driver error: {0}")]
    Driver(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Interrupt error: {0}")]
    Interrupt(String),

    #[error("Networking error: {0}")]
    Networking(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
