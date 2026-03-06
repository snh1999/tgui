use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::process::managed_process::ProcessEvent;
use crate::process::models::{LogLineEvent, StreamingConfig};
use crate::process::streaming::LogStreamer;

const WAIT: Duration = Duration::from_secs(5);

/// Spawn a real child process and return its stdout/stderr handles.
/// We need real ChildStdout/ChildStderr because tokio's types wrap OS fds
/// and can't be constructed directly.
async fn spawn_child(
    cmd: &str,
    args: &[&str],
) -> (
    tokio::process::Child,
    Option<tokio::process::ChildStdout>,
    Option<tokio::process::ChildStderr>,
) {
    let mut child = tokio::process::Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn child");

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    (child, stdout, stderr)
}

/// Drain ProcessEvents from the bridge output until channel closes. Simulate read from process_rx
async fn drain_process_events(
    mut rx: tokio::sync::mpsc::Receiver<ProcessEvent>,
) -> Vec<Arc<LogLineEvent>> {
    let mut lines = Vec::new();
    timeout(WAIT, async {
        while let Some(batch) = rx.recv().await {
            if let ProcessEvent::LogBatch(batch) = batch {
                lines.extend(batch);
            }
        }
    })
    .await
    .expect("timed out draining process events");
    lines
}

fn immediate_config() -> StreamingConfig {
    StreamingConfig::immediate()
}

fn batching_config(batch_size: usize, timeout_ms: u64) -> StreamingConfig {
    StreamingConfig {
        buffer_capacity: 100,
        batch_size,
        batch_timeout_ms: timeout_ms,
        max_line_length: 10000,
    }
}

