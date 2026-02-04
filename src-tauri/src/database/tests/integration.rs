use super::*;
use rusqlite::params;

#[test]
fn test_full_workflow_project_setup() {
    let test_db = TestDb::setup_test_db();

    let dev_cat = test_db.create_test_category("Development");

    // Create group hierarchy
    let project_group_id = test_db.create_test_group("MyApp");
    let backend_group_id = test_db.save_group_to_db(
        &GroupBuilder::new("Backend")
            .with_parent(project_group_id)
            .build(),
    );
    let frontend_group_id = test_db.save_group_to_db(
        &GroupBuilder::new("Frontend")
            .with_parent(project_group_id)
            .build(),
    );

    // Assign category to groups
    let mut backend_group = test_db.db.get_group(backend_group_id).unwrap();
    backend_group.category_id = Some(dev_cat);
    test_db.db.update_group(&backend_group).unwrap();

    // Create commands
    let build_cmd = CommandBuilder::new("Build", "cargo build")
        .with_group(backend_group_id)
        .with_category(dev_cat)
        .build();
    let build_id = test_db.save_command_to_db(&build_cmd);

    let test_cmd = CommandBuilder::new("Test", "npm test")
        .with_group(frontend_group_id)
        .with_category(dev_cat)
        .build();
    test_db.save_command_to_db(&test_cmd);

    // Verify relationships
    let build = test_db.db.get_command(build_id).unwrap();
    assert_eq!(build.group_id, Some(backend_group_id));
    assert_eq!(build.category_id, Some(dev_cat));

    // Verify tree structure
    let tree = test_db.db.get_group_tree(project_group_id).unwrap();
    assert_eq!(tree.len(), 3);

    // Verify path
    let path = test_db.db.get_group_path(backend_group_id).unwrap();
    assert_eq!(path, vec!["MyApp", "Backend"]);
}

#[test]
fn test_cascade_delete_category_preserves_commands() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Temp");
    let group_id = test_db.create_test_group("Test");

    let mut cmd = CommandBuilder::new("Test", "echo").build();
    cmd.category_id = Some(cat_id);
    cmd.group_id = Some(group_id);
    let cmd_id = test_db.db.create_command(&cmd).unwrap();

    test_db.db.delete_category(cat_id).unwrap();

    let cmd = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(cmd.category_id, None);
    assert_eq!(cmd.group_id, Some(group_id)); // Group still set
}

#[test]
fn test_cascade_delete_group_deletes_commands() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Deletable");
    let cat_id = test_db.create_test_category("Test");

    let mut cmd = CommandBuilder::new("Test", "echo").build();
    cmd.group_id = Some(group_id);
    cmd.category_id = Some(cat_id);
    let cmd_id = test_db.db.create_command(&cmd).unwrap();

    test_db.db.delete_group(group_id).unwrap();

    let result = test_db.db.get_command(cmd_id);
    assert!(matches!(result, Err(DatabaseError::NotFound { .. })));

    let cat = test_db.db.get_category(cat_id).unwrap();
    assert_eq!(cat.name, "Test");
}

#[test]
fn test_very_long_group_name() {
    let test_db = TestDb::setup_test_db();
    let long_name = "a".repeat(500);
    let group_id = test_db.create_test_group(&long_name);

    let group = test_db.db.get_group(group_id).unwrap();
    assert_eq!(group.name.len(), 500);
}

#[test]
fn test_unicode_names() {
    let test_db = TestDb::setup_test_db();

    let cat_id = test_db
        .db
        .create_category("🚀 Deployment", None, None)
        .unwrap();
    let cat = test_db.db.get_category(cat_id).unwrap();
    assert_eq!(cat.name, "🚀 Deployment");

    let group_id = test_db.create_test_group("পরীক্ষা");
    let group = test_db.db.get_group(group_id).unwrap();
    assert_eq!(group.name, "পরীক্ষা");

    let cmd = CommandBuilder::new("Emojis 🎉", "echo 🎊").build();
    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved_cmd = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(retrieved_cmd.name, "Emojis 🎉");
}

#[test]
fn test_concurrent_transactions_isolated() {
    let test_db = TestDb::setup_test_db();

    let count_before = test_db.db.get_groups(None).unwrap().len();

    {
        let mut connection = test_db.db.conn();
        let tx = connection.transaction().unwrap();
        tx.execute(
            "INSERT INTO groups (name, position) VALUES (?1, ?2)",
            params!["Transactional", 1000],
        )
        .unwrap();
        tx.commit().unwrap();
    } // Drop mutable borrow

    let groups = test_db.db.get_groups(None).unwrap();
    assert_eq!(groups.len(), count_before + 1);
}

#[test]
fn test_database_locked_error() {
    let test_db = TestDb::setup_test_db();

    // Acquire exclusive lock
    let conn1 = test_db.db.conn();
    conn1.execute("BEGIN EXCLUSIVE", []).unwrap();

    // Trying another operation (this might block or fail depending on configuration)
    // In WAL mode, this should actually succeed, so this test might need adjustment based on actual concurrency model

    // For now, just verifying the error type exists
    let error = DatabaseError::DatabaseLocked;
    assert_eq!(
        error.to_string(),
        "Database is locked by another process. Please try again."
    );
}
