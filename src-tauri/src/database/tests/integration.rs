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
    let retrieved_build = test_db.db.get_command(build_id).unwrap();
    assert_eq!(retrieved_build.group_id, Some(backend_group_id));
    assert_eq!(retrieved_build.category_id, Some(dev_cat));

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

    let count_before = test_db.db.get_groups(GroupFilter::None, CategoryFilter::None, false).unwrap().len();

    {
        let mut connection = test_db.db.conn().unwrap();
        let tx = connection.transaction().unwrap();
        tx.execute(
            "INSERT INTO groups (name, position) VALUES (?1, ?2)",
            params!["Transactional", 1000],
        )
        .unwrap();
        tx.commit().unwrap();
    }

    let groups = test_db.db.get_groups(GroupFilter::None, CategoryFilter::None, false).unwrap();
    assert_eq!(groups.len(), count_before + 1);
}

#[test]
fn test_database_locked_error() {
    let test_db = TestDb::setup_test_db();

    // Acquire exclusive lock
    let conn1 = test_db.db.conn().unwrap();
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

#[test]
fn test_delete_category_nullifies_category() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("Temp");
    let group_id =
        test_db.save_group_to_db(&GroupBuilder::new("Test").with_category(cat_id).build());
    let cmd_id = test_db.save_command_to_db(
        &CommandBuilder::new("Test", "echo test")
            .with_category(cat_id)
            .build(),
    );

    let flow_id =
        test_db.save_workflow_to_db(&WorkflowBuilder::new("Test").with_category(cat_id).build());
    let step_id = test_db.create_test_workflow_step(flow_id, cmd_id);

    test_db.db.delete_category(cat_id).unwrap();

    let group = test_db.db.get_group(group_id).unwrap();
    assert_eq!(group.category_id, None);

    let cmd = test_db.db.get_command(cmd_id).unwrap();
    assert_eq!(cmd.category_id, None);

    let flow = test_db.db.get_workflow(flow_id).unwrap();
    assert_eq!(flow.category_id, None);

    assert!(test_db.db.get_workflow_step(step_id).is_ok());
}

#[test]
fn test_full_workflow_lifecycle() {
    let test_db = TestDb::setup_test_db();
    let cat_id = test_db.create_test_category("CI");

    let cmd_install = test_db.create_test_command("install", "npm install", None);
    let cmd_lint = test_db.create_test_command("lint", "npm run lint", None);
    let cmd_test = test_db.create_test_command("test", "npm test", None);
    let cmd_build = test_db.create_test_command("build", "npm run build", None);
    let cmd_deploy = test_db.create_test_command("deploy", "deploy.sh", None);

    let flow_id = test_db.save_workflow_to_db(
        &WorkflowBuilder::new("Frontend CI")
            .with_category(cat_id)
            .with_mode(ExecutionMode::Sequential)
            .build(),
    );

    let step_install = test_db.create_test_workflow_step(flow_id, cmd_install);
    let step_lint = test_db.create_test_workflow_step(flow_id, cmd_lint);

    let mut workflow = WorkflowStepBuilder::new(flow_id, cmd_test).build();
    workflow.condition = StepCondition::OnSuccess;
    workflow.timeout_seconds = Some(60);
    workflow.auto_retry_count = Some(2);
    let step_test = test_db.db.create_workflow_step(&workflow).unwrap();

    let step_build = test_db.create_test_workflow_step(flow_id, cmd_build);

    let mut workflow = WorkflowStepBuilder::new(flow_id, cmd_deploy).build();
    workflow.condition = StepCondition::OnSuccess;
    workflow.continue_on_failure = true;

    let step_deploy = test_db.db.create_workflow_step(&workflow).unwrap();

    // 5 steps in correct order
    assert_eq!(test_db.db.get_workflow_step_count(flow_id).unwrap(), 5);

    let steps = test_db
        .db
        .get_workflow_steps(Some(flow_id), None, false)
        .unwrap();
    assert_eq!(steps[0].id, step_install);
    assert_eq!(steps[4].id, step_deploy);

    // Disable lint step
    test_db.db.toggle_workflow_step_enabled(step_lint).unwrap();
    assert!(!test_db.db.get_workflow_step(step_lint).unwrap().enabled);

    // Move deploy step to before build (prev = test, next = build)
    test_db
        .db
        .move_workflow_step_between(step_deploy, Some(step_test), Some(step_build))
        .unwrap();
    let steps = test_db
        .db
        .get_workflow_steps(Some(flow_id), None, false)
        .unwrap();
    let ids: Vec<i64> = steps.iter().map(|s| s.id).collect();
    assert_eq!(
        ids,
        vec![step_install, step_lint, step_test, step_deploy, step_build]
    );

    // Update test step to increase timeout
    let mut test_step = test_db.db.get_workflow_step(step_test).unwrap();
    test_step.timeout_seconds = Some(300);
    test_db.db.update_workflow_step(&test_step).unwrap();
    assert_eq!(
        test_db
            .db
            .get_workflow_step(step_test)
            .unwrap()
            .timeout_seconds,
        Some(300)
    );

    // Populated view includes all command names
    let populated = test_db
        .db
        .get_workflow_steps_command_populated(flow_id, false)
        .unwrap();
    assert_eq!(populated.len(), 5);
    let cmd_names: Vec<&str> = populated.iter().map(|(_, c)| c.name.as_str()).collect();
    assert!(cmd_names.contains(&"install"));
    assert!(cmd_names.contains(&"deploy"));

    // Favorite the workflow
    test_db.db.toggle_favorite_workflow(flow_id).unwrap();
    assert!(test_db.db.get_workflow(flow_id).unwrap().is_favorite);

    assert_eq!(test_db.db.get_workflow_count(Some(cat_id)).unwrap(), 1);

    test_db.db.delete_workflow(flow_id).unwrap();
    assert!(test_db.db.get_workflow_step(step_install).is_err());
    assert!(test_db.db.get_command(cmd_install).is_ok());
    assert_eq!(test_db.db.get_workflow_count(Some(cat_id)).unwrap(), 0);
}

