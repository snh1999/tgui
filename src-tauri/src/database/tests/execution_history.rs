use super::*;
use crate::constants::{
    COMMANDS_TABLE, EXECUTION_HISTORY_TABLE, WORKFLOWS_TABLE, WORKFLOW_STEPS_TABLE,
};
use rusqlite::params;

#[test]
fn test_create_execution_history_builder_with_command() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let id = test_db.save_execution_history(&history);
    assert!(id > 0);

    let retrieved = test_db.db.get_execution_history(id).unwrap();
    assert_eq!(retrieved.command_id, Some(cmd_id));
    assert_eq!(retrieved.workflow_id, None);
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert_eq!(retrieved.triggered_by, TriggeredBy::Manual);
    assert_eq!(retrieved.context, history.context);
    assert!(retrieved.completed_at.is_none());
}

#[test]
fn test_create_execution_history_builder_with_workflow() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("test");

    let history = ExecutionHistoryBuilder::new()
        .with_workflow(flow_id)
        .build();
    let id = test_db.save_execution_history(&history);
    assert!(id > 0);

    let retrieved = test_db.db.get_execution_history(id).unwrap();
    assert_eq!(retrieved.command_id, None);
    assert_eq!(retrieved.workflow_id, Some(flow_id));
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert_eq!(retrieved.triggered_by, TriggeredBy::Manual);
    assert_eq!(retrieved.context, history.context);
    assert!(retrieved.completed_at.is_none());
}

#[test]
fn test_create_execution_history_builder_with_workflow_step() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let flow_id = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let history = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, flow_id, flow_step_id)
        .build();
    let id = test_db.save_execution_history(&history);
    assert!(id > 0);

    let retrieved = test_db.db.get_execution_history(id).unwrap();
    assert_eq!(retrieved.command_id, Some(cmd_id));
    assert_eq!(retrieved.workflow_id, Some(flow_id));
    assert_eq!(retrieved.workflow_step_id, Some(flow_step_id));
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert_eq!(retrieved.triggered_by, TriggeredBy::Workflow);
    assert_eq!(retrieved.context, history.context);
    assert!(retrieved.completed_at.is_none());
}

#[test]
fn test_create_execution_history_builder_with_workflow_step_invalid_workflow() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let flow_id = test_db.create_test_workflow("test");
    let flow_id_2 = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let history = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, flow_id_2, flow_step_id)
        .build();
    let result = test_db.db.create_execution_history(&history);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "workflow_step_id",
            ..
        })
    ));
}

#[test]
fn test_create_execution_history_rejects_when_command_already_running() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());

    let result = test_db.db.create_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "command", .. })
    ));
}

#[test]
fn test_create_execution_history_builder_with_workflow_step_invalid_command() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let cmd_id_2 = test_db.create_test_command("Test", "echo", None);

    let flow_id = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let history = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id_2, flow_id, flow_step_id)
        .build();
    let result = test_db.db.create_execution_history(&history);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData {
            field: "workflow_step_id",
            ..
        })
    ));
}

#[test]
fn test_create_execution_history_builder_with_invalid_combination() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let flow_id = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let history_without_step = ExecutionHistoryBuilder::new()
        .with_command(cmd_id)
        .with_workflow(flow_id)
        .build();
    let result = test_db.db.create_execution_history(&history_without_step);
    assert!(matches!(result, Err(DatabaseError::InvalidData { .. })));

    let mut history_without_command = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, flow_id, flow_step_id)
        .build();
    history_without_command.command_id = None;
    let result = test_db
        .db
        .create_execution_history(&history_without_command);
    assert!(matches!(result, Err(DatabaseError::InvalidData { .. })));

    let mut history_history_without_workflow = ExecutionHistoryBuilder::new()
        .with_workflow_step(cmd_id, flow_id, flow_step_id)
        .build();
    history_history_without_workflow.workflow_id = None;
    let result = test_db
        .db
        .create_execution_history(&history_history_without_workflow);
    assert!(matches!(result, Err(DatabaseError::InvalidData { .. })));
}

