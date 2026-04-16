use crate::database::{CategoryFilter, Database, Group, GroupFilter, GroupNode};
use crate::handlers::serialize_errors::SerializableError;
use tauri::State;

#[tauri::command]
pub fn create_group(db: State<'_, Database>, group: Group) -> Result<i64, SerializableError> {
    db.create_group(&group).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_group(db: State<'_, Database>, id: i64) -> Result<Group, SerializableError> {
    db.get_group(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_groups(
    db: State<'_, Database>,
    parent_id: GroupFilter,
    category_id: CategoryFilter,
    favorites_only: bool,
) -> Result<Vec<Group>, SerializableError> {
    db.get_groups(parent_id, category_id, favorites_only)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn update_group(db: State<'_, Database>, group: Group) -> Result<(), SerializableError> {
    db.update_group(&group).map_err(|err| err.into())
}

#[tauri::command]
pub fn move_group_between(
    db: State<'_, Database>,
    id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), SerializableError> {
    db.move_group_between(id, prev_id, next_id)
        .map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_group(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.delete_group(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_group_tree(
    db: State<'_, Database>,
    root_id: i64,
) -> Result<GroupNode, SerializableError> {
    db.get_group_tree(root_id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_group_path(
    db: State<'_, Database>,
    root_id: i64,
) -> Result<Vec<String>, SerializableError> {
    db.get_group_path(root_id).map_err(|err| err.into())
}

#[tauri::command]
pub fn toggle_group_favorite(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.toggle_group_favorite(id).map_err(|err| err.into())
}


#[tauri::command]
pub fn get_groups_by_directory(db: State<'_, Database>, directory: Option<&str>) -> Result<Vec<Group>, SerializableError> {
   db.get_groups_by_directory(directory).map_err(|err| err.into())
    }

#[tauri::command]
pub fn replace_groups_directory(
    db: State<'_, Database>,
    ids: Vec<i64>,
    new_directory: Option<&str>,
) -> Result<usize, SerializableError> {
    db.replace_groups_directory(ids, new_directory).map_err(|err| err.into())
}

#[tauri::command]
pub fn duplicate_groups(
    db: State<'_, Database>,
    ids: Vec<i64>,
    name_prefix: &str,
    recursive: bool,
) -> Result<Vec<i64>, SerializableError> {
    db.duplicate_groups(ids, name_prefix, recursive).map_err(|err| err.into())
}