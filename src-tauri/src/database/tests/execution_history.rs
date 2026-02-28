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
    assert!(matches!(result, Err(DatabaseError::InvalidData { field: "workflow_step_id", ..})));
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
    assert!(matches!(result, Err(DatabaseError::InvalidData { field: "workflow_step_id", ..})));

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
        test_db
            .save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
        std::thread::sleep(std::time::Duration::from_millis(50));
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
        test_db
            .save_execution_history(&ExecutionHistoryBuilder::new().with_command(cmd_id).build());
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

    let history = test_db.db.get_command_execution_history(cmd_id, None).unwrap();
    assert!(history.is_empty());
}

#[test]
fn test_get_workflow_execution_history() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let flow_id = test_db.create_test_workflow("test");
    let flow_step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    let mut history = ExecutionHistoryBuilder::new().with_workflow(flow_id).build();
    history.triggered_by = TriggeredBy::Workflow;
    test_db.save_execution_history(&history);

    history.triggered_by = TriggeredBy::Schedule;
    test_db.save_execution_history(&history);

   test_db.save_execution_history(
        &ExecutionHistoryBuilder::new().with_workflow_step(cmd_id, flow_id, flow_step_id).build()
    );

    let history = test_db.db.get_workflow_execution_history(flow_id, None).unwrap();
    assert_eq!(history.len(), 3);
}

#[test]
fn test_get_workflow_execution_history_with_limit() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("test");

    for _ in 0..5 {
        test_db.save_execution_history(&ExecutionHistoryBuilder::new().with_workflow(flow_id).build());
    }

    let history = test_db.db.get_workflow_execution_history(flow_id, Some(2)).unwrap();
    assert_eq!(history.len(), 2);
}


#[test]
fn test_get_running_commands_all() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id1 = test_db.save_execution_history(&history);
    let history_id2 = test_db.save_execution_history(&history);
    let history_id3 = test_db.save_execution_history(&history);

    test_db.db.update_execution_history_status(history_id2, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.db.update_execution_history_status(history_id3, ExecutionStatus::Failed, Some(100)).unwrap();

    let running = test_db.db.get_running_commands(None, None).unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, history_id1);
    assert_eq!(running[0].status, ExecutionStatus::Running);
}

#[test]
fn test_get_running_commands_by_command_id() {
    let test_db = TestDb::setup_test_db();
    let cmd_id1 = test_db.create_test_command("Test", "echo", None);
    let cmd_id2 = test_db.create_test_command("Test", "echo", None);


    let history = ExecutionHistoryBuilder::new().with_command(cmd_id1).build();
    test_db.save_execution_history(&history);
    test_db.save_execution_history(&history);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id2).build();
    test_db.save_execution_history(&history);

    let running = test_db.db.get_running_commands(Some(cmd_id1), None).unwrap();
    assert_eq!(running.len(), 2);
}

#[test]
fn test_get_running_commands_by_workflow_id() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("test");
    let flow_id_2 = test_db.create_test_workflow("test");

    let history = ExecutionHistoryBuilder::new().with_workflow(flow_id).build();
    test_db.save_execution_history(&history);
    test_db.save_execution_history(&history);
    let history = ExecutionHistoryBuilder::new().with_workflow(flow_id_2).build();
    test_db.save_execution_history(&history);
    test_db.save_execution_history(&history);

    let running = test_db.db.get_running_commands(None, Some(flow_id)).unwrap();
    assert_eq!(running.len(), 2);
}

#[test]
fn test_get_running_commands_both_params_error() {
    let test_db = TestDb::setup_test_db();
    let flow_id = test_db.create_test_workflow("test");
    let cmd_id = test_db.create_test_command("Cmd", "echo 1", None);

    let result = test_db.db.get_running_commands(Some(cmd_id), Some(flow_id));

    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field, reason })
        if field == "workflow_id" && reason.contains("only one query params allowed")
    ));
}

#[test]
fn test_get_running_commands_no_matches() {
    let test_db = TestDb::setup_test_db();
    let running = test_db.db.get_running_commands(None, None).unwrap();
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
fn test_update_execution_pid_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.update_execution_pid(999, 12345);
    assert!(matches!(result, Err(DatabaseError::NotFound {entity: EXECUTION_HISTORY_TABLE, id:999 })));
}

