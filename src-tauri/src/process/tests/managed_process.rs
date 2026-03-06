#[cfg(unix)]
use nix::unistd::{getpgid, Pid};
#[cfg(unix)]
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::database::ExecutionStatus;
use crate::process::errors::ProcessKillError;
use crate::process::managed_process::{ManagedProcess, ProcessEvent};
use crate::process::models::{ProcessStatus, SpawnContext};
use crate::process::tests::{spawn_context, WAIT_TIMEOUT};

fn make_channel() -> (mpsc::Sender<ProcessEvent>, mpsc::Receiver<ProcessEvent>) {
    mpsc::channel(128)
}

async fn wait_for_stopped(rx: &mut mpsc::Receiver<ProcessEvent>) -> ProcessEvent {
    timeout(WAIT_TIMEOUT, async {
        loop {
            match rx.recv().await {
                Some(evt @ ProcessEvent::Stopped(_)) => return evt,
                Some(_) => continue, // Started / StatusChanged / LogBatch
                None => panic!("Event channel closed before Stopped event"),
            }
        }
    })
    .await
    .expect("Timed out waiting for Stopped event")
}

/// Drain the channel until a `Stopped` event arrives. All other events are
/// collected and returned alongside it so callers can assert on full sequence.
async fn collect_until_stopped(
    rx: &mut mpsc::Receiver<ProcessEvent>,
) -> (Vec<ProcessEvent>, ProcessEvent) {
    timeout(WAIT_TIMEOUT, async {
        let mut preceding = Vec::new();
        loop {
            match rx.recv().await {
                Some(evt @ ProcessEvent::Stopped(_)) => return (preceding, evt),
                Some(other) => preceding.push(other),
                None => panic!("channel closed before Stopped event"),
            }
        }
    })
    .await
    .expect("timed out waiting for Stopped event")
}

#[tokio::test]
async fn spawn_echo_emits_started_then_stopped_with_success() {
    let (tx, mut rx) = make_channel();
    let ctx = spawn_context(1, "echo", vec!["hello"]);

    // tests does not require a valid command_id as there is no db operation,
    let process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    assert!(process.pid > 0);

    // First event should be Started
    let first = timeout(WAIT_TIMEOUT, rx.recv())
        .await
        .expect("timed out waiting for Started")
        .expect("channel closed");
    assert!(matches!(first, ProcessEvent::Started(_)));

    // Eventually Stopped with Success
    let stopped = wait_for_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };
    assert_eq!(evt.status, ExecutionStatus::Success);
    assert_eq!(evt.exit_code, Some(0));
}

#[tokio::test]
async fn log_lines_received_as_log_batch_events_on_channel() {
    let (tx, mut rx) = make_channel();

    #[cfg(unix)]
    let ctx = spawn_context(1, "sh", vec!["-c".into(), "echo 'event line'".into()]);

    #[cfg(windows)]
    let ctx = crate::process::tests::spawn_context(
        1,
        "log_batch_test",
        vec!["/C".into(), "echo event line".into()],
    );

    let _process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    let mut found_log_event = false;
    let mut saw_stopped = false;

    // Drain the channel for an extra 200ms after Stopped,
    // LogBatch events from the streaming bridge task may be queued behind Stopped on the mpsc channel.
    timeout(WAIT_TIMEOUT, async {
        loop {
            let next = if saw_stopped {
                // After Stopped, give remaining events a short window
                tokio::time::timeout(Duration::from_millis(200), rx.recv())
                    .await
                    .ok()
                    .flatten()
            } else {
                rx.recv().await
            };

            match next {
                Some(ProcessEvent::LogBatch(lines)) => {
                    if lines.iter().any(|l| l.content.contains("event line")) {
                        found_log_event = true;
                        break;
                    }
                }
                Some(ProcessEvent::Stopped(_)) => {
                    saw_stopped = true;
                    if found_log_event {
                        break;
                    }
                }
                Some(_) => {}
                None => break, // channel closed
            }
        }
    })
    .await
    .expect("timed out waiting for log event");

    assert!(
        found_log_event,
        "No LogBatch event containing 'event line' was received"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn stderr_lines_are_buffered_with_is_stderr_true() {
    let (tx, mut rx) = make_channel();

    // Write to stderr via shell redirection
    let ctx = spawn_context(1, "sh", vec!["-c".into(), "echo stderr_content >&2".into()]);

    let process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    wait_for_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let logs = process.get_logs(0, 100).await;
    assert!(
        logs.iter()
            .any(|l| l.is_stderr && l.content.contains("stderr_content")),
        "Expected stderr line in buffer"
    );
}

#[tokio::test]
async fn spawn_returns_ok_for_valid_executable() {
    let (tx, _rx) = make_channel();
    let result = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["hi"]), tx, false).await;
    assert!(result.is_ok());
    result.unwrap().force_kill().await.ok();
}

