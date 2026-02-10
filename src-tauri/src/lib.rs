mod constants;
mod database;
mod handlers;

use crate::database::Database;
use crate::handlers::{categories, commands, groups, workflows};
use handlers::logger;
use tauri::Manager;
use tracing::{error, info};

fn error_map(error_message: &str) -> &str {
    error!(error_message);
    error_message
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|_| error_map("Failed to get app data dir"))?;

            std::fs::create_dir_all(&app_dir)
                .map_err(|_| error_map("Failed to create app data dir"))?;

            let guard =
                logger::init(&app_dir).map_err(|_| error_map("Failed to initialize logger"))?;

            info!("TGUI starting");

            let db_path = app_dir.join("commands.db");
            let db = Database::new(&db_path).map_err(|e| {
                error!(error = %e, "Database initialization failed");
                e
            })?;

            info!(db_path = %db_path.display(), "Database initialized successfully");

            app.manage(guard);
            app.manage(db);
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
            workflows::create_workflow,
            workflows::get_workflow,
            workflows::get_workflows,
            workflows::update_workflow,
            workflows::delete_workflow,
            workflows::toggle_favorite_workflow,
            workflows::get_workflow_count_for_category,
            workflows::move_workflow_between,
            workflows::create_workflow_step,
            workflows::get_workflow_step,
            workflows::get_workflow_steps,
            workflows::get_workflow_steps_command_populated,
            workflows::update_workflow_step,
            workflows::delete_workflow_step,
            workflows::move_workflow_step_between,
            workflows::toggle_workflow_step_enabled,
            workflows::get_workflow_step_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