#[test]
fn test_create_execution_history_builder_with_invalid_id() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let flow_id = test_db.create_test_workflow("test");

    let result = test_db
        .db
        .create_execution_history(&ExecutionHistoryBuilder::new().with_command(999).build());
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: COMMANDS_TABLE,
            id: 999
        })
    ));

    let result = test_db
        .db
        .create_execution_history(&ExecutionHistoryBuilder::new().with_workflow(999).build());
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id: 999
        })
    ));

    let result = test_db.db.create_execution_history(
        &ExecutionHistoryBuilder::new()
            .with_workflow_step(cmd_id, flow_id, 999)
            .build(),
    );
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOW_STEPS_TABLE,
            id: 999
        })
    ));
}

#[test]
fn test_get_execution_history_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_execution_history(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: EXECUTION_HISTORY_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_get_command_execution_history_default_limit() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    for _ in 0..5 {
        let execution_id = test_db
            .save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
        std::thread::sleep(std::time::Duration::from_millis(50));
        test_db.db.update_execution_history_status(execution_id, ExecutionStatus::Failed, None).unwrap();
    }

    let history = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert_eq!(history.len(), 5);

    for i in 0..history.len() - 1 {
        assert!(history[i].started_at >= history[i + 1].started_at);
    }
}

#[test]
fn test_get_command_execution_history_with_limit() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    for _ in 0..5 {
        let execution_id = test_db
            .save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
        std::thread::sleep(std::time::Duration::from_millis(100));
        test_db.db.update_execution_history_status(execution_id, ExecutionStatus::Cancelled, None).unwrap();
    }

    let history = test_db
        .db
        .get_command_execution_history(cmd_id, Some(3))
        .unwrap();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_get_command_execution_history_empty() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert!(history.is_empty());
}

#[test]
fn test_get_workflow_execution_history() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let flow_id = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let mut history = ExecutionHistoryBuilder::new()
        .with_workflow(flow_id)
        .build();
    history.triggered_by = TriggeredBy::Workflow;
    test_db.save_execution_history(&history);

    history.triggered_by = TriggeredBy::Schedule;
    test_db.save_execution_history(&history);

    test_db.save_execution_history(
        &ExecutionHistoryBuilder::new()
            .with_workflow_step(cmd_id, flow_id, flow_step_id)
            .build(),
    );

    let history = test_db
        .db
        .get_workflow_execution_history(flow_id, None)
        .unwrap();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_get_workflow_execution_history_with_limit() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("test");

    for _ in 0..5 {
        test_db.save_execution_history(
            &ExecutionHistoryBuilder::new()
                .with_workflow(flow_id)
                .build(),
        );
    }

    let history = test_db
        .db
        .get_workflow_execution_history(flow_id, Some(2))
        .unwrap();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_get_running_commands_all() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id1 = test_db.save_execution_history(&history);
    test_db
        .db
        .update_execution_history_status(history_id1, ExecutionStatus::Success, Some(0))
        .unwrap();

    let history_id2 = test_db.save_execution_history(&history);
    test_db
        .db
        .update_execution_history_status(history_id2, ExecutionStatus::Failed, Some(100))
        .unwrap();

    let history_id3 = test_db.save_execution_history(&history);


    let running = test_db.db.get_running_commands().unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, history_id3   );
    assert_eq!(running[0].status, ExecutionStatus::Running);
}

#[test]
fn test_get_running_commands() {
    let test_db = TestDb::setup_test_db();
    let cmd_id1 = test_db.create_test_command("Test", "echo", None);
    let cmd_id2 = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id1).build();
    test_db.save_execution_history(&history);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id2).build();
    test_db.save_execution_history(&history);

    let running = test_db
        .db
        .get_running_commands()
        .unwrap();
    assert_eq!(running.len(), 2);
}

#[test]
fn test_get_running_commands_excludes_workflow_step_entries() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let flow_id = test_db.create_test_workflow("test");
    let step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    test_db.save_execution_history(
        &ExecutionHistoryBuilder::new()
            .with_workflow_step(cmd_id, flow_id, step_id)
            .build(),
    );

    let running = test_db.db.get_running_commands().unwrap();
    assert!(running.is_empty());
}

#[test]
fn test_get_running_commands_no_matches() {
    let test_db = TestDb::setup_test_db();
    let running = test_db.db.get_running_commands().unwrap();
    assert!(running.is_empty());
}

#[test]
fn test_update_execution_pid() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);

    test_db.db.update_execution_pid(history_id, 12345).unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.pid, Some(12345));
}

