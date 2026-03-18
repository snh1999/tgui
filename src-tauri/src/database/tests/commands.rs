use super::*;
use crate::constants::COMMANDS_TABLE;
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
            entity: COMMANDS_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_commands_count_empty_db() {
    let test_db = TestDb::setup_test_db();

    let count = test_db.db.get_commands_count(None, None, false).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_get_commands_count_with_group_filter() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.create_test_command("Cmd3", "echo 3", None);

    let count_group = test_db
        .db
        .get_commands_count(Some(group_id), None, false)
        .unwrap();
    let count_root = test_db.db.get_commands_count(None, None, false).unwrap();

    assert_eq!(count_group, 2);
    assert_eq!(count_root, 1);
}

#[test]
fn test_get_commands_count_with_category_filter() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.save_command_to_db(
        &CommandBuilder::new("Cmd2", "echo 2")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    let count_no_category = test_db
        .db
        .get_commands_count(Some(group_id), None, false)
        .unwrap();
    let count_with_category = test_db
        .db
        .get_commands_count(Some(group_id), Some(category_id), false)
        .unwrap();

    assert_eq!(count_no_category, 2);
    assert_eq!(count_with_category, 1);
}

#[test]
fn test_get_commands_count_favorites_only() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd1 = test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    let cmd2 = test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    test_db.create_test_command("Cmd3", "echo 3", Some(group_id));

    test_db.db.toggle_command_favorite(cmd1).unwrap();
    test_db.db.toggle_command_favorite(cmd2).unwrap();

    let count_all = test_db
        .db
        .get_commands_count(Some(group_id), None, false)
        .unwrap();
    let count_favorites = test_db
        .db
        .get_commands_count(Some(group_id), None, true)
        .unwrap();

    assert_eq!(count_all, 3);
    assert_eq!(count_favorites, 2);
}

#[test]
fn test_get_commands_count_combined_filters() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    let cmd1 = test_db.create_test_command("Cmd1", "echo 1", Some(group_id));
    test_db.create_test_command("Cmd2", "echo 2", Some(group_id));
    let cmd3 = test_db.save_command_to_db(
        &CommandBuilder::new("Cmd3", "echo 3")
            .with_group(group_id)
            .with_category(category_id)
            .build(),
    );

    test_db.db.toggle_command_favorite(cmd1).unwrap();
    test_db.db.toggle_command_favorite(cmd3).unwrap();

    let count = test_db
        .db
        .get_commands_count(Some(group_id), Some(category_id), true)
        .unwrap();
    assert_eq!(count, 1);
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
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

    let commands = test_db
        .db
        .get_commands(GroupFilter::None, CategoryFilter::None, false, None, None)
        .unwrap();
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    assert_eq!(commands.len(), 2);
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::Category(category_id), false, None, None)
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::Category(category_id), false, None, None)
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

    let favorites = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, true, None, None)
        .unwrap();
    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, fav_id);
}

#[test]
fn test_get_commands_pagination_limit() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    for i in 0..5 {
        test_db.create_test_command(&format!("Cmd{}", i), &format!("echo {}", i), Some(group_id));
    }

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, Some(3), None)
        .unwrap();
    assert_eq!(commands.len(), 3);
}

#[test]
fn test_get_commands_pagination_offset() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let mut ids = vec![];
    for i in 0..5 {
        ids.push(test_db.create_test_command(
            &format!("Cmd{}", i),
            &format!("echo {}", i),
            Some(group_id),
        ));
    }

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, Some(2))
        .unwrap();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].id, ids[2]);
}

#[test]
fn test_get_commands_pagination_limit_and_offset() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let mut ids = vec![];
    for i in 0..5 {
        ids.push(test_db.create_test_command(
            &format!("Cmd{}", i),
            &format!("echo {}", i),
            Some(group_id),
        ));
    }

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, Some(2), Some(1))
        .unwrap();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].id, ids[1]);
    assert_eq!(commands[1].id, ids[2]);
}

#[test]
fn test_get_commands_pagination_offset_beyond_total() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    test_db.create_test_command("Cmd1", "echo 1", Some(group_id));

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, Some(100))
        .unwrap();
    assert!(commands.is_empty());
}

