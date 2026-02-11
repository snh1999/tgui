use super::*;
use crate::constants::CATEGORIES_TABLE;

#[test]
fn test_create_and_get_category() {
    let test_db = TestDb::setup_test_db();
    let cat_name = "Development";
    let cat_id = test_db.create_test_category(cat_name);

    assert!(cat_id > 0);
    let category = test_db.db.get_category(cat_id).unwrap();
    assert_eq!(category.name, cat_name);
    assert_eq!(category.icon, None);
    assert_eq!(category.color, None);
}

#[test]
fn test_create_category_with_icon_and_color() {
    let test_db = TestDb::setup_test_db();
    let cat_name = "Development";
    let icon = "smiley";
    let color = "#FF0000";

    let cat_id = test_db
        .db
        .create_category(cat_name, Some(icon), Some(color))
        .unwrap();
    let category = test_db.db.get_category(cat_id).unwrap();

    assert_eq!(category.icon, Some(icon.to_string()));
    assert_eq!(category.color, Some(color.to_string()));
}

#[test]
fn test_create_category_empty_name() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.create_category("", None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_category_whitespace_name() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.create_category("   ", None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_category_duplicate_name() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_category("Unique");

    let result = test_db.db.create_category("Unique", None, None);
    assert!(matches!(result, Err(DatabaseError::Internal(msg)) if msg.contains("UNIQUE")));
}

#[test]
fn test_get_category_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_category(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: CATEGORIES_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_all_categories_ordered() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_category("Zebra");
    test_db.create_test_category("Alpha");
    test_db.create_test_category("Beta");

    let categories = test_db.db.get_categories().unwrap();
    assert_eq!(categories.len(), 3);
    assert_eq!(categories[0].name, "Alpha");
    assert_eq!(categories[1].name, "Beta");
    assert_eq!(categories[2].name, "Zebra");
}

#[test]
fn test_update_category() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Original");

    test_db
        .db
        .update_category(cat_id, "Updated", Some("🎨"), Some("#00FF00"))
        .unwrap();

    let updated = test_db.db.get_category(cat_id).unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.icon, Some("🎨".to_string()));
    assert_eq!(updated.color, Some("#00FF00".to_string()));
}

#[test]
fn test_update_category_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.update_category(99999, "Ghost", None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: CATEGORIES_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_update_category_to_empty_name() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Valid");

    let result = test_db.db.update_category(cat_id, "", None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_update_category_duplicate_name() {
    let test_db = TestDb::setup_test_db();
    test_db.create_test_category("First");
    let cat2_id = test_db.create_test_category("Second");

    let result = test_db.db.update_category(cat2_id, "First", None, None);
    assert!(matches!(result, Err(DatabaseError::Internal(msg)) if msg.contains("UNIQUE")));
}

#[test]
fn test_delete_category() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("To Delete");

    test_db.db.delete_category(cat_id).unwrap();

    let result = test_db.db.get_category(cat_id);
    assert!(matches!(result, Err(DatabaseError::NotFound { .. })));
}

#[test]
fn test_delete_category_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.delete_category(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: CATEGORIES_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_delete_category_sets_commands_to_null() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Development");
    let group_id = test_db.create_test_group("Test Group");

    let mut cmd = CommandBuilder::new("Test", "echo").build();
    cmd.category_id = Some(cat_id);
    cmd.group_id = Some(group_id);

    let cmd_id = test_db.db.create_command(&cmd).unwrap();

    test_db.db.delete_category(cat_id).unwrap();

    let command = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(command.category_id, None);
}

#[test]
fn test_get_category_command_count() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Development");
    let group_id = test_db.create_test_group("Test Group");

    let mut cmd1 = CommandBuilder::new("Build", "cargo build").build();
    cmd1.category_id = Some(cat_id);
    cmd1.group_id = Some(group_id);

    let mut cmd2 = CommandBuilder::new("Test", "cargo test").build();
    cmd2.category_id = Some(cat_id);
    cmd2.group_id = Some(group_id);

    test_db.db.create_command(&cmd1).unwrap();
    test_db.db.create_command(&cmd2).unwrap();

    let count = test_db.db.get_category_command_count(cat_id).unwrap();
    assert_eq!(count, 2);
}