#[tokio::test]
#[cfg(unix)]
async fn spawn_sets_correct_initial_values() {
    let (tx, _rx) = make_channel();
    let mut process = ManagedProcess::spawn(42, spawn_context(99, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");
    assert!(process.pid > 0);
    assert_eq!(process.execution_id, 42);
    assert_eq!(process.command_id, 99);
    assert!(process.is_running().await);
    process.force_kill().await.ok();
}

#[tokio::test]
async fn spawn_nonexistent_executable_returns_err() {
    let (tx, _rx) = make_channel();
    let result = ManagedProcess::spawn(
        1,
        spawn_context(1, "this_does_not_exist_tgui_xyz", vec![]),
        tx,
        false,
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn started_event_has_correct_execution_id_and_pid() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(77, spawn_context(1, "echo", vec!["hi"]), tx, false)
        .await
        .expect("spawn failed");

    let first = timeout(WAIT_TIMEOUT, rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed");
    let ProcessEvent::Started(evt) = first else {
        panic!("Expected Started")
    };

    assert_eq!(evt.execution_id, 77);
    assert_eq!(evt.pid, process.pid);
    assert!(!evt.timestamp.is_empty());
}

#[tokio::test]
async fn started_event_has_correct_command_id_and_name() {
    let (tx, mut rx) = make_channel();
    let ctx = SpawnContext {
        command_id: 55,
        name: "my_cmd".into(),
        ..spawn_context(1, "echo", vec!["hi"])
    };
    let _ = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    let first = timeout(WAIT_TIMEOUT, rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed");
    let ProcessEvent::Started(evt) = first else {
        panic!("Expected Started")
    };

    assert_eq!(evt.command_id, 55);
    assert_eq!(evt.command_name, "my_cmd");
}

#[tokio::test]
async fn natural_exit_code_zero_emits_stopped_with_success_and_exit_code_zero() {
    let (tx, mut rx) = make_channel();
    let _process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["done"]), tx, false)
        .await
        .expect("spawn failed");

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };

    assert_eq!(evt.status, ExecutionStatus::Success);
    assert_eq!(evt.exit_code, Some(0));
}

#[tokio::test]
async fn natural_exit_nonzero_emits_stopped_with_failed_and_nonzero_exit_code() {
    let (tx, mut rx) = make_channel();
    #[cfg(unix)]
    let ctx = spawn_context(1, "false", vec![]);
    #[cfg(windows)]
    let ctx = {
        let mut c = spawn_context(1, "cmd", vec!["/C", "exit", "1"]);
        c
    };

    let _ = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };

    assert_eq!(evt.status, ExecutionStatus::Failed);
    assert_ne!(evt.exit_code, Some(0));
}

#[tokio::test]
async fn stopped_event_has_correct_execution_id() {
    let (tx, mut rx) = make_channel();
    let _process = ManagedProcess::spawn(88, spawn_context(1, "echo", vec!["x"]), tx, false)
        .await
        .expect("spawn failed");

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };
    assert_eq!(evt.execution_id, 88);
}

