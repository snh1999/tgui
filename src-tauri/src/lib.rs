mod commands;
mod database;
mod errors;

use crate::database::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).unwrap();

            let db_path = app_dir.join("commands.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");

            app.manage(db); // Directly manage the Database struct
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_command,
            commands::get_command,
            commands::get_commands,
            commands::update_command,
            commands::delete_command,
            commands::search_commands,
            commands::move_command_between,
            commands::toggle_favorite,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
