use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Out of memory")]
    OutOfMemory,

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

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_include_context() {
        let err = Error::Memory("bad address".to_string());
        assert_eq!(err.to_string(), "Memory error: bad address");
    }

    #[test]
    fn io_error_converts_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn ipc_error_includes_context() {
        let err = Error::Ipc("mailbox full: frame".to_string());
        assert_eq!(err.to_string(), "IPC error: mailbox full: frame");
    }

    #[test]
    fn out_of_memory_has_fixed_message() {
        assert_eq!(Error::OutOfMemory.to_string(), "Out of memory");
    }
}
