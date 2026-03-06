mod constants;
mod database;
mod handlers;
mod process;
mod utils;

use crate::database::Database;
use crate::handlers::{
    categories, commands, execution_history, groups, process_handler, settings, workflows,
};
use crate::process::manager::ProcessManager;
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
            let pm = ProcessManager::new(db.clone(), Some(app.handle().clone()));
            pm.detect_and_mark_orphans();

            info!(db_path = %db_path.display(), "Database initialized successfully");

            app.manage(guard);
            app.manage(db);
            app.manage(pm);
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
            execution_history::get_execution_history,
            execution_history::get_command_execution_history,
            execution_history::get_workflow_execution_history,
            execution_history::get_running_commands,
            execution_history::cleanup_command_history,
            execution_history::cleanup_history_older_than,
            execution_history::get_command_execution_stats,
            process_handler::spawn_command,
            process_handler::kill_process,
            process_handler::get_running_processes,
            process_handler::get_process_status,
            process_handler::get_log_buffer,
            process_handler::clear_log_buffer,
            process_handler::stop_all_processes,
            process_handler::get_tray_status,
            process_handler::get_valid_shells,
            logger::logs_dir,
            logger::list_log_files,
            logger::delete_logs_older_than,
            logger::delete_log_by_date,
            logger::delete_all_logs,
            logger::get_recent_logs,
            settings::get_setting,
            settings::set_setting,
            settings::reset_settings,
            settings::get_all_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
