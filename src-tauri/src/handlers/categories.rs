use crate::database::{Category, Database, Group};
use tauri::State;

#[tauri::command]
pub fn create_category(db: State<'_, Database>, name: &str,
                       icon: Option<&str>,
                       color: Option<&str>) -> Result<i64, String> {
    db.create_category(name, icon, color).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_category(db: State<'_, Database>, id: i64) -> Result<Category, String> {
    db.get_category(id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_categories(db: State<'_, Database>, parent_id: Option<i64>) -> Result<Vec<Category>, String> {
    db.get_categories().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn update_category(db: State<'_, Database>,         id: i64,
                       name: &str,
                       icon: Option<&str>,
                       color: Option<&str>,) -> Result<(), String> {
    db.update_category(id, name, icon, color).map_err(|err| err.to_string())
}



#[tauri::command]
pub fn delete_category(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete_category(id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_category_command_count(db: State<'_, Database>, id: i64) -> Result<i64, String> {
    db.get_category_command_count(id)
        .map_err(|err| err.to_string())
}