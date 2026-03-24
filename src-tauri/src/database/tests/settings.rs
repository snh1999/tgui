use super::*;

#[test]
fn test_initialize_settings_creates_defaults() {
    let test_db = TestDb::setup_test_db();

    let theme = test_db.db.get_setting("theme").unwrap();
    assert_eq!(theme, "system");

    let shell = test_db.db.get_setting("default_shell").unwrap();
    assert_eq!(shell, "sh");
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
    test_db.db.set_setting("theme", "tgui-dark").unwrap();
    assert_eq!(test_db.db.get_setting("theme").unwrap(), "tgui-dark");
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

    test_db.db.set_setting("theme", "tgui").unwrap();

    let all_settings = test_db.db.get_all_settings().unwrap();
    assert!(all_settings.contains_key("theme"));
    assert!(all_settings.contains_key("default_shell"));
    assert!(all_settings.contains_key("log_buffer_size"));
    assert_eq!(all_settings.get("theme").unwrap(), "tgui");
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

#[test]
fn test_initialize_settings_is_idempotent() {
    let test_db = TestDb::setup_test_db();
    test_db.db.set_setting("theme", "dark").unwrap();

    test_db.db.initialize_settings().unwrap();

    assert_eq!(test_db.db.get_setting("theme").unwrap(), "dark");
}

#[test]
fn test_set_setting_max_concurrent_processes_validates_number() {
    let test_db = TestDb::setup_test_db();

    let result = test_db
        .db
        .set_setting("max_concurrent_processes", "not_a_number");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    test_db
        .db
        .set_setting("max_concurrent_processes", "10")
        .unwrap();
    assert_eq!(
        test_db.db.get_setting("max_concurrent_processes").unwrap(),
        "10"
    );
}

#[test]
fn test_set_setting_rejects_float_for_numeric_fields() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.set_setting("log_buffer_size", "1.5");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    let result = test_db.db.set_setting("max_concurrent_processes", "4.0");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));
}

#[test]
fn test_set_setting_warn_before_kill_validates_boolean() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.set_setting("warn_before_kill", "yes");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    test_db.db.set_setting("warn_before_kill", "false").unwrap();
    assert_eq!(test_db.db.get_setting("warn_before_kill").unwrap(), "false");
}

#[test]
fn test_set_setting_kill_process_tree_validates_boolean() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.set_setting("kill_process_tree_by_default", "1");
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "value", .. })
    ));

    test_db
        .db
        .set_setting("kill_process_tree_by_default", "true")
        .unwrap();
    assert_eq!(
        test_db
            .db
            .get_setting("kill_process_tree_by_default")
            .unwrap(),
        "true"
    );
}

#[test]
fn test_set_setting_available_shells_accepts_any_string() {
    let test_db = TestDb::setup_test_db();
    // falls through to _ => Ok(()), no validation on this key
    test_db
        .db
        .set_setting("available_shells", r#"["/bin/bash","/bin/zsh"]"#)
        .unwrap();
    assert_eq!(
        test_db.db.get_setting("available_shells").unwrap(),
        r#"["/bin/bash","/bin/zsh"]"#
    );
}

#[test]
fn test_get_all_settings_returns_all_default_keys() {
    let test_db = TestDb::setup_test_db();
    let all = test_db.db.get_all_settings().unwrap();

    for key in &[
        "theme",
        "log_buffer_size",
        "max_concurrent_processes",
        "auto_scroll_logs",
        "warn_before_kill",
        "kill_process_tree_by_default",
        "available_shells",
        "default_shell",
    ] {
        assert!(all.contains_key(*key), "missing key: {key}");
    }
}
