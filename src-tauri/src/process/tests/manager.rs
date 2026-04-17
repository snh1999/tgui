use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

use crate::database::tests::{CommandBuilder, GroupBuilder, TestDb};
use crate::database::{Database, ExecutionHistory, ExecutionStatus, StatsTarget, TriggeredBy};
use crate::process::errors::{ProcessKillError, ProcessManagerError};
use crate::process::manager::ProcessManager;
use crate::process::models::{ProcessStatus, SpawnContext};
use crate::process::tests::{spawn_context, WAIT_TIMEOUT};

#[cfg(unix)]
use nix::sys::signal::{kill as nix_kill, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

fn create_test_command(db: &Database) -> i64 {
    db.create_command(&CommandBuilder::new("test_command", "echo test").build())
        .expect("Failed to create test command")
}

pub fn create_test_db() -> Database {
    TestDb::setup_test_db().db
}

macro_rules! wait_until {
    ($label:expr, $condition:expr) => {
        timeout(WAIT_TIMEOUT, async {
            loop {
                if $condition {
                    return;
                }
                sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("Timed out waiting for: {}", $label))
    };
}

fn make_manager() -> Arc<ProcessManager> {
    ProcessManager::new(create_test_db(), None)
}

fn make_manager_with_db() -> (Arc<ProcessManager>, Database, i64) {
    let db = create_test_db();
    let cmd_id = create_test_command(&db);
    let pm = ProcessManager::new(db.clone(), None);
    (pm, db, cmd_id)
}

#[tokio::test]
#[cfg(unix)]
async fn spawn_creates_execution_history_row_with_running_status() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;

    let history = db.get_execution_history(id).expect("no history row");
    assert_eq!(history.id, id);
    assert_eq!(history.status, ExecutionStatus::Running);
    pm.stop_all(true).await;
}

#[tokio::test]
#[cfg(unix)]
async fn spawn_writes_pid_to_execution_history() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;

    let history = db.get_execution_history(id).expect("no history row");
    assert!(history.pid.is_some(), "PID not written to DB");
    assert!(history.pid.unwrap() > 0);
    pm.stop_all(true).await;
}

#[tokio::test]
async fn consecutive_spawns_get_different_execution_ids() {
    let (pm, _, cmd_id) = make_manager_with_db();

    let id1 = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["a"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 1 failed");

    sleep(Duration::from_millis(100)).await;

    let id2 = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["b"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 2 failed");
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn spawn_stores_correct_triggered_by_in_db() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["x"]),
            TriggeredBy::Workflow,
        )
        .await
        .expect("spawn failed");

    wait_until!("row stable", db.get_execution_history(id).is_ok());
    assert_eq!(
        db.get_execution_history(id).unwrap().triggered_by,
        TriggeredBy::Workflow
    );
}

#[tokio::test]
async fn spawn_invalid_shell_returns_error_before_creating_db_row() {
    let (pm, db, command_id) = make_manager_with_db();
    let ctx = SpawnContext {
        command_id,
        name: "test".into(),
        executable: "echo".into(),
        arguments: vec![],
        working_directory: PathBuf::from("/tmp"),
        env_vars: vec![],
        shell: Some("rm".into()),
    };

    let result = pm.spawn_command(ctx, TriggeredBy::Manual).await;
    assert!(matches!(
        result,
        Err(crate::process::errors::ProcessSpawnError::InvalidShell(_))
    ));

    // No orphaned DB row should exist — invalid shell is rejected before DB write
    let running = db.get_running_commands().unwrap_or_default();
    assert!(
        running.is_empty(),
        "Orphaned DB row created for rejected shell"
    );
}

#[tokio::test]
async fn spawn_failure_marks_db_row_as_failed_not_running() {
    let (pm, db, cmd_id) = make_manager_with_db();

    let result = pm
        .spawn_command(
            spawn_context(cmd_id, "this_does_not_exist_tgui", vec![]),
            TriggeredBy::Manual,
        )
        .await;

    assert!(result.is_err(), "Expected spawn to fail");

    // The db entry must not be stuck as 'running'
    let running = db.get_running_commands().unwrap_or_default();
    assert!(
        running.is_empty(),
        "DB row left as 'running' after spawn failure — cancel_execution not called"
    );
}

#[tokio::test]
async fn spawn_pid_update_failure_kills_the_process() {
    // This test requires a way to simulate update_execution_pid failure.
    // this test is for documentation, no way to simulate db failure now
}

#[tokio::test]
async fn natural_exit_code_zero_updates_db_to_success() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["done"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("DB = success", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Success)
            .unwrap_or(false)
    });

    let history = db.get_execution_history(id).unwrap();
    assert_eq!(history.exit_code, Some(0));
    assert!(
        history.completed_at.is_some(),
        "completed_at not set by trigger"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn natural_exit_nonzero_updates_db_to_failed() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(spawn_context(cmd_id, "false", vec![]), TriggeredBy::Manual)
        .await
        .expect("spawn failed");

    wait_until!("DB = failed", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Failed)
            .unwrap_or(false)
    });

    let history = db.get_execution_history(id).unwrap();
    assert_ne!(history.exit_code.unwrap_or(0), 0);
    assert!(history.completed_at.is_some());
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_updates_db_to_cancelled() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;
    pm.kill_process(id, false).await.expect("kill failed");

    wait_until!("DB = cancelled", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Cancelled)
            .unwrap_or(false)
    });

    let h = db.get_execution_history(id).unwrap();
    assert!(h.completed_at.is_some());
}

