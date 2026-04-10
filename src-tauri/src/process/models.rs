use crate::constants::MAX_LOG_LINES;
use crate::database::ExecutionStatus;
use serde::{Deserialize, Serialize};

/// Returned by `get_running_executions` on startup for orphan detection.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrphanedProcess {
    pub execution_id: i64,
    pub command_id: Option<i64>,
    pub pid: i64,
    /// Whether the OS process is still alive (checked at startup).
    pub still_running: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub execution_id: i64,
    pub pid: u32,
    pub command_id: i64,
    pub command_name: String,
    pub command: String,
    pub status: ProcessStatus,
    pub start_time: String,
    pub exit_code: Option<i32>,
    pub log_line_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum KillMode {
    Graceful,
    Force,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Idle,
    Running {
        pid: u32,
        start_time: String,
    },
    Stopping {
        since: String,
    },
    Stopped {
        exit_code: i32,
        completed_at: String,
    },
    Error {
        exit_code: Option<i32>,
        message: String,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStartedEvent {
    pub execution_id: i64,
    pub pid: u32,
    pub command_id: i64,
    pub command_name: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStatusChangedEvent {
    pub execution_id: i64,
    pub old_status: ProcessStatus,
    pub new_status: ProcessStatus,
    pub timestamp: String,
}

/// A single log line emitted to the frontend.
/// Event name: `process:log:{execution_id}`
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogLineEvent {
    pub execution_id: i64,
    pub timestamp: String,
    pub content: String,
    pub is_stderr: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStoppedEvent {
    pub execution_id: i64,
    pub pid: u32,
    pub exit_code: Option<i32>,
    pub status: ExecutionStatus,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnContext {
    pub command_id: i64,
    pub name: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: std::path::PathBuf,
    pub env_vars: Vec<(String, String)>,
    pub shell: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingConfig {
    /// Max lines per process in memory
    pub buffer_capacity: usize,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub max_line_length: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: MAX_LOG_LINES,
            batch_size: 50,
            batch_timeout_ms: 50,
            max_line_length: MAX_LOG_LINES,
        }
    }
}

impl StreamingConfig {
    pub fn immediate() -> Self {
        Self {
            buffer_capacity: MAX_LOG_LINES,
            batch_size: 1,
            batch_timeout_ms: 0,
            max_line_length: 10000,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStatus {
    pub running_count: i64,
    pub error_count: i64,
    pub total_commands: i64,
}