#[test]
fn test_update_execution_pid_rejects_non_running_status() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.update_execution_history_status(id, ExecutionStatus::Failed, Some(1)).unwrap();

    let result = test_db.db.update_execution_pid(id, 9999);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "pid", .. })
    ));
}

#[test]
fn test_update_execution_pid_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.update_execution_pid(999, 12345);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: EXECUTION_HISTORY_TABLE,
            id: 999
        })
    ));
}

#[test]
fn test_finish_execution_history_success() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);
    test_db.db.update_execution_pid(history_id, 12345).unwrap();

    test_db
        .db
        .update_execution_history_status(history_id, ExecutionStatus::Success, Some(0))
        .unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Success);
    assert_eq!(retrieved.pid, Some(12345));
    assert_eq!(retrieved.exit_code, Some(0));
    assert!(retrieved.completed_at.is_some());
}

#[test]
fn test_finish_execution_history_failed() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);

    test_db
        .db
        .update_execution_history_status(history_id, ExecutionStatus::Failed, Some(1))
        .unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert_eq!(retrieved.exit_code, Some(1));
}

#[test]
fn test_update_execution_history_status_rejects_running_to_running() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );

    let result = test_db.db.update_execution_history_status(id, ExecutionStatus::Running, None);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "status", .. })
    ));
}

#[test]
fn test_update_execution_history_status_rejects_already_terminal() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();

    let result = test_db.db.update_execution_history_status(id, ExecutionStatus::Failed, Some(1));
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "status", .. })
    ));
}

#[test]
fn test_kill_failed_execution_history() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);
    test_db.db.kill_failed_execution(history_id).unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert_eq!(retrieved.exit_code, None);
}

#[test]
fn test_delete_execution_history() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);

    test_db.db.delete_execution_history(history_id).unwrap();

    let result = test_db.db.get_execution_history(history_id);
    assert!(matches!(result, Err(DatabaseError::NotFound { .. })));
}

#[test]
fn test_delete_execution_history_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.delete_execution_history(99999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: EXECUTION_HISTORY_TABLE,
            id: 99999
        })
    ));
}

#[test]
fn test_cleanup_command_history_keep_last() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();

    let mut ids = vec![];
    for _ in 0..5 {
        let id = test_db.save_execution_history(&history);
        test_db
            .db
            .update_execution_history_status(id, ExecutionStatus::Success, Some(0))
            .unwrap();
        ids.push(id);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    test_db.db.cleanup_command_history(cmd_id, 2).unwrap();

    let remaining = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert_eq!(remaining.len(), 2);
    assert!(ids.contains(&remaining[0].id));
    assert!(ids.contains(&remaining[1].id));
}

#[test]
fn test_cleanup_command_history_keep_all() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();

    for _ in 0..3 {
        let id = test_db.save_execution_history(&history);
        std::thread::sleep(std::time::Duration::from_millis(10));
        test_db
            .db
            .update_execution_history_status(id, ExecutionStatus::Success, Some(0))
            .unwrap();
    }

    test_db.db.cleanup_command_history(cmd_id, 10).unwrap();

    let remaining = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn test_cleanup_command_history_zero_keep_last_deletes_all() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    for _ in 0..3 {
        let id = test_db.save_execution_history(
            &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
        );
        test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    }

    test_db.db.cleanup_command_history(cmd_id, 0).unwrap();

    let remaining = test_db.db.get_command_execution_history(cmd_id, None).unwrap();
    assert!(remaining.is_empty());
}

#[test]
fn test_cleanup_history_older_than_zero_days_deletes_all_non_running() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    // Non-running, old
    let finished = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.update_execution_history_status(finished, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-1 days') WHERE id = ?1",
        params![finished],
    ).unwrap();

    // Running, old (should be preserved)
    let running = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-1 days') WHERE id = ?1",
        params![running],
    ).unwrap();

    test_db.db.cleanup_history_older_than(0).unwrap();

    assert!(test_db.db.get_execution_history(finished).is_err());
    assert!(test_db.db.get_execution_history(running).is_ok());
}

#[test]
fn test_cleanup_history_older_than_preserves_running_entries() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let running_id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-10 days') WHERE id = ?1",
        params![running_id],
    ).unwrap();

    test_db.db.cleanup_history_older_than(7).unwrap();

    assert!(test_db.db.get_execution_history(running_id).is_ok());
}

