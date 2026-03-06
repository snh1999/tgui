mod log_buffer;
mod managed_process;
mod manager;
mod shell;
mod streaming;

use crate::process::models::{LogLineEvent, SpawnContext};
use crate::utils::get_utc_timestamp_string;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub const WAIT_TIMEOUT: Duration = Duration::from_secs(8);

#[cfg(unix)]
pub fn spawn_context(command_id: i64, executable: &str, arguments: Vec<&str>) -> SpawnContext {
    SpawnContext {
        command_id,
        name: executable.to_string(),
        executable: executable.to_string(),
        arguments: arguments.into_iter().map(String::from).collect(),
        working_directory: PathBuf::from("/"),
        env_vars: vec![],
        shell: None,
    }
}

#[cfg(windows)]
pub fn spawn_context(command_id: i64, executable: &str, arguments: Vec<&str>) -> SpawnContext {
    SpawnContext {
        command_id,
        name: executable.to_string(),
        executable: "cmd".to_string(),
        arguments: arguments.into_iter().map(String::from).collect(),
        working_directory: std::env::temp_dir(),
        env_vars: vec![],
        shell: None,
    }
}

fn create_test_line(content: &str, is_stderr: bool, execution_id: i64) -> Arc<LogLineEvent> {
    Arc::new(LogLineEvent {
        execution_id,
        timestamp: get_utc_timestamp_string(),
        content: content.to_string(),
        is_stderr,
    })
}