#[tokio::test]
#[cfg(unix)]
async fn stop_all_graceful_sends_sigterm() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;
    let killed = pm.stop_all(false).await;
    assert_eq!(killed, 1);

    wait_until!("DB = cancelled", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Cancelled)
            .unwrap_or(false)
    });
}

#[tokio::test]
#[cfg(unix)]
async fn stop_all_skips_already_exited_processes_in_count() {
    let (pm, db, cmd_id) = make_manager_with_db();

    pm.spawn_command(
        spawn_context(cmd_id, "echo", vec!["bye"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    let cmd_id = create_test_command(&db);

    pm.spawn_command(
        spawn_context(cmd_id, "echo", vec!["bye"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    let cmd_id = create_test_command(&db);

    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    let cmd_id = create_test_command(&db);

    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    let cmd_id = create_test_command(&db);

    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    wait_until!("running_count = 3", pm.running_count().await == 3);

    let killed = pm.stop_all(true).await;
    assert_eq!(
        killed, 3,
        "Should only count the process that was actually killed"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn force_kill_updates_db_to_cancelled() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;
    pm.kill_process(id, true).await.expect("force kill failed");

    wait_until!("DB = cancelled", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Cancelled)
            .unwrap_or(false)
    });
}

#[tokio::test]
async fn kill_unknown_execution_id_returns_not_found() {
    let pm = make_manager();
    let result = pm.kill_process(999_999, false).await;
    assert!(matches!(result, Err(ProcessKillError::NotFound(_))));
}

#[tokio::test]
#[cfg(unix)]
async fn stop_all_kills_all_running_processes() {
    let (pm, db, cmd_id) = make_manager_with_db();

    let id1 = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 1 failed");

    let cmd_id = create_test_command(&db);
    let id2 = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 2 failed");

    let cmd_id = create_test_command(&db);
    let id3 = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 3 failed");

    sleep(Duration::from_millis(200)).await;

    let killed = pm.stop_all(true).await;
    assert_eq!(killed, 3);

    wait_until!("all cancelled in DB", {
        [id1, id2, id3].iter().all(|&id| {
            db.get_execution_history(id)
                .map(|h| h.status == ExecutionStatus::Cancelled)
                .unwrap_or(false)
        })
    });
}

#[tokio::test]
async fn stop_all_on_empty_manager_returns_zero() {
    let pm = make_manager();
    assert_eq!(pm.stop_all(true).await, 0);
}

#[tokio::test]
#[cfg(unix)]
async fn stop_all_does_not_double_kill_already_stopping_processes() {
    let (pm, _, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;
    pm.kill_process(id, false).await.ok();

    // stop_all should not try to kill it again since is_running=false during Stopping
    let count = pm.stop_all(true).await;
    assert_eq!(
        count, 0,
        "stop_all killed a process that was already Stopping"
    );
}

#[tokio::test]
async fn running_count_zero_on_fresh_manager() {
    let pm = make_manager();
    assert_eq!(pm.running_count().await, 0);
}

#[tokio::test]
#[cfg(unix)]
async fn running_count_tracks_spawned_processes() {
    let (pm, db, cmd_id) = make_manager_with_db();

    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn 1 failed");

    let cmd_id = create_test_command(&db);
    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn 2 failed");

    sleep(Duration::from_millis(100)).await;
    assert_eq!(pm.running_count().await, 2);

    pm.stop_all(true).await;
}

#[tokio::test]
async fn running_count_decreases_after_process_exits() {
    let (pm, _, cmd_id) = make_manager_with_db();

    pm.spawn_command(
        spawn_context(cmd_id, "echo", vec!["bye"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    wait_until!("count = 0", pm.running_count().await == 0);
}

#[tokio::test]
#[cfg(unix)]
async fn get_process_info_returns_correct_fields_for_running_process() {
    let (pm, _, cmd_id) = make_manager_with_db();

    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    sleep(Duration::from_millis(100)).await;

    let info = pm.get_process_info(id).await.expect("expected Some");
    assert_eq!(info.execution_id, id);
    assert_eq!(info.command_id, 1);
    assert!(info.pid > 0);
    assert!(matches!(info.status, ProcessStatus::Running { .. }));
    assert!(!info.start_time.is_empty());

    pm.kill_process(id, true).await.ok();
}

#[tokio::test]
#[cfg(unix)]
async fn get_running_processes_excludes_stopped_processes() {
    let (pm, db, cmd_id) = make_manager_with_db();

    pm.spawn_command(
        spawn_context(cmd_id, "echo", vec!["done"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn echo failed");

    sleep(Duration::from_millis(100)).await;
    let sleep_id = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn sleep failed");

    let cmd_id = create_test_command(&db);
    pm.spawn_command(
        spawn_context(cmd_id, "sleep", vec!["60"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn sleep failed");

    wait_until!("count = 2", pm.running_count().await == 2);

    let running = pm.get_running_processes().await;
    assert_eq!(
        running.len(),
        2,
        "Only the sleeping process should be running"
    );
    assert!(running.iter().any(|v| v.execution_id == sleep_id));

    pm.stop_all(true).await;
}

#[tokio::test]
#[cfg(unix)]
async fn get_logs_offset_and_limit_paginate_correctly() {
    let (pm, _, cmd_id) = make_manager_with_db();

    let id = pm
        .spawn_command(
            spawn_context(
                cmd_id,
                "sh",
                vec![
                    "-c".into(),
                    "echo l1; echo l2; echo l3; echo l4; echo l5".into(),
                ],
            ),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn echo failed");

    wait_until!("count = 0", pm.running_count().await == 0);
    sleep(Duration::from_millis(100)).await;

    let all = pm.get_logs(id, 0, 100).await.expect("logs");
    let page = pm.get_logs(id, 1, 2).await.expect("page");

    assert_eq!(page.len(), 2);
    assert_eq!(page[0].content, all[1].content);
    assert_eq!(page[1].content, all[2].content);
}

#[tokio::test(start_paused = true)]
async fn process_removed_from_map_after_cleanup_delay() {
    let (pm, _, cmd_id) = make_manager_with_db();
    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["cleanup"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("count = 0", pm.running_count().await == 0);

    // Still accessible within the 5s window
    let info = pm.get_process_info(id).await;
    assert!(
        info.is_some(),
        "Process should still be in map within 5s window"
    );
    assert!(info.unwrap().log_line_count >= 1);

    // Logs should be readable immediately after process stops, within the window
    let logs = pm.get_logs(id, 0, 100).await;
    assert!(
        logs.is_some(),
        "Logs should be accessible within cleanup window"
    );

    tokio::time::advance(Duration::from_secs(6)).await;
    tokio::time::sleep(Duration::from_millis(1)).await;

    assert!(
        pm.get_process_info(id).await.is_none(),
        "Process should be removed from map after cleanup delay"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn get_process_info_returns_none_for_unknown_id() {
    let pm = make_manager();
    assert!(pm.get_process_info(999_999).await.is_none());
}

#[tokio::test]
async fn get_running_processes_lists_all_active() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let cmd_id_1 = create_test_command(&db);
    let id1 = pm
        .spawn_command(
            spawn_context(cmd_id, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 1 failed");
    let id2 = pm
        .spawn_command(
            spawn_context(cmd_id_1, "sleep", vec!["60"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 2 failed");

    sleep(Duration::from_millis(100)).await;

    let running = pm.get_running_processes().await;
    let ids: Vec<i64> = running.iter().map(|p| p.execution_id).collect();
    assert!(ids.contains(&id1), "id1 missing from running list");
    assert!(ids.contains(&id2), "id2 missing from running list");

    pm.stop_all(true).await;
}

#[tokio::test]
async fn get_running_processes_empty_after_all_exit() {
    let (pm, _, cmd_id) = make_manager_with_db();
    pm.spawn_command(
        spawn_context(cmd_id, "echo", vec!["done"]),
        TriggeredBy::Manual,
    )
    .await
    .expect("spawn failed");

    wait_until!("count = 0", pm.running_count().await == 0);
    assert!(pm.get_running_processes().await.is_empty());
}

#[tokio::test]
async fn get_logs_returns_stdout_from_process() {
    let (pm, _, cmd_id) = make_manager_with_db();

    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["manager_log"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("count = 0", pm.running_count().await == 0);
    sleep(Duration::from_millis(100)).await;

    let logs = pm.get_logs(id, 0, 100).await.expect("expected Some");
    assert!(
        logs.iter().any(|l| l.content.contains("manager_log")),
        "Expected 'manager_log' in logs: {logs:?}"
    );
}

#[tokio::test]
async fn get_logs_returns_none_for_unknown_id() {
    let pm = make_manager();
    assert!(pm.get_logs(999_999, 0, 100).await.is_none());
}

#[tokio::test]
async fn clear_logs_empties_buffer() {
    let (pm, _, cmd_id) = make_manager_with_db();

    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["clear_test"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("count = 0", pm.running_count().await == 0);
    sleep(Duration::from_millis(100)).await;

    pm.clear_logs(id).await.expect("clear_logs failed");
    assert!(pm
        .get_logs(id, 0, 100)
        .await
        .expect("expected Some")
        .is_empty());
}

#[tokio::test]
async fn clear_logs_unknown_id_returns_not_found() {
    let pm = make_manager();
    let result = pm.clear_logs(999_999).await;
    assert!(matches!(
        result,
        Err(ProcessManagerError::ProcessNotFound(_))
    ));
}

#[tokio::test]
async fn concurrent_processes_have_isolated_log_buffers() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let cmd_id_1 = create_test_command(&db);
    let id1 = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["alpha_unique"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 1 failed");
    let id2 = pm
        .spawn_command(
            spawn_context(cmd_id_1, "echo", vec!["beta_unique"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 2 failed");

    wait_until!("count = 0", pm.running_count().await == 0);
    sleep(Duration::from_millis(100)).await;

    let logs1 = pm.get_logs(id1, 0, 100).await.expect("logs1");
    let logs2 = pm.get_logs(id2, 0, 100).await.expect("logs2");

    assert!(
        logs1.iter().any(|l| l.content.contains("alpha_unique")),
        "id1 missing own output"
    );
    assert!(
        logs2.iter().any(|l| l.content.contains("beta_unique")),
        "id2 missing own output"
    );
    assert!(
        !logs1.iter().any(|l| l.content.contains("beta_unique")),
        "id1 leaked id2's output"
    );
    assert!(
        !logs2.iter().any(|l| l.content.contains("alpha_unique")),
        "id2 leaked id1's output"
    );
}

#[tokio::test]
async fn concurrent_processes_have_isolated_db_rows() {
    let (pm, db, cmd_id) = make_manager_with_db();
    let cmd_2 = create_test_command(&db);
    let id1 = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["a"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn 1 failed");
    let id2 = pm
        .spawn_command(spawn_context(cmd_2, "echo", vec!["b"]), TriggeredBy::Manual)
        .await
        .expect("spawn 2 failed");

    wait_until!("both success in DB", {
        let s1 = db
            .get_execution_history(id1)
            .map(|h| h.status == ExecutionStatus::Success)
            .unwrap_or(false);
        let s2 = db
            .get_execution_history(id2)
            .map(|h| h.status == ExecutionStatus::Success)
            .unwrap_or(false);
        s1 && s2
    });

    assert_eq!(db.get_execution_history(id1).unwrap().command_id, Some(1));
    assert_eq!(db.get_execution_history(id2).unwrap().command_id, Some(2));
}

#[tokio::test]
async fn detect_orphans_empty_db_returns_empty() {
    let pm = make_manager();
    assert!(pm.detect_and_mark_orphans().is_empty());
}

#[tokio::test]
#[cfg(unix)]
async fn detect_orphans_null_pid_row_marked_failed_not_in_result() {
    // A row with pid=NULL means spawn crashed before update_execution_pid.
    // detect_and_mark_orphans should mark it Failed and not include it in the orphans vec
    let test_db = create_test_db();

    // Insert a command row to satisfy FK, then insert a history row with no PID.
    let cmd_id = create_test_command(&test_db);
    test_db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id,
            TriggeredBy::Manual,
        ))
        .expect("create history");
    // the row has status='running' and pid=NULL here
    let pm = ProcessManager::new(test_db.clone(), None);
    let orphans = pm.detect_and_mark_orphans();

    // Null-PID rows must be cleaned up, not returned
    assert!(
        orphans.is_empty(),
        "Null-PID orphan should be cleaned up, not returned: {orphans:?}"
    );

    // DB row must be marked failed
    let rows = test_db.get_running_commands().unwrap();
    assert!(
        rows.is_empty(),
        "Null-PID row still marked running after detect_and_mark_orphans"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn detect_orphans_dead_pid_row_marked_failed_in_result_with_still_running_false() {
    let db = create_test_db();
    let cmd_id = create_test_command(&db);

    // Spawn a real process directly — no ProcessManager, no monitor task
    let mut child = tokio::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("failed to spawn sleep");
    let pid = child.id().expect("no pid");

    // Write the DB row manually as if ProcessManager had done it
    let execution_id = db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id,
            TriggeredBy::Manual,
        ))
        .expect("failed to create history row");
    db.update_execution_pid(execution_id, pid)
        .expect("failed to write pid");

    // Kill the process externally — simulating a crash
    nix_kill(Pid::from_raw(pid as i32), Signal::SIGKILL).expect("kill failed");
    // Reap it so it doesn't become a zombie
    let _ = child.wait().await;
    sleep(Duration::from_millis(100)).await;

    // Now simulate app restart — fresh ProcessManager, no in-memory state,
    // DB row still says 'running' because no monitor task updated it
    let pm = ProcessManager::new(db.clone(), None);
    let orphans = pm.detect_and_mark_orphans();

    let orphan = orphans
        .iter()
        .find(|o| o.execution_id == execution_id)
        .expect("orphan not found in result");
    assert!(
        !orphan.still_running,
        "dead process should have still_running=false"
    );
    assert_eq!(orphan.pid, pid as i64);

    let h = db.get_execution_history(execution_id).unwrap();
    assert_eq!(
        h.status,
        ExecutionStatus::Failed,
        "dead orphan DB row should be marked Failed"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn detect_orphans_alive_pid_in_result_with_still_running_true() {
    let db = create_test_db();
    let cmd_id = create_test_command(&db);

    // Spawn process and immediately drop the ProcessManager
    // so it doesn't concurrently update DB state during detect
    let id = {
        let pm_temp = ProcessManager::new(db.clone(), None);
        let id = pm_temp
            .spawn_command(
                spawn_context(cmd_id, "sleep", vec!["60"]),
                TriggeredBy::Manual,
            )
            .await
            .expect("spawn failed");
        sleep(Duration::from_millis(100)).await;
        id
        // pm_temp dropped here — no concurrent DB updates
    };

    let pid = db.get_execution_history(id).unwrap().pid.unwrap() as u32;

    // Simulate restart with a fresh ProcessManager
    let pm2 = ProcessManager::new(db.clone(), None);
    let orphans = pm2.detect_and_mark_orphans();

    // Give OS time to process SIGKILL before checking
    sleep(Duration::from_millis(100)).await;

    // Orphan should be in results with still_running=true
    let orphan = orphans
        .iter()
        .find(|o| o.execution_id == id)
        .expect("alive orphan not in result");

    assert!(
        orphan.still_running,
        "alive process should have still_running=true"
    );

    // DB row should be Canceled — implementation kills and marks alive orphans
    let h = db.get_execution_history(id).unwrap();
    assert_eq!(
        h.status,
        ExecutionStatus::Cancelled,
        "alive orphan should be marked Cancelled"
    );

    // OS process should be dead — implementation SIGKILLs alive orphans
    let still_alive = nix_kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok();
    assert!(!still_alive, "orphan OS process should have been killed");
}

#[tokio::test]
#[cfg(unix)]
async fn detect_orphans_mixed_dead_and_alive_handled_independently() {
    let db = create_test_db();
    let cmd_id_1 = create_test_command(&db);
    let cmd_id_2 = create_test_command(&db);

    // Spawn two processes directly
    let mut child1 = tokio::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn 1");
    let pid1 = child1.id().expect("pid1");

    let mut child2 = tokio::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn 2");
    let pid2 = child2.id().expect("pid2");

    let exec_id1 = db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id_1,
            TriggeredBy::Manual,
        ))
        .expect("history 1");
    db.update_execution_pid(exec_id1, pid1).expect("pid1 write");

    nix_kill(Pid::from_raw(pid1 as i32), Signal::SIGKILL).expect("kill pid1");
    let _ = child1.wait().await;

    let exec_id2 = db
        .create_execution_history(&ExecutionHistory::new_with_command(
            cmd_id_2,
            TriggeredBy::Manual,
        ))
        .expect("history 2");
    db.update_execution_pid(exec_id2, pid2).expect("pid2 write");

    let pm = ProcessManager::new(db.clone(), None);
    let orphans = pm.detect_and_mark_orphans();

    let orphan1 = orphans
        .iter()
        .find(|o| o.execution_id == exec_id1)
        .expect("dead orphan missing");

    let orphan2 = orphans
        .iter()
        .find(|o| o.execution_id == exec_id2)
        .expect("alive orphan missing");

    assert!(!orphan1.still_running, "pid1 is dead");
    assert!(orphan2.still_running, "pid2 is alive");

    assert_eq!(
        db.get_execution_history(exec_id1).unwrap().status,
        ExecutionStatus::Failed
    );
    assert_eq!(
        db.get_execution_history(exec_id2).unwrap().status,
        ExecutionStatus::Cancelled
    );

    // Cleanup pid2
    let _ = nix_kill(Pid::from_raw(pid2 as i32), Signal::SIGKILL);
    let _ = child2.wait().await;
}

#[tokio::test]
async fn one_spawn_produces_exactly_one_execution_history_row() {
    let (pm, db, cmd_id) = make_manager_with_db();

    let before = db
        .get_execution_stats(StatsTarget::Command(cmd_id), None)
        .unwrap()
        .total_count;

    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["once"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("terminal status", {
        db.get_execution_history(id)
            .map(|h| h.status != ExecutionStatus::Running)
            .unwrap_or(false)
    });

    let after = db
        .get_execution_stats(StatsTarget::Command(cmd_id), None)
        .expect("stats failed")
        .total_count;
    assert_eq!(
        after - before,
        1,
        "Expected exactly 1 new history row per spawn"
    );
}

#[tokio::test]
async fn successful_run_increments_success_count_not_failed_count() {
    let (pm, db, cmd_id) = make_manager_with_db();

    let id = pm
        .spawn_command(
            spawn_context(cmd_id, "echo", vec!["ok"]),
            TriggeredBy::Manual,
        )
        .await
        .expect("spawn failed");

    wait_until!("success in DB", {
        db.get_execution_history(id)
            .map(|h| h.status == ExecutionStatus::Success)
            .unwrap_or(false)
    });

    let stats = db
        .get_execution_stats(StatsTarget::Command(cmd_id), None)
        .expect("stats");
    let success = stats.success_count;
    let failed = stats.failed_count;

    assert!(success >= 1);
    assert_eq!(failed, 0);
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_returns_correct_fields() {
    let db = create_test_db();
    let mut cmd = CommandBuilder::new("test", "echo test").build();
    cmd.working_directory = Some("/tmp".to_string());
    cmd.shell = Some("bash".to_string());

    let cmd_id = db.create_command(&cmd).unwrap();
    let pm = ProcessManager::new(db, None);

    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");

    assert_eq!(ctx.command_id, cmd_id);
    assert_eq!(ctx.name, cmd.name);
    assert_eq!(ctx.executable, cmd.command);
    assert_eq!(ctx.working_directory, PathBuf::from("/tmp"));
    assert_eq!(ctx.shell, cmd.shell);
}

#[tokio::test]
async fn resolve_spawn_context_command_not_found_returns_not_found_error() {
    let pm = make_manager();
    let result = pm.resolve_spawn_context(99).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_working_dir_from_command() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    let group_id = db.create_group(&group).expect("create group");

    let mut cmd = CommandBuilder::new("test", "echo test")
        .with_group(group_id)
        .build();
    cmd.working_directory = Some("/var/tmp".to_string());
    let cmd_id = db.create_command(&cmd).unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");
    assert_eq!(
        ctx.working_directory,
        PathBuf::from(cmd.working_directory.unwrap())
    );
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_working_dir_falls_back_to_group() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    let group_id = db.create_group(&group).expect("create group");

    let cmd = CommandBuilder::new("test", "echo test")
        .with_group(group_id)
        .build();
    let cmd_id = db.create_command(&cmd).unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");
    assert_eq!(
        ctx.working_directory,
        PathBuf::from(group.working_directory.unwrap())
    );
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_tilde_expanded_to_home() {
    let db = create_test_db();

    let mut cmd = CommandBuilder::new("test", "echo test").build();
    cmd.working_directory = Some("~".to_string());
    let cmd_id = db.create_command(&cmd).unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");

    let home = dirs::home_dir().expect("no home dir");
    assert_eq!(ctx.working_directory, home);
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_shell_command_uses_command_shell() {
    let db = create_test_db();

    db.set_setting("default_shell", "sh").ok();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    group.shell = Some("zsh".to_string());
    let group_id = db.create_group(&group).expect("create group");

    let mut cmd = CommandBuilder::new("test", "echo test")
        .with_group(group_id)
        .build();
    cmd.shell = Some("bash".to_string());
    let cmd_id = db.create_command(&cmd).unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");
    assert_eq!(ctx.shell, cmd.shell);
    assert_eq!(
        ctx.working_directory,
        PathBuf::from(group.working_directory.unwrap())
    );
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_shell_falls_back_to_group() {
    let db = create_test_db();
    db.set_setting("default_shell", "sh").ok();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    group.shell = Some("zsh".to_string());
    let group_id = db.create_group(&group).expect("create group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(group_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");
    assert_eq!(ctx.shell, group.shell);
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_shell_no_group_falls_back_to_default_setting() {
    let db = create_test_db();
    db.set_setting("default_shell", "sh").ok();

    let cmd_id = db
        .create_command(&CommandBuilder::new("test", "echo test").build())
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");
    assert_eq!(ctx.shell, Some("sh".to_string()));
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_shell_falls_back_to_default_setting() {
    let db = create_test_db();
    db.set_setting("default_shell", "sh").ok();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    let group_id = db.create_group(&group).expect("create group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(group_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");
    assert_eq!(ctx.shell, Some("sh".to_string()));
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_env_vars_group_applied_as_base() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group")
        .with_env("GROUP_VAR", "from_group")
        .build();
    group.working_directory = Some("/tmp".to_string());

    let group_id = db.create_group(&group).expect("create group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(group_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");

    assert!(
        ctx.env_vars
            .iter()
            .any(|(k, v)| k == "GROUP_VAR" && v == "from_group"),
        "Group env var missing: {:?}",
        ctx.env_vars
    );
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_env_vars_command_overrides_group_for_same_key() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group")
        .with_env("GROUP_VAR", "from_group")
        .with_env("SHARED_KEY", "group_value")
        .build();
    group.working_directory = Some("/tmp".to_string());

    let group_id = db.create_group(&group).expect("create group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(group_id)
                .with_env("SHARED_KEY", "cmd_value")
                .with_env("COMMAND_KEY", "from_cmd")
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");

    // SHARED_KEY should appear exactly once
    let shared = ctx.env_vars.iter().filter(|(k, _)| k == "SHARED_KEY");
    assert_eq!(
        shared.count(),
        1,
        "SHARED_KEY should appear exactly once after dedup"
    );
    // Command overrides group for SHARED_KEY
    let shared = ctx.env_vars.iter().find(|(k, _)| k == "SHARED_KEY");
    assert_eq!(
        shared.map(|(_, v)| v.as_str()),
        Some("cmd_value"),
        "Command should override group"
    );
    assert!(shared
        .iter()
        .all(|(k, v)| k == "SHARED_KEY" && v == "cmd_value"));

    assert!(ctx
        .env_vars
        .iter()
        .any(|(k, v)| k == "GROUP_VAR" && v == "from_group"));

    assert!(ctx
        .env_vars
        .iter()
        .any(|(k, v)| k == "COMMAND_KEY" && v == "from_cmd"),);
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_working_dir_walks_to_grandparent() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    let root_id = db.create_group(&group).expect("root group");

    let child_id = db
        .create_group(&GroupBuilder::new("child").with_parent(root_id).build())
        .expect("child group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(child_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");
    assert_eq!(ctx.working_directory, PathBuf::from("/tmp"));
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_closest_ancestor_wins_over_grandparent() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group").build();
    group.working_directory = Some("/tmp".to_string());
    let root_id = db.create_group(&group).expect("root group");

    let mut dir_group = GroupBuilder::new("group").with_parent(root_id).build();
    dir_group.working_directory = Some("/var/tmp".to_string());
    let parent_id = db.create_group(&dir_group).expect("root group");

    let mut shell_group = GroupBuilder::new("group").with_parent(parent_id).build();
    shell_group.shell = Some("zsh".to_string());

    let child_id = db.create_group(&shell_group).expect("child group");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(child_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm
        .resolve_spawn_context(cmd_id)
        .await
        .expect("resolve failed");
    assert_eq!(
        ctx.working_directory,
        PathBuf::from(dir_group.working_directory.unwrap())
    );
    assert_eq!(ctx.shell, shell_group.shell);
}

#[tokio::test]
#[cfg(unix)]
async fn resolve_spawn_context_env_vars_merge_across_ancestor_chain() {
    let db = create_test_db();

    let mut group = GroupBuilder::new("group")
        .with_env("ROOT_VAR", "root")
        .with_env("SHARED", "from_root")
        .build();
    group.working_directory = Some("/tmp".to_string());
    let root_id = db.create_group(&group).expect("root group");

    let child_id = db
        .create_group(
            &GroupBuilder::new("child")
                .with_env("SHARED", "from_child")
                .with_parent(root_id)
                .build(),
        )
        .expect("child");

    let cmd_id = db
        .create_command(
            &CommandBuilder::new("test", "echo test")
                .with_group(child_id)
                .build(),
        )
        .unwrap();

    let pm = ProcessManager::new(db, None);
    let ctx = pm.resolve_spawn_context(cmd_id).await.expect("resolve");

    assert!(
        ctx.env_vars
            .iter()
            .any(|(k, v)| k == "ROOT_VAR" && v == "root"),
        "root var missing"
    );
    assert!(
        ctx.env_vars
            .iter()
            .any(|(k, v)| k == "SHARED" && v == "from_child"),
        "child should override root"
    );
    assert_eq!(
        ctx.env_vars.iter().filter(|(k, _)| k == "SHARED").count(),
        1,
        "SHARED should appear once"
    );
}
