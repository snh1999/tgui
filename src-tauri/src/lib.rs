mod constants;
mod database;
mod handlers;

use crate::database::Database;
use crate::handlers::{categories, commands, groups};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| "Failed to get app data dir")?;

            std::fs::create_dir_all(&app_dir).map_err(|_| "Failed to create app data dir")?;

            let db_path = app_dir.join("commands.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");

            app.manage(db); // Directly manage the Database struct
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            categories::create_category,
            categories::get_category,
            categories::get_categories,
            categories::update_category,
            categories::delete_category,
            categories::get_category_command_count,
            groups::create_group,
            groups::get_group,
            groups::get_groups,
            groups::update_group,
            groups::move_group_between,
            groups::delete_group,
            groups::get_group_command_count,
            groups::get_group_tree,
            groups::get_group_path,
            groups::toggle_group_favorite,
            commands::create_command,
            commands::get_command,
            commands::get_commands,
            commands::update_command,
            commands::delete_command,
            commands::search_commands,
            commands::move_command_between,
            commands::toggle_command_favorite,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
