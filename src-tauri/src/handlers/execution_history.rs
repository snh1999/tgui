use crate::database::{Database, ExecutionHistory, ExecutionStatus};
use crate::handlers::serialize_errors::SerializableError;
use tauri::State;

#[tauri::command]
pub fn get_execution_history(
    db: State<'_, Database>,
    id: i64,
) -> Result<ExecutionHistory, SerializableError> {
    db.get_execution_history(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_command_execution_history(
    db: State<'_, Database>,
    command_id: i64,
    limit: Option<i64>,
) -> Result<Vec<ExecutionHistory>, SerializableError> {
    db.get_command_execution_history(command_id, limit)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_execution_history(
    db: State<'_, Database>,
    workflow_id: i64,
    limit: Option<i64>,
) -> Result<Vec<ExecutionHistory>, SerializableError> {
    db.get_workflow_execution_history(workflow_id, limit)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_running_commands(
    db: State<'_, Database>,
    command_id: Option<i64>,
    workflow_id: Option<i64>,
) -> Result<Vec<ExecutionHistory>, SerializableError> {
    db.get_running_commands(command_id, workflow_id)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn cleanup_command_history(
    db: State<'_, Database>,
    command_id: i64,
    keep_last: i64,
) -> Result<(), SerializableError> {
    db.cleanup_command_history(command_id, keep_last)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn cleanup_history_older_than(
    db: State<'_, Database>,
    days: i64,
) -> Result<(), SerializableError> {
    db.cleanup_history_older_than(days)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_command_execution_stats(
    db: State<'_, Database>,
    command_id: i64,
    status: Option<ExecutionStatus>,
) -> Result<i64, SerializableError> {
    db.get_command_execution_stats(command_id, status)
        .map_err(|err| err.into())
}