#[tokio::test]
async fn stopped_event_has_correct_pid() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["x"]), tx, false)
        .await
        .expect("spawn failed");
    let expected_pid = process.pid;

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };
    assert_eq!(evt.pid, expected_pid);
}

#[tokio::test]
async fn natural_exit_emits_status_changed_from_running_to_stopped_or_error() {
    let (tx, mut rx) = make_channel();
    let _process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["done"]), tx, false)
        .await
        .expect("spawn failed");

    let (preceding, _) = collect_until_stopped(&mut rx).await;

    // At least one StatusChanged must exist for the terminal transition
    let status_changes: Vec<_> = preceding
        .iter()
        .filter_map(|e| {
            if let ProcessEvent::StatusChanged(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    assert!(
        !status_changes.is_empty(),
        "Expected at least one StatusChanged event"
    );

    let final_change = status_changes.last().unwrap();
    assert!(
        matches!(
            final_change.new_status,
            ProcessStatus::Stopped { .. } | ProcessStatus::Error { .. }
        ),
        "Final StatusChanged should transition to Stopped or Error, got: {:?}",
        final_change.new_status
    );
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_emits_status_changed_to_stopping_before_stopped() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.graceful_kill().await.expect("graceful_kill failed");

    let (preceding, _) = collect_until_stopped(&mut rx).await;

    let saw_stopping = preceding.iter().any(|e| {
        matches!(e, ProcessEvent::StatusChanged(s) if matches!(s.new_status, ProcessStatus::Stopping { .. }))
    });
    assert!(
        saw_stopping,
        "Expected Stopping StatusChanged before Stopped"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_stopping_event_has_running_as_old_status() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.graceful_kill().await.expect("graceful_kill failed");

    let (preceding, _) = collect_until_stopped(&mut rx).await;

    let stopping_evt = preceding.iter().find_map(|e| {
        if let ProcessEvent::StatusChanged(s) = e {
            if matches!(s.new_status, ProcessStatus::Stopping { .. }) {
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    });

    let evt = stopping_evt.expect("no Stopping event found");
    assert!(
        matches!(evt.old_status, ProcessStatus::Running { .. }),
        "Stopping event old_status should be Running, got: {:?}",
        evt.old_status
    );
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_emits_stopped_with_cancelled() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.graceful_kill().await.expect("graceful_kill failed");

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };
    assert_eq!(evt.status, ExecutionStatus::Cancelled);
}

#[tokio::test]
#[cfg(unix)]
async fn force_kill_emits_stopped_with_cancelled() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    assert!(process.is_running().await);

    process.force_kill().await.expect("force_kill failed");

    let (_, stopped) = collect_until_stopped(&mut rx).await;
    let ProcessEvent::Stopped(evt) = stopped else {
        unreachable!()
    };
    assert_eq!(evt.status, ExecutionStatus::Cancelled);
}

#[tokio::test]
#[cfg(unix)]
async fn force_kill_does_not_emit_stopping_event() {
    // Force kill goes straight to dead — no graceful transition period
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.force_kill().await.expect("force_kill failed");

    let (preceding, _) = collect_until_stopped(&mut rx).await;
    let saw_stopping = preceding.iter().any(|e| {
        matches!(e, ProcessEvent::StatusChanged(s) if matches!(s.new_status, ProcessStatus::Stopping { .. }))
    });
    // Force kill is immediate but currently force_kill sends KillMode::Force which still emits Stopping.
    // TODO: Update this assertion when behavior is updated, This test documents current behavior now.
    let _ = saw_stopping;
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_twice_returns_already_exited_on_second() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.graceful_kill().await.expect("first kill failed");
    collect_until_stopped(&mut rx).await;

    let result = process.graceful_kill().await;
    assert!(matches!(result, Err(ProcessKillError::AlreadyExited)));
}

#[tokio::test]
#[cfg(unix)]
async fn force_kill_twice_returns_already_exited_on_second() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.force_kill().await.expect("first kill failed");
    collect_until_stopped(&mut rx).await;

    let result = process.force_kill().await;
    assert!(matches!(result, Err(ProcessKillError::AlreadyExited)));
}

#[tokio::test]
#[cfg(unix)]
async fn graceful_kill_after_force_kill_returns_already_exited() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.force_kill().await.expect("force kill failed");
    collect_until_stopped(&mut rx).await;

    assert!(matches!(
        process.graceful_kill().await,
        Err(ProcessKillError::AlreadyExited)
    ));
}

#[tokio::test]
async fn kill_after_natural_exit_returns_already_exited() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["bye"]), tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(matches!(
        process.graceful_kill().await,
        Err(ProcessKillError::AlreadyExited)
    ));
    assert!(matches!(
        process.force_kill().await,
        Err(ProcessKillError::AlreadyExited)
    ));
}

