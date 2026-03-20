use crate::database::{Database, TriggeredBy};
use crate::handlers::serialize_errors::SerializableError;
use crate::process::manager::ProcessManager;
use crate::process::models::{LogLineEvent, ProcessInfo, SpawnContext, TrayStatus};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

#[tauri::command]
pub async fn resolve_command_context(
    command_id: i64,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<SpawnContext, SerializableError> {
    pm.resolve_spawn_context(command_id)
        .await
        .map_err(|e| SerializableError::from(e))
}

#[tauri::command]
pub async fn spawn_command(
    command_id: i64,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<i64, SerializableError> {
    debug!(command_id, "Spawning command");
    let context = pm
        .resolve_spawn_context(command_id)
        .await
        .map_err(|e| SerializableError::from(e))?;
    let execution_id = pm
        .spawn_command(context, TriggeredBy::Manual)
        .await
        .map_err(|e| SerializableError::from(e.to_string()))?;
    Ok(execution_id)
}

#[tauri::command]
pub async fn kill_process(
    execution_id: i64,
    force: bool,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<(), SerializableError> {
    debug!(execution_id, force, "Killing process");
    pm.kill_process(execution_id, force)
        .await
        .map_err(|e| SerializableError::from(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn get_running_processes(
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<Vec<ProcessInfo>, SerializableError> {
    Ok(pm.get_running_processes().await)
}

#[tauri::command]
pub async fn get_process_status(
    execution_id: i64,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<ProcessInfo, SerializableError> {
    pm.get_process_info(execution_id)
        .await
        .ok_or_else(|| SerializableError::from("Process not found".to_string()))
}

#[tauri::command]
pub async fn get_log_buffer(
    execution_id: i64,
    offset: usize,
    limit: usize,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<Vec<Arc<LogLineEvent>>, SerializableError> {
    pm.get_logs(execution_id, offset, limit)
        .await
        .ok_or_else(|| SerializableError::from("Process not found".to_string()))
}

#[tauri::command]
pub async fn clear_log_buffer(
    execution_id: i64,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<(), SerializableError> {
    pm.clear_logs(execution_id)
        .await
        .map_err(|e| SerializableError::from(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn stop_all_processes(
    force: bool,
    pm: State<'_, Arc<ProcessManager>>,
) -> Result<usize, SerializableError> {
    let count = pm.stop_all(force).await;
    Ok(count)
}

// TODO: remove this and switch to execution history
#[tauri::command]
pub async fn get_tray_status(
    pm: State<'_, Arc<ProcessManager>>,
    db: State<'_, Database>,
) -> Result<TrayStatus, SerializableError> {
    let running_count = pm.running_count().await as i64;
    let total_commands = db.get_commands_count(None, None, false).unwrap_or(0);
    let error_count = 0i64;

    Ok(TrayStatus {
        running_count,
        error_count,
        total_commands,
    })
}

#[tauri::command]
pub fn get_valid_shells() -> Vec<String> {
    crate::process::shell::get_allowed_shells()
}
