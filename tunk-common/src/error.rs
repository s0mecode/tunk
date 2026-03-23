use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BackendError {
    #[error("IoError: {0}")]
    IoError(String),
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Process error: {0}")]
    ProcessError(#[from] ProcessError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ProcessError {
    #[error("Process wait failed: {0}")]
    ProcessWaitFailed(String),
    #[error("Process kill failed: {0}")]
    ProcessKillFailed(String),
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(String),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DaemonError {
    #[error("Tunnel not found: {0}")]
    TunnelNotFound(String),

    #[error("Tunnel already running: {0}")]
    TunnelAlreadyRunning(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Task panicked: {0}")]
    TaskPanic(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for DaemonError {
    fn from(e: std::io::Error) -> Self {
        DaemonError::Io(e.to_string())
    }
}