#[tokio::test]
async fn stdout_lines_are_received_as_batch_events() {
    let (mut child, stdout, stderr) = spawn_child("echo", &["hello from stdout"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert!(
        lines
            .iter()
            .any(|l| l.content.contains("hello from stdout")),
        "Expected stdout content in events, got: {lines:?}"
    );
}

#[tokio::test]
async fn stdout_lines_have_is_stderr_false() {
    let (mut child, stdout, stderr) = spawn_child("echo", &["stdout_line"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    let line = lines
        .iter()
        .find(|l| l.content.contains("stdout_line"))
        .expect("stdout_line not found");
    assert!(!line.is_stderr, "stdout line incorrectly marked as stderr");
}

#[tokio::test]
async fn execution_id_timestamp_is_set_on_every_log_line() {
    let (mut child, stdout, stderr) = spawn_child("echo", &["id_test"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(10, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    for line in &lines {
        assert_eq!(
            line.execution_id, 10,
            "execution_id mismatch on line: {line:?}"
        );
        assert!(!line.timestamp.is_empty(), "timestamp should not be empty");
    }
}

#[tokio::test]
#[cfg(unix)]
async fn stderr_lines_have_is_stderr_true() {
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", "echo stderr_content >&2"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    let line = lines
        .iter()
        .find(|l| l.content.contains("stderr_content"))
        .expect("stderr_content not found in events");
    assert!(line.is_stderr, "stderr line not marked as is_stderr");
}

#[tokio::test]
#[cfg(unix)]
async fn mixed_stdout_and_stderr_are_both_captured() {
    let (mut child, stdout, stderr) =
        spawn_child("sh", &["-c", "echo stdout_only; echo stderr_only >&2"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert!(
        lines
            .iter()
            .any(|l| !l.is_stderr && l.content.contains("stdout_only")),
        "stdout_only not found"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.is_stderr && l.content.contains("stderr_only")),
        "stderr_only not found"
    );
}

/// Test logStreamer maintains two copies of log data simultaneously
#[tokio::test]
async fn log_lines_are_written_to_buffer_and_emitted() {
    let (mut child, stdout, stderr) = spawn_child("echo", &["buffered_line"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, buffer) = LogStreamer::new(1, 1, immediate_config());
    let _handle = streamer.start(stdout, stderr, tx);
    // Drain events so streamer task can complete
    let _ = drain_process_events(rx).await;
    let _ = child.wait().await;

    // Give buffer write a moment
    tokio::time::sleep(Duration::from_millis(50)).await;

    let buf = buffer.read().await;
    assert!(
        buf.get_all()
            .iter()
            .any(|l| l.content.contains("buffered_line")),
        "Expected buffered_line in LogBuffer"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn buffer_and_events_contain_same_lines() {
    let (mut child, stdout, stderr) =
        spawn_child("sh", &["-c", "echo line_a; echo line_b; echo line_c"]).await;

    let (streamer, buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(stdout, stderr, tx);
    let event_lines = drain_process_events(rx).await;
    let _ = child.wait().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let buf_lines = buffer.read().await.get_all();

    for label in &["line_a", "line_b", "line_c"] {
        assert!(
            event_lines.iter().any(|l| l.content.contains(label)),
            "{label} missing from events"
        );
        assert!(
            buf_lines.iter().any(|l| l.content.contains(label)),
            "{label} missing from buffer"
        );
    }
}

#[tokio::test]
#[cfg(unix)]
async fn batch_emits_when_size_threshold_reached() {
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", "echo a; echo b; echo c"]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let config = batching_config(3, 5000); // high timeout so only size triggers
    let (streamer, _buffer) = LogStreamer::new(1, 1, config);
    let _ = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert_eq!(
        lines.len(),
        3,
        "Expected 3 lines total, got {}",
        lines.len()
    );
    assert!(lines.iter().any(|l| l.content.contains('a')));
    assert!(lines.iter().any(|l| l.content.contains('b')));
    assert!(lines.iter().any(|l| l.content.contains('c')));
}

#[tokio::test]
#[cfg(unix)]
async fn batch_emits_via_timeout_when_below_size_threshold() {
    let (mut child, stdout, stderr) = spawn_child("echo", &["timeout_line"]).await;

    let config = batching_config(100, 50); // small timeout

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _) = LogStreamer::new(1, 1, config);
    let _ = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert!(
        lines.iter().any(|l| l.content.contains("timeout_line")),
        "Expected timeout_line to be emitted via timeout, got: {lines:?}"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn all_batch_is_flushed_on_stream_close() {
    // all lines must flush via the else/EOF branch
    let (mut child, stdout, stderr) =
        spawn_child("sh", &["-c", "echo flush_a; echo flush_b"]).await;

    let config = batching_config(100, 30000); // 30s timeout, definitely won't fire
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, config);
    let _handle = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert!(
        lines.iter().any(|l| l.content.contains("flush_a")),
        "flush_a missing"
    );
    assert!(
        lines.iter().any(|l| l.content.contains("flush_b")),
        "flush_b missing"
    );
}

#[tokio::test]
#[cfg(unix)]
async fn immediate_config_emits_each_line_as_its_own_batch() {
    let (mut child, stdout, stderr) =
        spawn_child("sh", &["-c", "echo one; echo two; echo three"]).await;

    let (streamer, _) = LogStreamer::new(1, 1, immediate_config());
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(stdout, stderr, tx);

    let mut batch_count = 0;
    let mut lines = Vec::new();
    timeout(WAIT, async {
        while let Some(event) = rx.recv().await {
            if let ProcessEvent::LogBatch(batch) = event {
                batch_count += 1;
                lines.extend(batch);
            }
        }
    })
    .await
    .expect("timed out");
    let _ = child.wait().await;

    assert!(
        batch_count >= 3,
        "Expected at least 3 batch events (one per line), got {batch_count}"
    );
    assert_eq!(lines.len(), 3);
}

#[tokio::test]
#[cfg(unix)]
async fn lines_exceeding_max_length_are_truncated_with_marker() {
    let long_line = "a".repeat(20);
    let cmd = format!("echo {long_line}");
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", &cmd]).await;

    let config = StreamingConfig {
        buffer_capacity: 1000,
        batch_size: 1,
        batch_timeout_ms: 0,
        max_line_length: 10,
    };
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _buffer) = LogStreamer::new(1, 1, config);
    let _ = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    let line = lines
        .iter()
        .find(|l| l.content.contains("...[truncated]"))
        .expect("Expected truncated line with marker");
    assert!(
        line.content.len() < 20 + 20, // 20 original chars → truncated + marker
        "Line should be shorter than original: {}",
        line.content
    );
}

#[tokio::test]
async fn process_with_no_output_produces_no_events() {
    #[cfg(unix)]
    let (mut child, stdout, stderr) = spawn_child("true", &[]).await;
    #[cfg(windows)]
    let (mut child, stdout, stderr) = spawn_child("cmd", &["/C", "exit 0"]).await;

    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);

    let _ = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    assert!(
        lines.is_empty(),
        "Expected no events for process with no output"
    );
}

#[tokio::test]
async fn no_stdout_no_stderr_handles_correctly() {
    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(None, None, tx);

    let lines = timeout(WAIT, async {
        let mut l = Vec::new();
        let mut rx = rx;

        while let Some(event) = rx.recv().await {
            if let ProcessEvent::LogBatch(batch) = event {
                l.extend(batch);
            }
        }
        l
    })
    .await
    .expect("timed out — streamer did not exit with None streams");

    assert!(lines.is_empty());
}

#[tokio::test]
async fn spawn_log_streaming_populates_buffer_and_forwards_events() {
    let (process_tx, process_rx) = tokio::sync::mpsc::channel(128);

    let (mut child, stdout, stderr) = spawn_child("echo", &["both_paths"]).await;
    let pid = child.id().unwrap_or(1);

    let (buffer, _handle) =
        LogStreamer::spawn_log_streaming(1, pid, stdout, stderr, immediate_config(), process_tx);

    let event_lines = drain_process_events(process_rx).await;
    let _ = child.wait().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let buf_lines = buffer.read().await.get_all();

    // Both paths must have the line
    assert!(
        event_lines.iter().any(|l| l.content.contains("both_paths")),
        "Missing from events"
    );
    assert!(
        buf_lines.iter().any(|l| l.content.contains("both_paths")),
        "Missing from buffer"
    );
}

#[tokio::test]
async fn spawn_log_streaming_bridge_exits_cleanly_when_process_exits() {
    let (process_tx, process_rx) = tokio::sync::mpsc::channel(128);

    let (mut child, stdout, stderr) = spawn_child("echo", &["exit_test"]).await;
    let pid = child.id().unwrap_or(1);

    let (_buffer, handle) =
        LogStreamer::spawn_log_streaming(1, pid, stdout, stderr, immediate_config(), process_tx);

    let _ = child.wait().await;
    // Drop the receiver to close the downstream channel
    drop(process_rx);

    timeout(WAIT, handle)
        .await
        .expect("bridge task timed out — did not exit cleanly")
        .expect("bridge task panicked");
}

#[tokio::test]
async fn spawn_log_streaming_bridge_exits_when_event_sender_is_dropped() {
    // If ProcessManager is dropped, event_sender closes (bridge task should stop rather than panic).
    let (process_tx, process_rx) = tokio::sync::mpsc::channel(1);

    #[cfg(unix)]
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", "sleep 1; echo done"]).await;
    #[cfg(windows)]
    let (mut child, stdout, stderr) = spawn_child("cmd", &["/C", "timeout 1 & echo done"]).await;

    let pid = child.id().unwrap_or(1);

    let (_buffer, handle) =
        LogStreamer::spawn_log_streaming(1, pid, stdout, stderr, immediate_config(), process_tx);

    // Drop receiver immediately — sender will error on next send
    drop(process_rx);

    // Bridge should detect and exit cleanly (is_err silenced inside bridge)
    timeout(WAIT, handle)
        .await
        .expect("bridge did not exit after receiver dropped")
        .expect("bridge panicked");

    let _ = child.kill().await;
}

#[tokio::test]
#[cfg(unix)]
async fn log_lines_maintain_chronological_order_within_stream() {
    let (mut child, stdout, stderr) = spawn_child(
        "sh",
        &[
            "-c",
            "echo first; echo second; echo third; echo fourth; echo fifth",
        ],
    )
    .await;

    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _ = streamer.start(stdout, stderr, tx);

    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    let stdout_lines: Vec<&str> = lines
        .iter()
        .filter(|l| !l.is_stderr)
        .map(|l| l.content.as_str())
        .collect();

    let order = ["first", "second", "third", "fourth", "fifth"];
    let mut last_pos = 0;
    for expected in &order {
        let pos = stdout_lines
            .iter()
            .position(|l| l.contains(expected))
            .unwrap_or_else(|| panic!("{expected} not found in output"));
        assert!(
            pos >= last_pos,
            "{expected} appeared before previous line (pos {pos} < {last_pos})"
        );
        last_pos = pos;
    }
}

#[tokio::test]
#[cfg(unix)]
async fn rapid_streams_capture_all() {
    // Rapidly produce many interleaved lines
    let script = (1..=20)
        .map(|i| format!("echo line{i}; echo err{i} >&2"))
        .collect::<Vec<_>>()
        .join(";");
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", &script]).await;

    let config = batching_config(10, 100); // Batch size 10, short timeout
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, buffer) = LogStreamer::new(1, 1, config);
    let _ = streamer.start(stdout, stderr, tx);
    let event_lines = drain_process_events(rx).await;

    let _ = child.wait().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let buf_lines = buffer.read().await.get_all();

    assert_eq!(
        event_lines.len(),
        40,
        "Expected 40 lines, got {}",
        event_lines.len()
    );
    assert_eq!(buf_lines.len(), 40, "Buffer should also have 40 lines");

    // Verify all lines present in both
    for i in 1..=20 {
        assert!(
            event_lines
                .iter()
                .any(|l| l.content.contains(&format!("line{i}"))),
            "line{i} missing from events"
        );
        assert!(
            event_lines
                .iter()
                .any(|l| l.content.contains(&format!("err{i}")) && l.is_stderr),
            "err{i} missing or not marked as stderr"
        );
    }
}

#[tokio::test]
#[cfg(unix)]
async fn large_volume_output_no_lines_lost() {
    let script = (1..=5000)
        .map(|i| format!("echo 'line_number_{i:04}'"))
        .collect::<Vec<_>>()
        .join(";");
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", &script]).await;

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, buffer) = LogStreamer::new(1, 1, immediate_config());
    let _ = streamer.start(stdout, stderr, tx);
    let event_lines = drain_process_events(rx).await;

    let _ = child.wait().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let buf_lines = buffer.read().await.get_all();

    assert_eq!(
        event_lines.len(),
        5000,
        "Expected 500 event lines, got {}",
        event_lines.len()
    );
    assert_eq!(
        buf_lines.len(),
        5000,
        "Expected 500 buffer lines, got {}",
        buf_lines.len()
    );

    assert!(event_lines
        .iter()
        .any(|l| l.content.contains("line_number_0001")));
    assert!(event_lines
        .iter()
        .any(|l| l.content.contains("line_number_0250")));
    assert!(event_lines
        .iter()
        .any(|l| l.content.contains("line_number_0500")));
}

#[tokio::test]
#[cfg(unix)]
async fn unicode_multibyte_at_truncation_boundary() {
    let emoji_line = "🎉".repeat(10); // 40 bytes
    let cmd = format!("echo '{}'", emoji_line);
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", &cmd]).await;

    let config = StreamingConfig {
        buffer_capacity: 1000,
        batch_size: 1,
        batch_timeout_ms: 0,
        max_line_length: 15, // Cuts into middle of emoji sequence
    };

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, _) = LogStreamer::new(1, 1, config);
    let _handle = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;

    let _ = child.wait().await;

    assert!(!lines.is_empty(), "Should have at least one line");
    let line = &lines[0];
    assert!(
        line.content.contains("...[truncated]"),
        "Should have truncation marker"
    );
    // Verify it's valid UTF-8 (no replacement chars)
    assert!(!line.content.contains('\u{FFFD}'), "Should be valid UTF-8");
}

#[tokio::test]
async fn empty_lines_are_preserved() {
    #[cfg(unix)]
    let (mut child, stdout, stderr) = spawn_child("printf", &["%s\n\n", "marker"]).await;
    #[cfg(windows)]
    let (mut child, stdout, stderr) =
        spawn_child("cmd", &["/C", "echo.", "&", "echo marker"]).await;

    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;
    let _ = child.wait().await;

    // Should have empty line and marker
    let empty_count = lines.iter().filter(|l| l.content.trim().is_empty()).count();
    assert!(
        empty_count >= 1,
        "Should have at least one empty line, got: {lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.content.contains("marker")),
        "marker should exist"
    );
}

#[tokio::test]
async fn buffer_capacity_smaller_than_batch_size() {
    // If buffer can only hold 5, but batch_size is 10, what happens?
    let (mut child, stdout, stderr) =
        spawn_child("sh", &["-c", "for i in $(seq 1 20); do echo $i; done"]).await;

    let config = StreamingConfig {
        buffer_capacity: 5,
        batch_size: 10,
        batch_timeout_ms: 50,
        max_line_length: 10000,
    };

    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let (streamer, buffer) = LogStreamer::new(1, 1, config);
    let _handle = streamer.start(stdout, stderr, tx);
    let event_lines = drain_process_events(rx).await;
    let _ = child.wait().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let buf_lines = buffer.read().await.get_all();

    // Buffer should only have 5 most recent lines due to circular behavior
    assert_eq!(buf_lines.len(), 5, "Buffer should be capped at capacity");
    // But events should have all 20
    assert_eq!(event_lines.len(), 20, "Events should have all lines");
}

#[tokio::test]
#[cfg(unix)]
async fn streamer_handles_broken_pipe_gracefully() {
    // Child closes stdout prematurely
    let (mut child, stdout, stderr) = spawn_child(
        "sh",
        &["-c", "echo before; exec 1>&-; sleep 0.1; echo after >&2"],
    )
    .await;

    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;

    let _ = child.wait().await;

    // Should have "before" and "after" (stderr), no crash
    assert!(lines.iter().any(|l| l.content.contains("before")));
    assert!(lines
        .iter()
        .any(|l| l.content.contains("after") && l.is_stderr));
}

#[tokio::test]
#[cfg(unix)]
async fn very_long_output_single_line() {
    let big_line = "x".repeat(100_000);
    let cmd = format!("echo '{}'", big_line);
    let (mut child, stdout, stderr) = spawn_child("sh", &["-c", &cmd]).await;

    let (streamer, _buffer) = LogStreamer::new(1, 1, immediate_config());
    let (tx, rx) = tokio::sync::mpsc::channel(128);
    let _handle = streamer.start(stdout, stderr, tx);
    let lines = drain_process_events(rx).await;

    let _ = child.wait().await;

    assert!(lines[0].content.contains("...[truncated]"));
    assert!(lines[0].content.len() > 10_000); // Includes marker
    assert!(lines[0].content.len() < 100_000);
}