#[test]
fn test_cleanup_history_older_than() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let cmd_id_2 = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();

    for _ in 0..2{
        let id = test_db.save_execution_history(&history);
        test_db
            .db
            .update_execution_history_status(id, ExecutionStatus::Success, Some(0))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    // this is not deleted
    test_db.save_execution_history(&history);

    // manually set started_at to 10 days ago
    test_db
        .db
        .conn()
        .unwrap()
        .execute(
            "UPDATE execution_history SET started_at = datetime('now', '-10 days')",
            params![],
        )
        .unwrap();

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id_2).build();
    for _ in 0..3{
        let id = test_db.save_execution_history(&history);
        test_db
            .db
            .update_execution_history_status(id, ExecutionStatus::Success, Some(0))
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    test_db.db.cleanup_history_older_than(7).unwrap();

    let remaining = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert_eq!(remaining.len(), 1);

    let remaining = test_db
        .db
        .get_command_execution_history(cmd_id_2, None)
        .unwrap();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn test_cleanup_history_older_than_no_matches() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.cleanup_history_older_than(30);
    assert!(result.is_ok());
}

#[test]
fn test_cleanup_command_history_preserves_workflow_associated_entries() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let flow_id = test_db.create_test_workflow("test");
    let step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let workflow_h = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new()
            .with_workflow_step(cmd_id, flow_id, step_id)
            .build(),
    );
    test_db.db.update_execution_history_status(workflow_h, ExecutionStatus::Success, Some(0)).unwrap();

    for _ in 0..3 {
        let id = test_db.save_execution_history(
            &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
        );
        test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    test_db.db.cleanup_command_history(cmd_id, 1).unwrap();

    assert!(test_db.db.get_execution_history(workflow_h).is_ok());
}

#[test]
fn test_execution_history_spawn_failure_cancels_row() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("test", "echo test", None);

    let exec_id = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id,
            TriggeredBy::Manual,
        ))
        .unwrap();

    // spawn failed — kill immediately, no PID ever stored
    test_db.db.kill_failed_execution(exec_id).unwrap();

    let row = test_db.db.get_execution_history(exec_id).unwrap();
    assert_eq!(row.status, ExecutionStatus::Failed);
    assert_eq!(row.pid, None);
    assert!(row.completed_at.is_some());
}

#[test]
fn test_cascade_delete_command_removes_all_history() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Temporary", "echo test", None);
    let other_cmd_id = test_db.create_test_command("Permanent", "echo test", None);

    let exec_ids: Vec<i64> = (0..3)
        .map(|_| {
            let id = test_db
                .db
                .create_execution_history(&ExecutionHistory::new_with_command(
                    cmd_id,
                    TriggeredBy::Manual,
                ))
                .unwrap();
            test_db
                .db
                .update_execution_history_status(id, ExecutionStatus::Success, Some(0))
                .unwrap();
            id
        })
        .collect();

    let survivor_id = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            other_cmd_id,
            TriggeredBy::Manual,
        ))
        .unwrap();

    test_db.db.delete_command(cmd_id).unwrap();

    for exec_id in exec_ids {
        assert!(matches!(
            test_db.db.get_execution_history(exec_id),
            Err(DatabaseError::NotFound { .. })
        ));
    }

    assert!(test_db.db.get_execution_history(survivor_id).is_ok());
}

#[test]
fn test_cascade_delete_workflow_removes_workflow_history() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("Ephemeral");

    let exec_id = test_db
        .db
        .create_execution_history(
            &ExecutionHistoryBuilder::new()
                .with_workflow(flow_id)
                .build(),
        )
        .unwrap();

    test_db.db.delete_workflow(flow_id).unwrap();

    assert!(matches!(
        test_db.db.get_execution_history(exec_id),
        Err(DatabaseError::NotFound { .. })
    ));
}