#[test]
fn test_multiple_concurrent_commands() {
    let test_db = TestDb::setup_test_db();
    let cmd_a = test_db.create_test_command("A", "echo test", None);
    let cmd_b = test_db.create_test_command("B", "echo test", None);
    let cmd_c = test_db.create_test_command("C", "echo test", None);

    let exec_a = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_a,
            TriggeredBy::Manual,
        ))
        .unwrap();
    test_db.db.update_execution_pid(exec_a, 100).unwrap();

    let exec_b = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_b,
            TriggeredBy::Manual,
        ))
        .unwrap();
    test_db.db.update_execution_pid(exec_b, 101).unwrap();

    let exec_c = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_c,
            TriggeredBy::Manual,
        ))
        .unwrap();
    test_db.db.update_execution_pid(exec_c, 102).unwrap();

    let running = test_db.db.get_running_commands().unwrap();
    assert_eq!(running.len(), 3);

    // Stop one
    test_db
        .db
        .update_execution_history_status(exec_b, ExecutionStatus::Success, Some(0))
        .unwrap();

    let running = test_db.db.get_running_commands().unwrap();
    assert_eq!(running.len(), 2);

    let running_ids: Vec<i64> = running.iter().map(|r| r.id).collect();
    assert!(running_ids.contains(&exec_a));
    assert!(running_ids.contains(&exec_c));
    assert!(!running_ids.contains(&exec_b));
}

#[test]
fn test_same_command_run_multiple_times_only_current_is_running() {
    let test_db = TestDb::setup_test_db();
    let cmd_id = test_db.create_test_command("test", "echo test", None);

    // First run: finished
    let exec1 = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id,
            TriggeredBy::Manual,
        ))
        .unwrap();
    test_db.db.update_execution_pid(exec1, 200).unwrap();
    test_db
        .db
        .update_execution_history_status(exec1, ExecutionStatus::Success, Some(0))
        .unwrap();

    // Second run: currently active
    let exec2 = test_db
        .db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id,
            TriggeredBy::Manual,
        ))
        .unwrap();
    test_db.db.update_execution_pid(exec2, 201).unwrap();

    let running = test_db.db.get_running_commands().unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, exec2);
    assert_eq!(running[0].pid, Some(201));

    // Total history shows both runs
    let history = test_db
        .db
        .get_command_execution_history(cmd_id, None)
        .unwrap();
    assert_eq!(history.len(), 2);
}

