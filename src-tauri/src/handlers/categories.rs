use crate::database::{Category, Database};
use crate::errors::SerializableError;
use tauri::State;

#[tauri::command]
pub fn create_category(
    db: State<'_, Database>,
    name: &str,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<i64, SerializableError> {
    db.create_category(name, icon, color)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_category(db: State<'_, Database>, id: i64) -> Result<Category, SerializableError> {
    db.get_category(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_categories(db: State<'_, Database>) -> Result<Vec<Category>, SerializableError> {
    db.get_categories().map_err(|err| err.into())
}

#[tauri::command]
pub fn update_category(
    db: State<'_, Database>,
    id: i64,
    name: &str,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<(), SerializableError> {
    db.update_category(id, name, icon, color)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn delete_category(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.delete_category(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_category_command_count(
    db: State<'_, Database>,
    id: i64,
) -> Result<i64, SerializableError> {
    db.get_category_command_count(id).map_err(|err| err.into())
}