#[test]
fn test_get_commands_with_history_joined() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let history = ExecutionHistoryBuilder::new()
        .with_command(cmd_id)
        .with_trigger(TriggeredBy::Manual)
        .build();

    let history_id = test_db.db.create_execution_history(&history).unwrap();
    test_db.db.update_execution_pid(history_id, 1234).unwrap();

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    assert_eq!(commands.len(), 1);

    let with_history = &commands[0];
    assert_eq!(with_history.id, cmd_id);
    assert!(with_history.history.is_some());

    let retrieved_history = with_history.history.as_ref().unwrap();
    assert_eq!(retrieved_history.id, history_id);
    assert_eq!(retrieved_history.pid, Some(1234));
    assert_eq!(retrieved_history.status, ExecutionStatus::Running);
}

#[test]
fn test_get_commands_with_history_status_updated() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let history = ExecutionHistoryBuilder::new()
        .with_command(cmd_id)
        .with_trigger(TriggeredBy::Manual)
        .build();

    let history_id = test_db.db.create_execution_history(&history).unwrap();
    test_db
        .db
        .update_execution_history_status(history_id, ExecutionStatus::Failed, Some(0))
        .unwrap();

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    assert_eq!(commands.len(), 1);

    let with_history = &commands[0];
    assert_eq!(with_history.id, cmd_id);
    assert!(with_history.history.is_some());

    let retrieved_history = with_history.history.as_ref().unwrap();
    assert_eq!(retrieved_history.id, history_id);
    assert_eq!(retrieved_history.status, ExecutionStatus::Failed);
    assert_eq!(retrieved_history.exit_code, Some(0));
}

#[test]
fn test_get_commands_history_most_recent_only() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let history = ExecutionHistoryBuilder::new()
        .with_command(cmd_id)
        .with_trigger(TriggeredBy::Manual)
        .build();

    let history_id = test_db.db.create_execution_history(&history).unwrap();
    test_db
        .db
        .update_execution_history_status(history_id, ExecutionStatus::Failed, Some(0))
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));

    let history_id2 = test_db.db.create_execution_history(&history).unwrap();
    test_db.db.update_execution_pid(history_id2, 1234).unwrap();

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    let retrieved_history = commands[0].history.as_ref().unwrap();
    assert_eq!(retrieved_history.id, history_id2);
    assert_eq!(retrieved_history.pid, Some(1234));
    assert_eq!(retrieved_history.status, ExecutionStatus::Running);
}

#[test]
fn test_get_commands_excludes_workflow_history() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));
    let workflow_id = test_db.create_test_workflow("Test Workflow");
    let workflow_step_id = test_db.create_test_workflow_step(workflow_id, cmd_id);

    let workflow_history = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, workflow_id, workflow_step_id)
        .build();

    test_db
        .db
        .create_execution_history(&workflow_history)
        .unwrap();

    let commands = test_db
        .db
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    assert!(commands[0].history.is_none());
}

#[test]
fn test_get_recent_commands_basic() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    // Create command with history
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));
    test_db
        .db
        .create_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build())
        .unwrap();

    let recent = test_db.db.get_recent_commands(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].item.id, cmd_id);
    assert!(recent[0].history.is_some());
}

#[test]
fn test_get_recent_commands_empty_when_no_history() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    // Create command WITHOUT history
    test_db.create_test_command("Test", "echo test", Some(group_id));

    let recent = test_db.db.get_recent_commands(10).unwrap();
    assert!(recent.is_empty());
}

#[test]
fn test_get_recent_commands_limit() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    for i in 0..5 {
        let cmd_id = test_db.create_test_command(&format!("Cmd{}", i), "echo test", Some(group_id));
        test_db
            .db
            .create_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build())
            .unwrap();
    }

    let recent = test_db.db.get_recent_commands(3).unwrap();
    assert_eq!(recent.len(), 3);
}

#[test]
fn test_get_recent_commands_ordering_by_started_at() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd1 = test_db.create_test_command("Old", "echo old", Some(group_id));
    let cmd2 = test_db.create_test_command("New", "echo new", Some(group_id));
    let cmd3 = test_db.create_test_command("Middle", "echo middle", Some(group_id));
    let cmd4 = test_db.create_test_command("Middle", "echo middle", Some(group_id));

    for cmd_id in [cmd4, cmd1, cmd3, cmd2] {
        test_db
            .db
            .create_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build())
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    let recent = test_db.db.get_recent_commands(3).unwrap();
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].item.id, cmd2);
    assert_eq!(recent[1].item.id, cmd3);
    assert_eq!(recent[2].item.id, cmd1);
}

