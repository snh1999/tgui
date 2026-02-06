use crate::database::{Command, Database};
use crate::handlers::serialize_errors::SerializableError;
use tauri::State;

#[tauri::command]
pub fn create_command(db: State<'_, Database>, cmd: Command) -> Result<i64, SerializableError> {
    db.create_command(&cmd).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_command(db: State<'_, Database>, id: i64) -> Result<Command, SerializableError> {
    db.get_command(id).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_commands(
    db: State<'_, Database>,
    group_id: Option<i64>,
    category_id: Option<i64>,
    favorites_only: bool,
) -> Result<Vec<Command>, SerializableError> {
    db.get_commands(group_id, category_id, favorites_only)
        .map_err(|e| e.into())
}

#[tauri::command]
pub fn update_command(db: State<'_, Database>, cmd: Command) -> Result<(), SerializableError> {
    db.update_command(&cmd).map_err(|e| e.into())
}

#[tauri::command]
pub fn move_command_between(
    db: State<'_, Database>,
    cmd_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), SerializableError> {
    db.move_command_between(cmd_id, prev_id, next_id)
        .map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_command(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.delete_command(id).map_err(|e| e.into())
}

#[tauri::command]
pub fn toggle_command_favorite(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.toggle_command_favorite(id).map_err(|e| e.into())
}

#[tauri::command]
pub fn search_commands(
    db: State<'_, Database>,
    term: String,
) -> Result<Vec<Command>, SerializableError> {
    db.search_commands(&term).map_err(|e| e.into())
}
