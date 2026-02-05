use crate::database::{Database, Group};
use tauri::State;

#[tauri::command]
pub fn create_group(db: State<'_, Database>, group: Group) -> Result<i64, String> {
    db.create_group(&group).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_group(db: State<'_, Database>, id: i64) -> Result<Group, String> {
    db.get_group(id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_groups(db: State<'_, Database>, parent_id: Option<i64>) -> Result<Vec<Group>, String> {
    db.get_groups(parent_id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn update_group(db: State<'_, Database>, group: Group) -> Result<(), String> {
    db.update_group(&group).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn move_command_between(
    db: State<'_, Database>,
    group_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), String> {
    db.move_group_between(group_id, prev_id, next_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_group(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_group(id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_group_command_count(db: State<'_, Database>, id: i64) -> Result<i64, String> {
    db.get_group_command_count(id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_group_tree(db: State<'_, Database>, root_id: i64) -> Result<Vec<Group>, String> {
    db.get_group_tree(root_id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_group_path(db: State<'_, Database>, group_id: i64) -> Result<Vec<String>, String> {
    db.get_group_path(group_id).map_err(|err| err.to_string())
}
