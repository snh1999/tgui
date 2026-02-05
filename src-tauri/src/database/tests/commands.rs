use super::*;
use crate::database::builders::CommandBuilder;
use rusqlite::params;
use std::collections::HashMap;

#[test]
fn test_command_builder_pattern() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");
    let category_id = test_db.create_test_category("Test Category");

    let mut cmd = CommandBuilder::new("Test", "cargo")
        .with_group(group_id)
        .with_args(vec!["test", "--release"])
        .with_category(category_id)
        .with_env("RUST_LOG", "debug")
        .build();

    cmd.shell = Some("test".to_string());
    cmd.position = 11;
    cmd.working_directory = Some("~/dir".to_string());
    let id = test_db.db.create_command(&cmd).unwrap();

    let retrieved_cmd = test_db.db.get_command(id).unwrap();
    assert_eq!(retrieved_cmd.name, cmd.name);
    assert_eq!(retrieved_cmd.description, cmd.description);
    assert_eq!(retrieved_cmd.arguments, cmd.arguments);
    assert_eq!(retrieved_cmd.working_directory, cmd.working_directory);
    assert_eq!(retrieved_cmd.env_vars, cmd.env_vars);
    assert_eq!(retrieved_cmd.shell, cmd.shell);
    assert_eq!(retrieved_cmd.category_id, cmd.category_id);
    assert_eq!(retrieved_cmd.is_favorite, false);
    assert_eq!(retrieved_cmd.position, Database::POSITION_GAP);
}

#[test]
fn test_create_command_and_get_command() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let name = "Test Command";
    let command = "ls";
    let cmd_id = test_db.create_test_command(name, command, Some(group_id));
    assert!(cmd_id > 0);

    let retrieved = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(retrieved.name, name);
    assert_eq!(retrieved.command, command);
    assert_eq!(retrieved.group_id, Some(group_id));
}

#[test]
fn test_create_command_without_group() {
    let test_db = TestDb::setup_test_db();

    let cmd_id = test_db.create_test_command("Standalone", "echo hello", None);

    let retrieved = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(retrieved.group_id, None);
}

#[test]
fn test_create_command_with_all_fields() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let env_var1 = ("key", "value");

    let mut command = CommandBuilder::new("Test", "ls")
        .with_args(vec!["test", "--release"])
        .with_group(group_id)
        .with_env(env_var1.0, env_var1.1)
        .build();

    command.description = Some("Run tests in release mode".to_string());
    command.working_directory = Some("/tmp".to_string());
    command.shell = Some("/bin/zsh".to_string());
    command.is_favorite = true;

    let cmd_id = test_db.save_command_to_db(&command);
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.name, command.name);
    assert_eq!(retrieved.description, command.description);
    assert_eq!(retrieved.working_directory, command.working_directory);
    assert_eq!(retrieved.shell, command.shell);
    assert_eq!(retrieved.category_id, command.category_id);
    assert_eq!(retrieved.position, Database::POSITION_GAP);
    assert_eq!(retrieved.arguments.len(), command.arguments.len());
    assert_eq!(
        retrieved.env_vars.as_ref().unwrap().get(env_var1.0),
        Some(&env_var1.1.to_string())
    );
    assert!(retrieved.is_favorite);
}

#[test]
fn test_create_command_duplicate_name() {
    let test_db = TestDb::setup_test_db();

    let name = "Test Name";
    let id1 = test_db.create_test_command(name, "ls", None);
    let id2 = test_db.create_test_command(name, "ls -la", None);

    assert_ne!(id1, id2);
}

#[test]
fn test_create_command_empty_name() {
    let test_db = TestDb::setup_test_db();
    let command = CommandBuilder::new("", "echo test").build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_command_whitespace_name() {
    let test_db = TestDb::setup_test_db();
    let command = CommandBuilder::new("      ", "echo test").build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_command_empty_command() {
    let test_db = TestDb::setup_test_db();
    let command = CommandBuilder::new("Test", "").build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "command",
            ..
        })
    ));
}

#[test]
fn test_create_command_whitespace_command() {
    let test_db = TestDb::setup_test_db();
    let command = CommandBuilder::new("Test", "    ").build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "command",
            ..
        })
    ));
}

#[test]
fn test_create_command_invalid_env_var_key() {
    let test_db = TestDb::setup_test_db();
    let mut env_vars = HashMap::new();
    env_vars.insert("INVALID KEY!".to_string(), "value".to_string());

    let command = CommandBuilder::new("Test", "pwd")
        .with_env("INVALID KEY!", "value")
        .build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "env_vars",
            ..
        })
    ));
}