#[test]
fn test_get_recent_commands_multiple_history_per_command() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let mut execution_id = 0;

    for _ in 0..3 {
        execution_id = test_db
            .db
            .create_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build())
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    let recent = test_db.db.get_recent_commands(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].history.as_ref().unwrap().id, execution_id);
}

#[test]
fn test_get_recent_commands_includes_workflow_history() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));
    let workflow_id = test_db.create_test_workflow("Test Workflow");
    let flow_step_id = test_db.create_test_workflow_step(workflow_id, cmd_id);

    let history = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, workflow_id, flow_step_id)
        .build();
    test_db.db.create_execution_history(&history).unwrap();

    let recent = test_db.db.get_recent_commands(10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(
        recent[0].history.as_ref().unwrap().triggered_by,
        TriggeredBy::Workflow
    );
}

#[test]
fn test_get_recent_commands_zero_limit() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    test_db.db.create_execution_history(&history).unwrap();

    let recent = test_db.db.get_recent_commands(0).unwrap();
    assert!(recent.is_empty());
}

#[test]
fn test_get_recent_commands_preserves_command_fields() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let category_id = test_db.create_test_category("Test Category");

    let cmd = CommandBuilder::new("Full", "echo full")
        .with_group(group_id)
        .with_category(category_id)
        .with_env("KEY", "value")
        .build();
    let cmd_id = test_db.save_command_to_db(&cmd);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    test_db.db.create_execution_history(&history).unwrap();

    let recent = test_db.db.get_recent_commands(10).unwrap();
    assert_eq!(recent.len(), 1);

    let retrieved = &recent[0].item;
    assert_eq!(retrieved.name, "Full");
    assert_eq!(retrieved.command, "echo full");
    assert_eq!(retrieved.group_id, Some(group_id));
    assert_eq!(retrieved.category_id, Some(category_id));
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
            entity: COMMANDS_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_update_command_preserves_position() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test");
    let cmd_id = test_db.create_test_command("Test", "echo", Some(group_id));

    let mut cmd = test_db.db.get_command(cmd_id).unwrap();
    let original_pos = cmd.position;

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
    let cmd_id = test_db.create_test_command("Test", "echo", None);

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
            entity: COMMANDS_TABLE,
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
            entity: COMMANDS_TABLE,
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
        .unwrap();
    assert_eq!(commands[0].id, id2);
    assert_eq!(commands[1].id, id1); // A now last
}

#[test]
fn test_move_command_invalid_prev_id() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));

    let result = test_db.db.move_command_between(id2, Some(99999), Some(id1));
    assert!(result.is_err());
}

#[test]
fn test_move_command_invalid_next_id() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));

    let result = test_db.db.move_command_between(id1, Some(id2), Some(99999));
    assert!(result.is_err());
}

#[test]
fn test_move_command_prev_next_different_groups() {
    let test_db = TestDb::setup_test_db();
    let group1 = test_db.create_test_group("Group 1");
    let group2 = test_db.create_test_group("Group 2");

    let id1 = test_db.create_test_command("A", "echo 1", Some(group1));
    let id2 = test_db.create_test_command("B", "echo 2", Some(group2));
    let id3 = test_db.create_test_command("C", "echo 3", Some(group1));

    let result = test_db.db.move_command_between(id3, Some(id1), Some(id2));
    assert!(result.is_err());
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
        .unwrap()
        .execute(
            "UPDATE commands SET position = 1000 WHERE id = ?1",
            params![id1],
        )
        .unwrap();

    let id2 = test_db.create_test_command("B", "echo 2", Some(group_id));
    test_db
        .db
        .conn()
        .unwrap()
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
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
        .get_commands(GroupFilter::Group(group_id), CategoryFilter::None, false, None, None)
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

    let result = test_db.db.create_command(&CommandBuilder::new(&long_name, "echo").build());

    assert!(result.is_err());

    let long_name = "a".repeat(254);
    let cmd_id = test_db.create_test_command(&long_name, "echo", None);
    let retrieved = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(retrieved.name.len(), 254);

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

#[test]
fn test_create_command_empty_env_vars_hashmap() {
    let test_db = TestDb::setup_test_db();

    let empty_env: HashMap<String, String> = HashMap::new();
    let mut cmd = CommandBuilder::new("Test", "echo test").build();
    cmd.env_vars = Some(empty_env);

    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert!(retrieved.env_vars.is_some());
    assert!(retrieved.env_vars.as_ref().unwrap().is_empty());
}

#[test]
fn test_create_command_env_vars_empty_values() {
    let test_db = TestDb::setup_test_db();

    let cmd = CommandBuilder::new("Test", "echo test")
        .with_env("EMPTY_VALUE", "")
        .build();

    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(
        retrieved.env_vars.as_ref().unwrap().get("EMPTY_VALUE"),
        Some(&"".to_string())
    );
}

#[test]
fn test_create_command_env_vars_special_characters_in_values() {
    let test_db = TestDb::setup_test_db();

    let cmd = CommandBuilder::new("Test", "echo test")
        .with_env("SPECIAL", "value with spaces and !@#$%^&*()")
        .with_env("PATH_VAR", "/usr/local/bin:/usr/bin")
        .with_env("JSON", r#"{"key": "value"}"#)
        .build();

    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    let env = retrieved.env_vars.unwrap();
    assert_eq!(
        env.get("SPECIAL"),
        Some(&"value with spaces and !@#$%^&*()".to_string())
    );
    assert_eq!(
        env.get("PATH_VAR"),
        Some(&"/usr/local/bin:/usr/bin".to_string())
    );
    assert_eq!(env.get("JSON"), Some(&r#"{"key": "value"}"#.to_string()));
}

#[test]
fn test_create_command_env_var_key_unicode_rejected() {
    let test_db = TestDb::setup_test_db();

    let cmd = CommandBuilder::new("Test", "echo test")
        .with_env("KEY_🔥", "value")
        .build();

    let result = test_db.db.create_command(&cmd);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "env_vars",
            ..
        })
    ));
}