#[test]
fn test_get_execution_stats_command_success_rate() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    // 2 success, 1 failed, 1 cancelled, 1 running = 5 total
    let success1 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(success1, ExecutionStatus::Success, Some(0)).unwrap();

    let success2 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(success2, ExecutionStatus::Success, Some(0)).unwrap();

    let failed = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(failed, ExecutionStatus::Failed, Some(1)).unwrap();

    let cancelled = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(cancelled, ExecutionStatus::Cancelled, None).unwrap();

    // Leave one as running
    test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());

    let stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();

    assert_eq!(stats.total_count, 5);
    assert_eq!(stats.success_count, 2);
    assert_eq!(stats.failed_count, 1);
    assert_eq!(stats.cancelled_count, 1);
    assert_eq!(stats.running_count, 1);
    assert_eq!(stats.success_rate, 0.4); // 2/5
    assert!(stats.average_duration_ms.is_some());
    assert!(stats.last_executed_at.is_some());
    assert!(stats.first_executed_at.is_some());
}

#[test]
fn test_get_execution_stats_workflow_excludes_steps() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Step", "echo", None);
    let flow_id = test_db.create_test_workflow("TestFlow");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    // Workflow-level execution (standalone)
    let flow_exec = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_workflow(flow_id).build()
    );
    test_db.db.update_execution_history_status(flow_exec, ExecutionStatus::Success, Some(0)).unwrap();

    // Workflow step execution (should NOT be counted in workflow stats)
    let step_exec = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_workflow_step(cmd_id, flow_id, flow_step_id).build()
    );
    test_db.db.update_execution_history_status(step_exec, ExecutionStatus::Failed, Some(1)).unwrap();

    let stats = test_db.db.get_execution_stats(StatsTarget::Workflow(flow_id), None).unwrap();

    // Should only count the standalone workflow execution, not the step
    assert_eq!(stats.total_count, 1);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failed_count, 0);
    assert_eq!(stats.success_rate, 1.0);
}

#[test]
fn test_get_execution_stats_global_excludes_steps() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Cmd", "echo", None);
    let flow_id = test_db.create_test_workflow("Flow");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    // Standalone command
    let cmd_exec = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(cmd_exec, ExecutionStatus::Success, Some(0)).unwrap();

    // Standalone workflow
    let flow_exec = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_workflow(flow_id).build());
    test_db.db.update_execution_history_status(flow_exec, ExecutionStatus::Failed, Some(1)).unwrap();

    // Workflow step (should be excluded from global stats)
    test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_workflow_step(cmd_id, flow_id, flow_step_id).build()
    );

    let stats = test_db.db.get_execution_stats(StatsTarget::Global, None).unwrap();

    assert_eq!(stats.total_count, 2);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failed_count, 1);
    assert_eq!(stats.success_rate, 0.5);
}

#[test]
fn test_get_execution_stats_empty() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();

    assert_eq!(stats.total_count, 0);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failed_count, 0);
    assert_eq!(stats.success_rate, 0.0);
    assert!(stats.average_duration_ms.is_none());
    assert!(stats.last_executed_at.is_none());
    assert!(stats.first_executed_at.is_none());
}

#[test]
fn test_get_execution_stats_duration_calculation() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "sleep", None);

    let exec1 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_pid(exec1, 1000).unwrap();
    test_db.db.update_execution_history_status(exec1, ExecutionStatus::Success, Some(0)).unwrap();

    let exec2 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_pid(exec2, 1001).unwrap();
    test_db.db.update_execution_history_status(exec2, ExecutionStatus::Failed, Some(1)).unwrap();

    let stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();

    assert!(stats.average_duration_ms.is_some());
}

#[test]
fn test_update_execution_pid_zero_is_valid() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history_id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );

    // PID 0 is technically valid (though unusual)
    test_db.db.update_execution_pid(history_id, 0).unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.pid, Some(0));
}

#[test]
fn test_update_execution_pid_large_value() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history_id = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );

    // Max u32 value
    let large_pid = u32::MAX;
    test_db.db.update_execution_pid(history_id, large_pid).unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.pid, Some(large_pid as i64));
}

#[test]
fn test_get_execution_stats_counts_all_status_types() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let statuses = [
        (ExecutionStatus::Success, Some(0)),
        (ExecutionStatus::Failed, Some(1)),
        (ExecutionStatus::Cancelled, None),
        (ExecutionStatus::TimeOut, None),
        (ExecutionStatus::Paused, None),
        (ExecutionStatus::Skipped, None),
        (ExecutionStatus::Running, None),
    ];

    for (status, exit_code) in &statuses {
        let exec_id = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
        if *status != ExecutionStatus::Running {
            test_db.db.update_execution_history_status(exec_id, status.clone(), *exit_code).unwrap();
        }
    }

    let stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();

    assert_eq!(stats.total_count, 7);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failed_count, 1);
    assert_eq!(stats.cancelled_count, 1);
    assert_eq!(stats.timeout_count, 1);
    assert_eq!(stats.running_count, 1);
    assert_eq!(stats.paused_count, 1);
    assert_eq!(stats.skipped_count, 1);
}