#[test]
fn test_get_command_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_command(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: "command",
            id: 99999
        })
    ));
}

#[test]
fn test_get_commands_by_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.create_test_command("Cmd3", "echo 3", None);

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands.len(), 2);
    assert!(commands.iter().all(|c| c.group_id == Some(group_id)));
}

#[test]
fn test_get_commands_root_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", None);
    test_db.create_test_command("Cmd3", "echo 3", None);
    test_db.create_test_command("Cmd3", "echo 3", None);

    let commands = test_db.db.get_commands(None, None, false).unwrap();
    assert_eq!(commands.len(), 3);
    assert!(commands.iter().all(|c| c.group_id == None));
}

#[test]
fn test_get_commands_by_none_category() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.save_command_to_db(
        &CommandBuilder::new("cmd3", "echo 3")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands.len(), 3);
}
#[test]
fn test_get_commands_by_category() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.save_command_to_db(
        &CommandBuilder::new("cmd3", "echo 3")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    let commands = test_db
        .db
        .get_commands(Some(group_id), Some(category_id), false)
        .unwrap();
    assert_eq!(commands.len(), 1);
    assert!(commands.iter().all(|c| c.category_id == Some(category_id)));
}

#[test]
fn test_get_commands_by_category_and_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.save_command_to_db(
        &CommandBuilder::new("cmd3", "echo 3")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    test_db.save_command_to_db(
        &CommandBuilder::new("cmd4", "echo 4")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    test_db.save_command_to_db(
        &CommandBuilder::new("cmd5", "echo 5")
            .with_category(category_id)
            .build(),
    );

    let commands = test_db
        .db
        .get_commands(Some(group_id), Some(category_id), false)
        .unwrap();
    assert_eq!(commands.len(), 2);
    assert!(commands.iter().all(|c| c.group_id == Some(group_id)));
    assert!(commands.iter().all(|c| c.category_id == Some(category_id)));
}

#[test]
fn test_get_favorite_commands() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let fav_id = test_db.create_test_command("Fav", "echo 1", Some(group_id));
    test_db.create_test_command("NotFav", "echo 2", Some(group_id));

    test_db.db.toggle_command_favorite(fav_id).unwrap();

    let favorites = test_db.db.get_commands(Some(group_id), None, true).unwrap();
    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, fav_id);
}

#[test]
fn test_search_commands() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Build Project", "cargo build", Some(group_id));
    test_db.create_test_command("Run Tests", "cargo test", Some(group_id));
    test_db.create_test_command("Delete All", "rm -rf", Some(group_id));

    let results = test_db.db.search_commands("cargo").unwrap();
    assert_eq!(results.len(), 2);
    assert!(results
        .iter()
        .all(|c| c.name.contains("cargo") || c.command.contains("cargo")));
}

#[test]
fn test_search_commands_case_insensitive() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Build", "CARGO build", Some(group_id));

    let results = test_db.db.search_commands("cargo").unwrap();
    assert_eq!(results.len(), 1); // SQLite LIKE is case-insensitive by default
}

#[test]
fn test_update_command() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Old Name", "old command", Some(group_id));

    let mut command = test_db.db.get_command(cmd_id).unwrap();
    command.name = "New Name".to_string();
    command.command = "new command".to_string();
    command.is_favorite = true;

    test_db.db.update_command(&command).unwrap();

    let updated = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.command, "new command");
    assert!(updated.is_favorite);
}

#[test]
fn test_update_command_not_found() {
    let test_db = TestDb::setup_test_db();
    let mut command = CommandBuilder::new("Ghost", "pwd").build();
    command.id = 99999;

    let result = test_db.db.update_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: "command",
            id: 99999
        })
    ));
}

#[test]
fn test_update_command_preserves_position() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");
    let cmd_id = test_db.create_test_command("Test", "echo", Some(group_id));

    let original_pos = test_db.db.get_command(cmd_id).unwrap().position;

    let mut cmd = test_db.db.get_command(cmd_id).unwrap();
    cmd.position = 9999;
    test_db.db.update_command(&cmd).unwrap();

    let updated = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(updated.position, original_pos); // Position unchanged
}

#[test]
fn test_update_command_validation() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Valid", "echo", Some(group_id));

    let mut command = test_db.db.get_command(cmd_id).unwrap();
    command.name = "".to_string();

    let result = test_db.db.update_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_toggle_favorite() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo", Some(group_id));

    let initial = test_db.db.get_command(cmd_id).unwrap().is_favorite;
    test_db.db.toggle_command_favorite(cmd_id).unwrap();

    assert_eq!(
        !initial,
        test_db.db.get_command(cmd_id).unwrap().is_favorite
    );
}

