#[derive(Debug, thiserror::Error)]
pub enum ProcessKillError {
    #[error("Failed to send signal: {0}")]
    SignalFailed(String),

    #[error("Failed to wait for process: {0}")]
    WaitFailed(String),

    #[error("Process already exited")]
    AlreadyExited,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Platform-specific error: {0}")]
    PlatformError(String),

    #[error("Process not found: {0}")]
    NotFound(i64),

    #[error("Invalid data")]
    Invalid,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessSpawnError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[source] std::io::Error),
    #[error("Failed to get PID after spawn")]
    NoPid,
    #[error("Failed to persist data: {0}")]
    DatabaseError(String),
    #[error("Executable not found: {0}")]
    ExecutableNotFound(String),
    #[error("Working directory does not exist: {0}")]
    InvalidWorkingDirectory(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid shell: {0}")]
    InvalidShell(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessManagerError {
    #[error("Process not found: {0}")]
    ProcessNotFound(i64),
    #[error("Process is not running: {0}")]
    NotRunning(i64),
    #[error("Database error: {0}")]
    DatabaseError(String),
}
