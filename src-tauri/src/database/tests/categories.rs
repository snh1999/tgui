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
fn test_get_categories_empty() {
    let test_db = TestDb::setup_test_db();
    let categories = test_db.db.get_categories().unwrap();
    assert!(categories.is_empty());
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
fn test_create_category_name_at_max_length() {
    let test_db = TestDb::setup_test_db();
    let name = "a".repeat(Database::MAX_NAME_LENGTH);
    let result = test_db.db.create_category(&name, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_create_category_name_over_max_length() {
    let test_db = TestDb::setup_test_db();
    let name = "a".repeat(Database::MAX_NAME_LENGTH + 1);
    let result = test_db.db.create_category(&name, None, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
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
fn test_update_category_clears_optional_fields() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db
        .db
        .create_category("Dev", Some("icon"), Some("#FFF"))
        .unwrap();

    test_db
        .db
        .update_category(cat_id, "Dev", None, None)
        .unwrap();

    let updated = test_db.db.get_category(cat_id).unwrap();
    assert_eq!(updated.icon, None);
    assert_eq!(updated.color, None);
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

    let cmd = CommandBuilder::new("Build", "cargo build")
        .with_category(cat_id)
        .with_group(group_id)
        .build();

    test_db.db.create_command(&cmd).unwrap();
    test_db.db.create_command(&cmd).unwrap();

    let count = test_db.db.get_category_command_count(cat_id).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_get_category_command_count_excludes_other_categories() {
    let test_db = TestDb::setup_test_db();
    let cat_a = test_db.create_test_category("A");
    let cat_b = test_db.create_test_category("B");
    let group_id = test_db.create_test_group("Group");

    let cmd = CommandBuilder::new("Build", "cargo build")
        .with_category(cat_b)
        .with_group(group_id)
        .build();
    test_db.db.create_command(&cmd).unwrap();

    let count = test_db.db.get_category_command_count(cat_a).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_get_category_command_count_nonexistent_id_returns_zero() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_category_command_count(99999);
    assert!(matches!(result, Ok(0)));
}

#[test]
fn test_get_category_group_count() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Development");

    let group = GroupBuilder::new("Test Group")
        .with_category(cat_id)
        .build();
    test_db.db.create_group(&group).unwrap();
    test_db.db.create_group(&group).unwrap();

    let count = test_db.db.get_category_group_count(cat_id).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_get_category_group_count_excludes_other_categories() {
    let test_db = TestDb::setup_test_db();
    let cat_a = test_db.create_test_category("A");
    let cat_b = test_db.create_test_category("B");

    let group = GroupBuilder::new("Test Group").with_category(cat_a).build();
    test_db.db.create_group(&group).unwrap();

    let count = test_db.db.get_category_group_count(cat_b).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_get_category_group_count_nonexistent_id_returns_zero() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_category_group_count(99999);
    assert!(matches!(result, Ok(0)));
}

#[test]
fn test_delete_category_only_nulls_its_own_commands() {
    let test_db = TestDb::setup_test_db();
    let cat_a = test_db.create_test_category("A");
    let cat_b = test_db.create_test_category("B");
    let group_id = test_db.create_test_group("Group");

    let mut cmd_a = CommandBuilder::new("CmdA", "echo a").build();
    cmd_a.category_id = Some(cat_a);
    cmd_a.group_id = Some(group_id);
    let cmd_a_id = test_db.db.create_command(&cmd_a).unwrap();

    let mut cmd_b = CommandBuilder::new("CmdB", "echo b").build();
    cmd_b.category_id = Some(cat_b);
    cmd_b.group_id = Some(group_id);
    let cmd_b_id = test_db.db.create_command(&cmd_b).unwrap();

    test_db.db.delete_category(cat_a).unwrap();

    assert_eq!(test_db.db.get_command(cmd_a_id).unwrap().category_id, None);
    assert_eq!(
        test_db.db.get_command(cmd_b_id).unwrap().category_id,
        Some(cat_b)
    );
}