#[test]
fn test_toggle_favorite_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.toggle_command_favorite(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: "command",
            id: 99999
        })
    ));
}

#[test]
fn test_delete_command() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("To Delete", "echo", Some(group_id));

    test_db.db.delete_command(cmd_id).unwrap();

    let result = test_db.db.get_command(cmd_id);
    assert!(matches!(result, Err(DatabaseError::NotFound { .. })));
}

#[test]
fn test_delete_command_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.delete_command(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: "command",
            id: 99999
        })
    ));
}

#[test]
fn test_move_command_between() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));
    let id3 = test_db.create_test_command("C", "echo 3", Some(group_id));

    test_db
        .db
        .move_command_between(id3, Some(id1), Some(id2))
        .unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands[0].id, id1);
    assert_eq!(commands[1].id, id3);
    assert_eq!(commands[2].id, id2);
}

#[test]
fn test_move_command_to_top() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));

    test_db
        .db
        .move_command_between(id2, None, Some(id1))
        .unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands[0].id, id2); // B now first
    assert_eq!(commands[1].id, id1);
}

#[test]
fn test_move_command_to_bottom() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));

    // Move A to bottom (no next)
    test_db
        .db
        .move_command_between(id1, Some(id2), None)
        .unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands[0].id, id2);
    assert_eq!(commands[1].id, id1); // A now last
}

#[test]
fn test_position_gap_exhaustion_triggers_renumber() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    // Create commands with tight gaps
    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));

    // Manually set positions to simulate exhaustion
    test_db
        .db
        .conn()
        .execute(
            "UPDATE commands SET position = 1000 WHERE id = ?1",
            params![id1],
        )
        .unwrap();

    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));
    test_db
        .db
        .conn()
        .execute(
            "UPDATE commands SET position = 1001 WHERE id = ?1",
            params![id2],
        )
        .unwrap();

    // This move should trigger renumbering
    let id3 = test_db.create_test_command("C", "echo 3", Some(group_id));
    test_db
        .db
        .move_command_between(id3, Some(id1), Some(id2))
        .unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands.len(), 3);

    assert_eq!(commands[0].id, id1);
    assert_eq!(commands[1].id, id3); // C moved to middle
    assert_eq!(commands[2].id, id2);

    assert!(commands[1].position > commands[0].position);
    assert!(commands[1].position < commands[2].position);
}

#[test]
fn test_foreign_key_constraint_on_delete_group() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    test_db.create_test_command("Orphaned", "echo", Some(group_id));

    test_db.db.delete_group(group_id).unwrap();

    let commands = test_db
        .db
        .get_commands(Some(group_id), None, false)
        .unwrap();
    assert_eq!(commands.len(), 0);
}

#[test]
fn test_foreign_key_constraint_invalid_group_id() {
    let test_db = TestDb::setup_test_db();

    let command = CommandBuilder::new("test", "echo").with_group(9999).build();

    let result = test_db.db.create_command(&command);
    assert!(matches!(
        result,
        Err(DatabaseError::ForeignKeyViolation { .. })
    ));
}

#[test]
fn test_very_long_command_name() {
    let test_db = TestDb::setup_test_db();
    let long_name = "a".repeat(1000);

    let cmd_id = test_db.create_test_command(&long_name, "echo", None);
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.name.len(), 1000);
}

#[test]
fn test_special_characters_in_command() {
    let test_db = TestDb::setup_test_db();
    let special_cmd = "echo 'Hello $USER' && ls > /dev/null";

    let cmd_id = test_db.create_test_command("Special", special_cmd, None);
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.command, special_cmd);
}

#[test]
fn test_null_values_deserialize_properly() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Minimal", "echo", None);

    let retrieved = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(retrieved.description, None);
    assert_eq!(retrieved.env_vars, None);
    assert_eq!(retrieved.working_directory, None);
    assert_eq!(retrieved.arguments, Vec::<String>::new()); // Empty vec, not null
}

#[test]
fn test_updated_at_changes_on_update() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let cmd = test_db.db.get_command(cmd_id).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000)); // Ensure time difference

    let mut cmd1 = cmd.clone();
    cmd1.name = "Updated".to_string();
    test_db.db.update_command(&cmd1).unwrap();

    let retrieved_cmd = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(cmd1.created_at, retrieved_cmd.created_at);
    assert_ne!(cmd1.updated_at, retrieved_cmd.updated_at);
}
