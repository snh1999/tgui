#[cfg(test)]
mod tests {
    use crate::constants::LOG_PREFIX;
    use crate::handlers::logger;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;
    use time::OffsetDateTime;

    // Helper function to create test log directory
    fn setup_test_logs() -> TempDir {
        TempDir::new().unwrap()
    }

    // Helper to create a log file with content
    fn create_log_file(dir: &Path, date: &str, content: &str) {
        let log_path = dir.join(std::format!("{}.{}", LOG_PREFIX, date));
        let mut file = File::create(log_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_logs_dir() {
        let temp_dir = setup_test_logs();
        let app_dir = temp_dir.path();

        let result = logger::logs_dir(app_dir);
        assert_eq!(result, app_dir.join("logs"));
    }

    #[test]
    fn test_list_log_files_empty_directory() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let result = logger::list_log_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_log_files_excludes_today() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        // Create today's log
        let today = OffsetDateTime::now_utc().date();
        create_log_file(&logs_path, &today.to_string(), "today's log");

        // Create yesterday's log
        let yesterday = today - time::Duration::days(1);
        create_log_file(&logs_path, &yesterday.to_string(), "yesterday's log");

        let result = logger::list_log_files(temp_dir.path()).unwrap();

        // Should only have yesterday's log
        assert_eq!(result.len(), 1);
        assert!(result[0].to_str().unwrap().contains(&yesterday.to_string()));
    }

    #[test]
    fn test_list_log_files_ignores_non_log_files() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        // Create a non-log file
        let txt_file = logs_path.join("readme.txt");
        File::create(txt_file).unwrap();

        // Create an old log file
        let old_date = (OffsetDateTime::now_utc().date() - time::Duration::days(5)).to_string();
        create_log_file(&logs_path, &old_date, "old log");

        let result = logger::list_log_files(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_delete_logs_older_than_invalid_days() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let result = logger::delete_logs_older_than(temp_dir.path(), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_logs_older_than_deletes_old_files() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let old_date = OffsetDateTime::now_utc().date() - time::Duration::days(35);
        create_log_file(&logs_path, &old_date.to_string(), "old log");

        let recent_date = OffsetDateTime::now_utc().date() - time::Duration::days(5);
        create_log_file(&logs_path, &recent_date.to_string(), "recent log");

        let deleted = logger::delete_logs_older_than(temp_dir.path(), 30).unwrap();

        assert_eq!(deleted, 1);
        assert!(!logs_path
            .join(std::format!("{}.{}", LOG_PREFIX, old_date))
            .exists());
        assert!(logs_path
            .join(std::format!("{}.{}", LOG_PREFIX, recent_date))
            .exists());
    }

    #[test]
    fn test_delete_log_by_date_existing_file() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let date = "2025-01-15";
        create_log_file(&logs_path, date, "test log");

        let result = logger::delete_log_by_date(temp_dir.path(), date).unwrap();

        assert!(result);
        assert!(!logs_path
            .join(std::format!("{}.{}", LOG_PREFIX, date))
            .exists());
    }

    #[test]
    fn test_delete_log_by_date_nonexistent_file() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let result = logger::delete_log_by_date(temp_dir.path(), "2025-01-15");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_all_logs() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        // Create multiple log files
        create_log_file(&logs_path, "2025-01-15", "log 1");
        create_log_file(&logs_path, "2025-01-16", "log 2");
        create_log_file(&logs_path, "2025-01-17", "log 3");

        // Create a non-log file that should not be deleted
        File::create(logs_path.join("readme.txt")).unwrap();

        let deleted = logger::delete_all_logs(temp_dir.path()).unwrap();

        assert_eq!(deleted, 3);
        assert!(logs_path.join("readme.txt").exists());
    }

    #[test]
    fn test_get_recent_logs_single_day() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let today = OffsetDateTime::now_utc().date();
        let log_content = "line1\nline2\nline3\n";
        create_log_file(&logs_path, &today.to_string(), log_content);

        let result = logger::get_recent_logs(temp_dir.path(), None, None).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "line1");
        assert_eq!(result[1], "line2");
        assert_eq!(result[2], "line3");
    }

    #[test]
    fn test_get_recent_logs_multiple_days() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let today = OffsetDateTime::now_utc().date();
        create_log_file(&logs_path, &today.to_string(), "today line 1\ntoday line 2");

        let yesterday = today - time::Duration::days(1);
        create_log_file(
            &logs_path,
            &yesterday.to_string(),
            "yesterday line 1\nyesterday line 2",
        );

        let result = logger::get_recent_logs(temp_dir.path(), Some(2), None).unwrap();

        // Should have header + lines from both days
        assert!(result.len() >= 4);
        assert!(result.iter().any(|line| line.contains("=== Logs from")));
    }

    #[test]
    fn test_get_recent_logs_line_limit() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let today = OffsetDateTime::now_utc().date();
        let mut content = String::new();
        for i in 1..=200 {
            content.push_str(&std::format!("line {}\n", i));
        }
        create_log_file(&logs_path, &today.to_string(), &content);

        let result = logger::get_recent_logs(temp_dir.path(), None, Some(50)).unwrap();

        // Should be capped at 50 lines
        assert_eq!(result.len(), 50);
        // Should be the LAST 50 lines
        assert_eq!(result[0], "line 151");
        assert_eq!(result[49], "line 200");
    }

    #[test]
    fn test_get_recent_logs_max_limits() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let today = OffsetDateTime::now_utc().date();
        create_log_file(&logs_path, &today.to_string(), "test");

        // Test days limit (max 30)
        let result = logger::get_recent_logs(temp_dir.path(), Some(50), None);
        assert!(result.is_ok());

        // Test lines limit (max 1000)
        let result = logger::get_recent_logs(temp_dir.path(), None, Some(5000));
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_recent_logs_missing_files() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        // Request logs from days that don't exist
        let result = logger::get_recent_logs(temp_dir.path(), Some(7), None).unwrap();

        // Should return empty vector, not error
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_get_recent_logs_default_values() {
        let temp_dir = setup_test_logs();
        let logs_path = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_path).unwrap();

        let today = OffsetDateTime::now_utc().date();
        let mut content = String::new();
        for i in 1..=150 {
            content.push_str(&std::format!("line {}\n", i));
        }
        create_log_file(&logs_path, &today.to_string(), &content);

        // Use defaults: 1 day, 100 lines
        let result = logger::get_recent_logs(temp_dir.path(), None, None).unwrap();

        // Should get last 100 lines
        assert_eq!(result.len(), 100);
        assert_eq!(result[0], "line 51");
        assert_eq!(result[99], "line 150");
    }
}
