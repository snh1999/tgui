use crate::database::Database;
use crate::handlers::serialize_errors::SerializableError;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn get_setting(db: State<'_, Database>, key: String) -> Result<String, SerializableError> {
    db.get_setting(&key).map_err(|err| err.into())
}

#[tauri::command]
pub fn set_setting(
    db: State<'_, Database>,
    key: String,
    value: String,
) -> Result<(), SerializableError> {
    db.set_setting(&key, &value).map_err(|err| err.into())
}

#[tauri::command]
pub fn reset_settings(db: State<'_, Database>) -> Result<(), SerializableError> {
    db.reset_settings().map_err(|err| err.into())
}

#[tauri::command]
pub fn get_all_settings(
    db: State<'_, Database>,
) -> Result<HashMap<String, String>, SerializableError> {
    db.get_all_settings().map_err(|err| err.into())
}