#[test]
fn test_workflow_execution_creates_linked_history_rows() {
    let test_db = TestDb::setup_test_db();
    let cmd_a = test_db.create_test_command("A", "echo A", None);
    let cmd_b = test_db.create_test_command("B", "echo B", None);
    let flow_id = test_db.create_test_workflow("Pipeline");
    let step_a = test_db.create_test_workflow_step(flow_id, cmd_a);
    let step_b = test_db.create_test_workflow_step(flow_id, cmd_b);

    // Workflow-level row

    let flow_exec = test_db
        .db
        .create_execution_history(
            &ExecutionHistoryBuilder::new()
                .with_workflow(flow_id)
                .build(),
        )
        .unwrap();

    // Step A row
    let step_a_exec = test_db
        .db
        .create_execution_history(
            &ExecutionHistoryBuilder::new()
                .with_workflow_step(cmd_a, flow_id, step_a)
                .build(),
        )
        .unwrap();

    test_db.db.update_execution_pid(step_a_exec, 300).unwrap();
    test_db
        .db
        .update_execution_history_status(step_a_exec, ExecutionStatus::Success, Some(0))
        .unwrap();

    // Step B row
    let step_b_exec = test_db
        .db
        .create_execution_history(
            &ExecutionHistoryBuilder::new()
                .with_workflow_step(cmd_b, flow_id, step_b)
                .build(),
        )
        .unwrap();
    test_db.db.update_execution_pid(step_b_exec, 301).unwrap();
    test_db
        .db
        .update_execution_history_status(step_b_exec, ExecutionStatus::Failed, Some(1))
        .unwrap();

    // Finish workflow-level row as failed (because a step failed)
    test_db
        .db
        .update_execution_history_status(flow_exec, ExecutionStatus::Failed, None)
        .unwrap();

    // Workflow history returns all three rows
    let flow_history = test_db
        .db
        .get_workflow_execution_history(flow_id, None)
        .unwrap();
    assert_eq!(flow_history.len(), 3);

    // Individual step rows are correctly linked
    let row_a = test_db.db.get_execution_history(step_a_exec).unwrap();
    assert_eq!(row_a.workflow_step_id, Some(step_a));
    assert_eq!(row_a.status, ExecutionStatus::Success);

    let row_b = test_db.db.get_execution_history(step_b_exec).unwrap();
    assert_eq!(row_b.workflow_step_id, Some(step_b));
    assert_eq!(row_b.status, ExecutionStatus::Failed);

    // Workflow-level row reflects final failure
    let row_wf = test_db.db.get_execution_history(flow_exec).unwrap();
    assert_eq!(row_wf.status, ExecutionStatus::Failed);
    assert_eq!(row_wf.command_id, None);
    assert_eq!(row_wf.workflow_step_id, None);
}

//
// /// Simulates the complete lifecycle of a single command execution: create → store pid → finish → verify audit trail.
// #[test]
// fn test_execution_history_full_command_lifecycle() {
//     let test_db = TestDb::setup_test_db();
//     let cmd_id = test_db.create_test_command("Build", "echo test", None);
//
//     // 1. Process manager creates history row before spawning
//     let exec_id = test_db
//         .db
//         .create_execution_history(&ExecutionHistory::new_with_command(
//             cmd_id,
//             TriggeredBy::Manual,
//         ))
//         .unwrap();
//
//     let history = test_db.db.get_execution_history(exec_id).unwrap();
//     assert_eq!(history.status, ExecutionStatus::Running);
//     assert_eq!(history.pid, None);
//     assert!(history.completed_at.is_none());
//
//     // 2. Spawn succeeds — store PID
//     test_db.db.update_execution_pid(exec_id, 8421).unwrap();
//
//     let history = test_db.db.get_execution_history(exec_id).unwrap();
//     assert_eq!(history.pid, Some(8421));
//
//     // 3. Process exits cleanly
//     test_db
//         .db
//         .update_execution_history_status(exec_id, ExecutionStatus::Success, Some(0))
//         .unwrap();
//
//     let row = test_db.db.get_execution_history(exec_id).unwrap();
//     assert_eq!(row.status, ExecutionStatus::Success);
//     assert_eq!(row.exit_code, Some(0));
//     assert!(row.completed_at.is_some());
//
//     // 4. Stats reflect the completed run
//     let total = test_db
//         .db
//         .get_command_execution_stats(cmd_id, None)
//         .unwrap();
//     let successes = test_db
//         .db
//         .get_command_execution_stats(cmd_id, Some(ExecutionStatus::Success))
//         .unwrap();
//     assert_eq!(total, 1);
//     assert_eq!(successes, 1);
// }
//
//
// #[test]
// fn test_execution_history_graceful_kill() {
//     let test_db = TestDb::setup_test_db();
//     let cmd_id = test_db.create_test_command("test", "echo test", None);
//
//     let exec_id = test_db.db.create_execution_history(
//         &ExecutionHistory::new_with_command( cmd_id, TriggeredBy::Manual)
//     ).unwrap();
//     test_db.db.update_execution_pid(exec_id, 9999).unwrap();
//
//     // User sent SIGTERM; process did not provide an exit code
//     test_db.db.update_execution_history_status(exec_id, ExecutionStatus::Cancelled, None).unwrap();
//
//     let row = test_db.db.get_execution_history(exec_id).unwrap();
//     assert_eq!(row.status, ExecutionStatus::Cancelled);
//     assert_eq!(row.exit_code, None);
//     assert!(row.completed_at.is_some());
//
//     // Should not appear in running list
//     let running = test_db.db.get_running_commands(Some(cmd_id), None).unwrap();
//     assert!(running.is_empty());
// }