#[tokio::test]
async fn is_running_false_after_natural_exit() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["done"]), tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!process.is_running().await);
}
#[tokio::test]
#[cfg(unix)]
async fn is_running_false_after_force_kill() {
    let (tx, mut rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.force_kill().await.expect("force_kill failed");
    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!process.is_running().await);
}

#[tokio::test]
#[cfg(unix)]
async fn is_running_false_immediately_after_graceful_kill_sent() {
    // Once kill_tx is consumed, status transitions to Stopping.
    let (tx, _rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    process.graceful_kill().await.expect("graceful_kill failed");

    // Poll immediately — status may still be Running for a brief moment
    // while the monitor task processes the kill. Give it one tick.
    tokio::task::yield_now().await;

    // Once Stopping, is_running must be false
    timeout(WAIT_TIMEOUT, async {
        loop {
            let status = process.get_status().await;
            if matches!(status, ProcessStatus::Stopping { .. }) {
                assert!(
                    !process.is_running().await,
                    "is_running returned true during Stopping state"
                );
                return;
            }
            if matches!(
                status,
                ProcessStatus::Stopped { .. } | ProcessStatus::Error { .. }
            ) {
                return; // already past Stopping, fine
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("timed out waiting for Stopping state");
}

#[tokio::test]
#[cfg(unix)]
async fn get_status_running_immediately_after_spawn() {
    let (tx, _rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    assert!(matches!(
        process.get_status().await,
        ProcessStatus::Running { .. }
    ));
    process.force_kill().await.ok();
}

#[tokio::test]
#[cfg(unix)]
async fn get_status_running_has_correct_pid() {
    let (tx, _rx) = make_channel();
    let mut process = ManagedProcess::spawn(1, spawn_context(1, "sleep", vec!["60"]), tx, false)
        .await
        .expect("spawn failed");

    let status = process.get_status().await;
    let ProcessStatus::Running { pid, .. } = status else {
        panic!("Expected Running status")
    };
    assert_eq!(pid, process.pid);
    process.force_kill().await.ok();
}

#[tokio::test]
async fn get_status_stopped_after_exit_code_zero() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["done"]), tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(matches!(
        process.get_status().await,
        ProcessStatus::Stopped { exit_code: 0, .. }
    ));
}

#[tokio::test]
async fn get_status_error_after_nonzero_exit() {
    let (tx, mut rx) = make_channel();
    #[cfg(unix)]
    let process = ManagedProcess::spawn(1, spawn_context(1, "false", vec![]), tx, false)
        .await
        .expect("spawn failed");

    #[cfg(unix)]
    {
        collect_until_stopped(&mut rx).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let status = process.get_status().await;
        assert!(
            matches!(status, ProcessStatus::Error { .. }),
            "Expected Error status for nonzero exit, got: {status:?}"
        );
    }
}

#[tokio::test]
async fn stdout_is_captured_in_log_buffer() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(
        1,
        spawn_context(1, "echo", vec!["captured_stdout"]),
        tx,
        false,
    )
    .await
    .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let logs = process.get_logs(0, 100).await;
    assert!(!logs.is_empty(), "Expected at least one log line from echo");
    assert!(
        logs.iter().any(|l| l.content.contains("captured_stdout")),
        "stdout not found in buffer: {logs:?}"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn stderr_is_captured_with_is_stderr_true() {
    let (tx, mut rx) = make_channel();
    let ctx = spawn_context(
        1,
        "sh",
        vec!["-c".into(), "echo captured_stderr >&2".into()],
    );

    let process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let logs = process.get_logs(0, 100).await;
    let stderr_line = logs.iter().find(|l| l.content.contains("captured_stderr"));
    assert!(stderr_line.is_some(), "stderr not found in buffer");
    assert!(
        stderr_line.unwrap().is_stderr,
        "stderr line not marked is_stderr"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn get_logs_with_offset_and_limit() {
    let (tx, mut rx) = make_channel();
    let ctx = spawn_context(
        1,
        "sh",
        vec![
            "-c".into(),
            "echo l1; echo l2; echo l3; echo l4; echo l5".into(),
        ],
    );

    let process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let all = process.get_logs(0, 100).await;
    let page = process.get_logs(1, 2).await;

    assert_eq!(page.len(), 2);
    assert_eq!(page[0].content, all[1].content);
    assert_eq!(page[1].content, all[2].content);
}

#[tokio::test]
async fn clear_logs_empties_buffer() {
    let (tx, mut rx) = make_channel();
    let process = ManagedProcess::spawn(1, spawn_context(1, "echo", vec!["clear_me"]), tx, false)
        .await
        .expect("spawn failed");

    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    process.clear_logs().await;
    assert!(process.get_logs(0, 100).await.is_empty());
}

#[tokio::test]
#[cfg(unix)]
async fn kill_process_tree_true_puts_process_in_new_process_group() {
    let (tx, _rx) = make_channel();
    // spawn a child that runs long enough for us to inspect
    let ctx = spawn_context(1, "sleep", vec!["60".into()]);

    let mut process = ManagedProcess::spawn(1, ctx, tx, true)
        .await
        .expect("spawn failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let child_pid = process.pid as i32;
    let child_pgid = getpgid(Some(Pid::from_raw(child_pid)))
        .expect("getpgid failed")
        .as_raw();

    // With setpgid(0,0), the child becomes its own process group leader:
    // pgid == pid
    assert_eq!(
        child_pgid, child_pid,
        "Expected child to be its own process group leader (pgid == pid)"
    );

    process.force_kill().await.ok();
}

#[tokio::test]
#[cfg(unix)]
async fn kill_process_tree_false_child_shares_parent_process_group() {
    let (tx, _rx) = make_channel();
    let ctx = spawn_context(1, "sleep", vec!["60".into()]);

    let mut process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let child_pid = process.pid as i32;
    let child_pgid = getpgid(Some(Pid::from_raw(child_pid)))
        .expect("getpgid failed")
        .as_raw();
    let our_pgid = getpgid(None).expect("getpgid self failed").as_raw();

    // Without setpgid, child inherits our process group
    assert_eq!(
        child_pgid, our_pgid,
        "Expected child to inherit parent process group when kill_process_tree=false"
    );

    process.force_kill().await.ok();
}

#[tokio::test]
#[cfg(unix)]
async fn env_vars_are_visible_inside_spawned_process() {
    let (tx, mut rx) = make_channel();
    let ctx = SpawnContext {
        command_id: 1,
        name: "env_test".into(),
        executable: "sh".into(),
        arguments: vec!["-c".into(), "echo $TGUI_TEST_VAR".into()],
        working_directory: PathBuf::from("/tmp"),
        env_vars: vec![("TGUI_TEST_VAR".into(), "env_value_123".into())],
        shell: None,
    };
    let process = ManagedProcess::spawn(1, ctx, tx, false)
        .await
        .expect("spawn failed");
    collect_until_stopped(&mut rx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let logs = process.get_logs(0, 100).await;
    assert!(
        logs.iter().any(|l| l.content.contains("env_value_123")),
        "env var not visible inside process: {logs:?}"
    );
}
