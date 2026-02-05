use crate::database::{Command, Database};
use tauri::State;

// TODO: use serializable error type and convert to e.into()
#[tauri::command]
pub fn create_command(db: State<'_, Database>, cmd: Command) -> Result<i64, String> {
    db.create_command(&cmd).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_command(db: State<'_, Database>, id: i64) -> Result<Command, String> {
    db.get_command(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_commands(
    db: State<'_, Database>,
    group_id: Option<i64>,
    category_id: Option<i64>,
    favorites_only: bool,
) -> Result<Vec<Command>, String> {
    db.get_commands(group_id, category_id, favorites_only)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_command(db: State<'_, Database>, cmd: Command) -> Result<(), String> {
    db.update_command(&cmd).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_command_between(
    db: State<'_, Database>,
    cmd_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), String> {
    db.move_command_between(cmd_id, prev_id, next_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_command(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_command(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_command_favorite(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.toggle_command_favorite(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_commands(db: State<'_, Database>, term: String) -> Result<Vec<Command>, String> {
    db.search_commands(&term).map_err(|e| e.to_string())
}
