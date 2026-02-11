use crate::constants::LOG_PREFIX;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::error_map;
use std::fs::File;
use std::io::{BufRead, BufReader};

const INVALID_INPUT: &str = "Invalid input";
const NOT_FOUND: &str = "Resource not found";
const DIRECTORY_ERROR: &str = "Failed to read/modify file/directory";
const INTERNAL_ERROR: &str = "An internal error occurred";

/// must return guard as we are using non-blocking.
/// it ensures all logs are flushed before app closing
pub fn init(app_dir: &PathBuf) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let logs_dir = app_dir.join("logs");
    fs::create_dir_all(&logs_dir).map_err(|_| error_map("Failed to create logs dir"))?;

    delete_logs_older_than(app_dir.as_path(), 30)?;

    let file_appender = tracing_appender::rolling::daily(&logs_dir, LOG_PREFIX);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(non_blocking)
        .json();

    let console_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_writer(std::io::stdout);

    // Filter: RUST_LOG=debug,tgui=info,sqlx=warn env var
    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Logging initialized");
    debug!(log_directory = %logs_dir.display(), "Log files location");

    Ok(guard)
}

#[macro_export]
macro_rules! db_span {
    ($operation:expr, $entity:expr, $id:expr) => {
        tracing::info_span!(
            "db_operation",
            operation = $operation,
            entity = $entity,
            entity_id = $id
        )
    };
}

#[tauri::command]
pub fn logs_dir(app_dir: &Path) -> PathBuf {
    app_dir.join("logs")
}

#[tauri::command]
pub fn list_log_files(app_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let logs_dir = logs_dir(app_dir);
    let mut log_files = Vec::new();

    if logs_dir.exists() {
        let today = format!("{}.{}", LOG_PREFIX, OffsetDateTime::now_utc().date());

        let entries = fs::read_dir(&logs_dir).map_err(|e| {
            error!(error = %e);
            "Failed to read directory."
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                error!(error = %e);
                INTERNAL_ERROR
            })?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with(LOG_PREFIX) {
                    if filename != today {
                        log_files.push(path);
                    }
                }
            }
        }
    }

    Ok(log_files)
}

#[tauri::command]
pub fn delete_logs_older_than(app_dir: &Path, days: i64) -> Result<usize, String> {
    if days < 1 {
        warn!("Invalid days parameter: {}", days);
        return Err(INVALID_INPUT.to_string());
    }

    let logs_dir = logs_dir(app_dir);
    let cutoff_date = OffsetDateTime::now_utc().date() - time::Duration::days(days);
    let mut deleted_count = 0;

    info!(days = days, cutoff_date = %cutoff_date, "Starting log cleanup");

    let date_format = time::format_description::parse("[year]-[month]-[day]").map_err(|e| {
        error!(error = %e);
        INTERNAL_ERROR
    })?;

    let entries = fs::read_dir(&logs_dir).map_err(|e| {
        error!(error = %e);
        DIRECTORY_ERROR
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!(error = %e);
            INTERNAL_ERROR
        })?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            let date_part = filename
                .strip_prefix(&format!("{}.", LOG_PREFIX))
                .unwrap_or(filename);

            if let Ok(file_date) = time::Date::parse(date_part, &date_format) {
                if file_date < cutoff_date {
                    match fs::remove_file(&path) {
                        Ok(_) => {
                            info!(file = %filename, "Deleted old log file");
                            deleted_count += 1;
                        }
                        Err(e) => {
                            error!(file = %filename, error = %e, "Failed to delete log file");
                        }
                    }
                }
            } else {
                debug!(filename = %filename, "Failed to parse date from filename");
            }
        }
    }

    info!(deleted_count = deleted_count, "Log cleanup completed");
    Ok(deleted_count)
}

#[tauri::command]
pub fn delete_log_by_date(app_dir: &Path, date: &str) -> Result<bool, String> {
    let logs_dir = logs_dir(app_dir);
    let log_file = logs_dir.join(std::format!("{}.{}", LOG_PREFIX, date));

    if !is_valid_date(date) {
        warn!(date = date, "Invalid date format provided");
        return Err(INVALID_INPUT.to_string());
    }

    // let canonical_log = log_file.canonicalize().ok();
    // let canonical_base = logs_dir.canonicalize().ok();
    //
    // if let (Some(log), Some(base)) = (canonical_log, canonical_base) {
    //     if !log.starts_with(&base) {
    //         warn!(date = date, "Path traversal attempt detected");
    //         return Err(LogError::InvalidInput);
    //     }
    // }

    if log_file.exists() {
        fs::remove_file(&log_file).map_err(|e| {
            error!(error = %e);
            DIRECTORY_ERROR
        })?;
        info!(date = date, "Deleted log file of date");
        Ok(true)
    } else {
        warn!(date = date, "Log file not found for deletion");
        Err(NOT_FOUND.to_string())
    }
}

fn is_valid_date(date: &str) -> bool {
    if date.contains('/') || date.contains('\\') || date.contains("..") {
        return false;
    }
    if date.len() != 10 {
        return false;
    }

    for char in date.chars() {
        if !char.is_numeric() && char != '-' {
            return false;
        }
    }

    true
}

#[tauri::command]
pub fn delete_all_logs(app_dir: &Path) -> Result<usize, String> {
    let logs_dir = logs_dir(app_dir);
    let mut deleted_count = 0;

    if logs_dir.exists() {
        let log_files = list_log_files(app_dir)?;

        for log_file in log_files {
            fs::remove_file(&log_file).map_err(|e| {
                error!(error = %e, "Failed to delete log file");
                INTERNAL_ERROR
            })?;
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

/// Get recent log entries from log files within the specified number of days
/// Returns logs from today going back N days, with optional line limit per day
#[tauri::command]
pub fn get_recent_logs(
    app_dir: &Path,
    days: Option<usize>,
    max_lines_per_day: Option<usize>,
) -> Result<Vec<String>, String> {
    let logs_dir = logs_dir(&app_dir);
    let days_to_fetch = days.unwrap_or(1).min(30);
    let lines_per_day = max_lines_per_day.unwrap_or(100).min(1000);

    let mut all_logs = Vec::new();
    let today = OffsetDateTime::now_utc().date();

    for day_offset in 0..days_to_fetch {
        let date = today - time::Duration::days(day_offset as i64);
        let log_file = logs_dir.join(std::format!("{}.{}", LOG_PREFIX, date));

        if !log_file.exists() {
            debug!(date = %date, "Log file not found, skipping");
            continue;
        }

        match File::open(&log_file) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut day_lines: Vec<String> = reader
                    .lines()
                    .collect::<Result<_, _>>()
                    .map_err(|e| std::format!("Failed to read log file {}: {}", date, e))?;

                // Take last N lines from each day
                let start_index = day_lines.len().saturating_sub(lines_per_day);
                day_lines.drain(..start_index);

                // Prepend date header if we have multiple days
                if days_to_fetch > 1 && !day_lines.is_empty() {
                    all_logs.push(std::format!("=== Logs from {} ===", date));
                }

                all_logs.extend(day_lines);
            }
            Err(e) => {
                warn!(date = %date, error = %e, "Failed to open log file");
            }
        }
    }

    info!(
        days = days_to_fetch,
        total_lines = all_logs.len(),
        "Retrieved recent logs"
    );

    Ok(all_logs)
}