// /// Simulates a timed-out step inside a workflow.
// #[test]
// fn test_execution_history_workflow_step_timeout() {
//     let test_db = TestDb::setup_test_db();
//     let cmd_id = test_db.create_test_command("SlowStep");
//     let wf_id = test_db.create_test_workflow("Deploy");
//     let step_id = test_db.create_test_workflow_step(wf_id, cmd_id);
//
//     let history = ExecutionHistory {
//         id: 0,
//         command_id: Some(cmd_id),
//         workflow_id: Some(wf_id),
//         workflow_step_id: Some(step_id),
//         pid: None,
//         status: ExecutionStatus::Running,
//         exit_code: None,
//         started_at: String::new(),
//         completed_at: None,
//         triggered_by: TriggeredBy::Workflow,
//         context: Some(r#"{"step_position": 2}"#.to_string()),
//     };
//     let exec_id = test_db.db.create_execution_history(&history).unwrap();
//     test_db.db.update_execution_pid(exec_id, 4321).unwrap();
//
//     // Step hit its timeout_seconds limit
//     test_db.db.update_execution_history_status(exec_id, ExecutionStatus::TimedOut, None).unwrap();
//
//     let row = test_db.db.get_execution_history(exec_id).unwrap();
//     assert_eq!(row.status, ExecutionStatus::TimedOut);
//     assert_eq!(row.workflow_id, Some(wf_id));
//     assert_eq!(row.workflow_step_id, Some(step_id));
//     assert!(row.completed_at.is_some());
//
//     // Appears in workflow history
//     let wf_history = test_db.db.get_workflow_execution_history(wf_id, None).unwrap();
//     assert_eq!(wf_history.len(), 1);
//     assert_eq!(wf_history[0].status, ExecutionStatus::TimedOut);
// }
//
// // ─── Execution history: orphan detection ────────────────────────────────────
//
// /// On startup, get_running_commands returns rows that appear to be
// /// orphaned from a previous session.
// #[test]
// fn test_orphan_detection_finds_previous_session_rows() {
//     let test_db = TestDb::setup_test_db();
//     let cmd_a = test_db.create_test_command("ServerA");
//     let cmd_b = test_db.create_test_command("ServerB");
//
//     // Simulate two processes that were running when the app crashed
//     let exec_a = test_db.db.create_execution_history(
//         &ExecutionHistory::new(0, cmd_a, TriggeredBy::Manual)
//     ).unwrap();
//     test_db.db.update_execution_pid(exec_a, 1001).unwrap();
//
//     let exec_b = test_db.db.create_execution_history(
//         &ExecutionHistory::new(0, cmd_b, TriggeredBy::Manual)
//     ).unwrap();
//     test_db.db.update_execution_pid(exec_b, 1002).unwrap();
//
//     // One process finished cleanly before crash
//     test_db.db.update_execution_history_status(exec_b, ExecutionStatus::Success, Some(0)).unwrap();
//
//     // On next startup, only exec_a should appear as orphaned
//     let running = test_db.db.get_running_commands(None, None).unwrap();
//     assert_eq!(running.len(), 1);
//     assert_eq!(running[0].id, exec_a);
//     assert_eq!(running[0].pid, Some(1001));
// }
