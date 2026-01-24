use super::*;

#[test]
fn test_initialize_settings_creates_defaults() {
    let test_db = TestDb::setup_test_db();

    let theme = test_db.db.get_setting("theme").unwrap();
    assert_eq!(theme, "system");

    let shell = test_db.db.get_setting("default_shell").unwrap();
    assert_eq!(shell, "/bin/bash");
}

#[test]
fn test_get_setting_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_setting("nonexistent");
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: "setting",
            ..
        })
    ));
}

#[test]
fn test_set_setting_validates_unknown_key() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.set_setting("unknown_key", "value");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "key", .. })
    ));
}

#[test]
fn test_set_setting_validates_number_type() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.set_setting("log_buffer_size", "not_a_number");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    // Valid number should work
    test_db.db.set_setting("log_buffer_size", "5000").unwrap();
    assert_eq!(test_db.db.get_setting("log_buffer_size").unwrap(), "5000");
}

#[test]
fn test_set_setting_validates_boolean_type() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.set_setting("auto_scroll_logs", "maybe");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    // Valid booleans
    test_db.db.set_setting("auto_scroll_logs", "true").unwrap();
    assert_eq!(test_db.db.get_setting("auto_scroll_logs").unwrap(), "true");

    test_db.db.set_setting("auto_scroll_logs", "false").unwrap();
    assert_eq!(test_db.db.get_setting("auto_scroll_logs").unwrap(), "false");
}

#[test]
fn test_set_setting_theme_allows_any_string() {
    let test_db = TestDb::setup_test_db();
    test_db.db.set_setting("theme", "custom-dark").unwrap();
    assert_eq!(test_db.db.get_setting("theme").unwrap(), "custom-dark");
}

#[test]
fn test_set_setting_overwrites_existing() {
    let test_db = TestDb::setup_test_db();

    test_db.db.set_setting("theme", "dark").unwrap();
    assert_eq!(test_db.db.get_setting("theme").unwrap(), "dark");

    test_db.db.set_setting("theme", "light").unwrap();
    assert_eq!(test_db.db.get_setting("theme").unwrap(), "light");
}

#[test]
fn test_reset_settings() {
    let test_db = TestDb::setup_test_db();

    // Change some settings
    test_db.db.set_setting("theme", "dark").unwrap();
    test_db.db.set_setting("log_buffer_size", "9999").unwrap();

    // Reset
    test_db.db.reset_settings().unwrap();

    // Should be back to defaults
    assert_eq!(test_db.db.get_setting("theme").unwrap(), "system");
    assert_eq!(test_db.db.get_setting("log_buffer_size").unwrap(), "10000");
}

#[test]
fn test_get_all_settings() {
    let test_db = TestDb::setup_test_db();

    test_db.db.set_setting("theme", "custom").unwrap();

    let all_settings = test_db.db.get_all_settings().unwrap();
    assert!(all_settings.contains_key("theme"));
    assert!(all_settings.contains_key("default_shell"));
    assert!(all_settings.contains_key("log_buffer_size"));
    assert_eq!(all_settings.get("theme").unwrap(), "custom");
}

#[test]
fn test_setting_updated_at_changes() {
    let test_db = TestDb::setup_test_db();

    let initial_theme = test_db.db.get_setting("theme").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));

    test_db.db.set_setting("theme", "dark").unwrap();

    // The value should have changed
    assert_ne!(initial_theme, test_db.db.get_setting("theme").unwrap());
}