#[test]
fn test_create_command_arguments_special_characters() {
    let test_db = TestDb::setup_test_db();

    let cmd = CommandBuilder::new("Test", "echo")
        .with_args(vec![
            "arg with spaces",
            "--flag=value",
            "$(echo injection)",
            "; rm -rf /",
            "| cat /etc/passwd",
            "`whoami`",
            "$VAR",
        ])
        .build();

    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.arguments.len(), 7);
    assert_eq!(retrieved.arguments[0], "arg with spaces");
    assert_eq!(retrieved.arguments[4], "| cat /etc/passwd");
}

#[test]
fn test_create_command_very_long_arguments() {
    let test_db = TestDb::setup_test_db();

    let long_arg = "x".repeat(10000);
    let cmd = CommandBuilder::new("Test", "echo")
        .with_args(vec![&long_arg])
        .build();

    let cmd_id = test_db.db.create_command(&cmd).unwrap();
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.arguments[0].len(), 10000);
}

#[test]
fn test_create_command_name_whitespace_only() {
    let test_db = TestDb::setup_test_db();

    let cmd = CommandBuilder::new("\t\n  ", "echo test").build();
    let result = test_db.db.create_command(&cmd);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_initial_position_assignment() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));
    let retrieved = test_db.db.get_command(cmd_id).unwrap();

    assert_eq!(retrieved.position, Database::POSITION_GAP);
}

#[test]
fn test_consecutive_positions_gap() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd1 = test_db.create_test_command("A", "echo 1", Some(group_id));
    let cmd2 = test_db.create_test_command("B", "echo 2", Some(group_id));
    let cmd3 = test_db.create_test_command("B", "echo 2", Some(group_id));

    let pos1 = test_db.db.get_command(cmd1).unwrap().position;
    let pos2 = test_db.db.get_command(cmd2).unwrap().position;
    let pos3 = test_db.db.get_command(cmd3).unwrap().position;

    assert_eq!(pos3 - pos2, Database::POSITION_GAP + 1);
    assert_eq!(pos2 - pos1, Database::POSITION_GAP + 1);
}

#[test]
fn test_row_to_command_invalid_arguments_json() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");

    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let result = test_db.db.conn().unwrap().execute(
        "UPDATE commands SET arguments = 'invalid json' WHERE id = ?1",
        params![cmd_id],
    );

    assert!(result.is_err());
}

#[test]
fn test_row_to_execution_history_unknown_status() {
    let test_db = TestDb::setup_test_db();
    let group_id = test_db.create_test_group("Test Group");
    let cmd_id = test_db.create_test_command("Test", "echo test", Some(group_id));

    let result = test_db.db.conn()
        .unwrap()
        .execute(
            "INSERT INTO execution_history (command_id, status, triggered_by) VALUES (?1, 'UnknownStatus', 'Manual')",
            params![cmd_id],
        );

    assert!(result.is_err());
}