#[test]
fn test_finish_execution_history_success() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();
    let history_id = test_db.save_execution_history(&history);
    test_db.db.update_execution_pid(history_id, 12345).unwrap();

    test_db.db.update_execution_history_status(history_id, ExecutionStatus::Success, Some(0)).unwrap();

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

    test_db.db.update_execution_history_status(history_id, ExecutionStatus::Failed, Some(1)).unwrap();

    let retrieved = test_db.db.get_execution_history(history_id).unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert_eq!(retrieved.exit_code, Some(1));
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
        test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
        ids.push(id);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    test_db.db.cleanup_command_history(cmd_id, 2).unwrap();

    let remaining = test_db.db.get_command_execution_history(cmd_id, None).unwrap();
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
        test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    }

    test_db.db.cleanup_command_history(cmd_id, 10).unwrap();

    let remaining = test_db.db.get_command_execution_history(cmd_id, None).unwrap();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn test_cleanup_history_older_than() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);
    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();

    let id = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    let id = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.save_execution_history(&history);

    // manually set started_at to 10 days ago
    test_db.db.conn().unwrap().execute(
        "UPDATE execution_history SET started_at = datetime('now', '-10 days')",
        params![],
    ).unwrap();

    let id = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    let id = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
    test_db.save_execution_history(&history);
    test_db.save_execution_history(&history);

    test_db.db.cleanup_history_older_than(7).unwrap();

    let remaining = test_db.db.get_command_execution_history(cmd_id, None).unwrap();
    assert_eq!(remaining.len(), 5);
}

#[test]
fn test_cleanup_history_older_than_no_matches() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.cleanup_history_older_than(30);
    assert!(result.is_ok());
}


#[test]
fn test_get_command_execution_stats() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let history = ExecutionHistoryBuilder::new().with_command(cmd_id).build();


    let success1 = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(success1, ExecutionStatus::Success, Some(0)).unwrap();

    let success2 = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(success2, ExecutionStatus::Success, Some(0)).unwrap();


    let failed = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(failed, ExecutionStatus::Failed, Some(1)).unwrap();

    let cancelled = test_db.save_execution_history(&history);
    test_db.db.update_execution_history_status(cancelled, ExecutionStatus::Cancelled, None).unwrap();

    test_db.save_execution_history(&history);

    let total = test_db.db.get_command_execution_stats(cmd_id, None).unwrap();
    assert_eq!(total, 5);
    let success = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Success)).unwrap();
    assert_eq!(success, 2);
    let failed = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Failed)).unwrap();
    assert_eq!(failed, 1);
    let cancelled = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Cancelled)).unwrap();
    assert_eq!(cancelled, 1);
}

#[test]
fn test_get_command_execution_stats_empty() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("Test", "echo", None);

    let total = test_db.db.get_command_execution_stats(cmd_id, None).unwrap();
    assert_eq!(total, 0);
    let success = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Success)).unwrap();
    assert_eq!(success, 0);
    let failed = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Failed)).unwrap();
    assert_eq!(failed, 0);
    let cancelled = test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Cancelled)).unwrap();
    assert_eq!(cancelled, 0);
}


#[test]
fn test_execution_history_stats_after_mixed_runs() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("test", "echo", None);

    let outcomes = [
        (ExecutionStatus::Success, Some(0)),
        (ExecutionStatus::Success, Some(0)),
        (ExecutionStatus::Failed, Some(1)),
        (ExecutionStatus::Cancelled, None),
        (ExecutionStatus::TimeOut, None),
    ];

    for (status, exit_code) in &outcomes {
        let exec_id = test_db.db.create_execution_history(
            &ExecutionHistory::new_with_command(cmd_id, TriggeredBy::Manual)
        ).unwrap();
        test_db.db.update_execution_history_status(exec_id, status.clone(), *exit_code).unwrap();
    }

    assert_eq!(test_db.db.get_command_execution_stats(cmd_id, None).unwrap(), 5);
    assert_eq!(test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Success)).unwrap(), 2);
    assert_eq!(test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Failed)).unwrap(), 1);
    assert_eq!(test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::Cancelled)).unwrap(), 1);
    assert_eq!(test_db.db.get_command_execution_stats(cmd_id, Some(ExecutionStatus::TimeOut)).unwrap(), 1);
}


#[test]
fn test_execution_history_spawn_failure_cancels_row() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("test", "echo test", None);

    let exec_id = test_db.db.create_execution_history(
        &ExecutionHistory::new_with_command(cmd_id, TriggeredBy::Manual)
    ).unwrap();

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

    let exec_ids: Vec<i64> = (0..3).map(|_| {
        let id = test_db.db.create_execution_history(
            &ExecutionHistory::new_with_command(cmd_id, TriggeredBy::Manual)
        ).unwrap();
        test_db.db.update_execution_history_status(id, ExecutionStatus::Success, Some(0)).unwrap();
        id
    }).collect();

    let survivor_id = test_db.db.create_execution_history(
        &ExecutionHistory::new_with_command( other_cmd_id, TriggeredBy::Manual)
    ).unwrap();

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

    let exec_id = test_db.db.create_execution_history(&ExecutionHistoryBuilder::new().with_workflow(flow_id).build()).unwrap();

    test_db.db.delete_workflow(flow_id).unwrap();

    assert!(matches!(
        test_db.db.get_execution_history(exec_id),
        Err(DatabaseError::NotFound { .. })
    ));
}