#[test]
fn test_get_execution_stats_timestamps_ordered() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    // Create with delays to ensure ordering
    let exec1 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(exec1, ExecutionStatus::Success, Some(0)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    let exec2 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(exec2, ExecutionStatus::Success, Some(0)).unwrap();

    let stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();

    assert!(stats.first_executed_at.is_some());
    assert!(stats.last_executed_at.is_some());
    assert!(stats.first_executed_at <= stats.last_executed_at);
}

#[test]
fn test_get_execution_stats_global_with_time_filter() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Cmd", "echo", None);

    // Old execution
    let old = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(old, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-10 days') WHERE id = ?1",
        params![old],
    ).unwrap();

    // Recent execution
    let recent = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(recent, ExecutionStatus::Failed, Some(1)).unwrap();

    let all = test_db.db.get_execution_stats(StatsTarget::Global, None).unwrap();
    assert_eq!(all.total_count, 2);

    let last_7 = test_db.db.get_execution_stats(StatsTarget::Global, Some(7)).unwrap();
    assert_eq!(last_7.total_count, 1);
    assert_eq!(last_7.failed_count, 1);
}


#[test]
fn test_get_execution_stats_with_time_filter() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    // Create old execution (30 days ago)
    let old_exec = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(old_exec, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-30 days'), completed_at = datetime('now', '-30 days') WHERE id = ?1",
        params![old_exec],
    ).unwrap();

    // Create recent execution (just now)
    let recent_exec = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_history_status(recent_exec, ExecutionStatus::Failed, Some(1)).unwrap();

    // All time stats - should see both
    let all_time = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();
    assert_eq!(all_time.total_count, 2);

    // Last 7 days - should only see recent
    let last_week = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), Some(7)).unwrap();
    assert_eq!(last_week.total_count, 1);
    assert_eq!(last_week.failed_count, 1);

    // Last 1 day - should see recent (created within last few seconds)
    let last_day = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), Some(1)).unwrap();
    assert_eq!(last_day.total_count, 1);
}

#[test]
fn test_get_execution_stats_time_filter_averages_and_timestamps() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    // Create execution 30 days ago
    let exec1 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_pid(exec1, 1000).unwrap();
    test_db.db.update_execution_history_status(exec1, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-30 days'), completed_at = datetime('now', '-30 days', '+5 seconds') WHERE id = ?1",
        params![exec1],
    ).unwrap();

    // Create execution now
    let exec2 = test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
    test_db.db.update_execution_pid(exec2, 1001).unwrap();
    test_db.db.update_execution_history_status(exec2, ExecutionStatus::Success, Some(0)).unwrap();

    // All time - 2 records
    let all_stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), None).unwrap();
    assert_eq!(all_stats.total_count, 2);

    // Last 7 days - 1 record
    let recent_stats = test_db.db.get_execution_stats(StatsTarget::Command(cmd_id), Some(7)).unwrap();
    assert_eq!(recent_stats.total_count, 1);
    assert!(recent_stats.first_executed_at.is_some());
    assert_eq!(recent_stats.first_executed_at, recent_stats.last_executed_at); // Only one record
}


#[test]
fn test_get_latest_execution_for_command_invalid(){
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.get_latest_execution_for_command(999);
    assert!(result.is_none());
}

#[test]
fn test_get_latest_execution_for_command_empty(){
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let result = test_db.db.get_latest_execution_for_command(cmd_id);
    assert!(result.is_none());
}

#[test]
fn test_get_latest_execution_for_command_returns_most_recent() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let first = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );
    test_db.db.update_execution_history_status(first, ExecutionStatus::Success, Some(0)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));

    let second = test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_command(cmd_id).build()
    );

    let result = test_db.db.get_latest_execution_for_command(cmd_id).unwrap();
    assert_eq!(result.id, second);
    assert_eq!(result.status, ExecutionStatus::Running);
}
