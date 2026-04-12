use crate::database::ExecutionStatus;
use crate::process::errors::{ProcessKillError, ProcessSpawnError};
use crate::process::log_buffer::LogBuffer;
use crate::process::models::{
    KillMode, LogLineEvent, ProcessStartedEvent, ProcessStatus, ProcessStatusChangedEvent,
    ProcessStoppedEvent, SpawnContext, StreamingConfig,
};
use crate::process::shell;
use crate::process::signals::ProcessHandle;
use crate::process::streaming::LogStreamer;
use crate::utils::get_utc_timestamp_string;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};

pub struct ManagedProcess {
    pub execution_id: i64,
    pub command_id: i64,
    pub command_name: String,
    pub pid: u32,
    pub status: Arc<RwLock<ProcessStatus>>,
    pub log_buffer: Arc<RwLock<LogBuffer>>,
    pub start_time: String,
    pub context: SpawnContext,
    kill_tx: Option<oneshot::Sender<KillMode>>,
}

#[derive(Debug)]
pub enum ProcessEvent {
    Started(ProcessStartedEvent),
    StatusChanged(ProcessStatusChangedEvent),
    Stopped(ProcessStoppedEvent),
    LogBatch(Vec<Arc<LogLineEvent>>),
}

impl ManagedProcess {
    pub async fn spawn(
        execution_id: i64,
        context: SpawnContext,
        event_sender: mpsc::Sender<ProcessEvent>,
        kill_process_tree: bool,
    ) -> Result<Self, ProcessSpawnError> {
        debug!(
            execution_id,
            command_id = context.command_id,
            executable = %context.executable,
            "Spawning process"
        );

        // build command via shell resolver
        let result = shell::build_exec(
            &context.executable,
            &context.arguments,
            context.shell.as_deref(),
        );

        let mut cmd = Command::new(&result.executable);
        cmd.args(&result.args);

        cmd.current_dir(&context.working_directory);
        for (key, value) in &context.env_vars {
            cmd.env(key, value);
        }

        // configure stdio
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        // Process group (for tree killing)
        #[cfg(unix)]
        {
            use libc::{prctl, PR_SET_PDEATHSIG, SIGKILL};
            unsafe {
                cmd.pre_exec(move || {
                    if kill_process_tree {
                        libc::setpgid(0, 0);
                    }
                    // If parent (TGUI) dies, kernel sends SIGKILL to this process immediately
                    let ret = prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0);
                    if ret != 0 {
                        // pre_exec runs in the forked child — return Error (tokio will surface as ProcessSpawnError)
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }

        // spawn child
        let mut child = cmd.spawn().map_err(ProcessSpawnError::SpawnFailed)?;
        let pid = child.id().ok_or(ProcessSpawnError::NoPid)?;
        let start_time = get_utc_timestamp_string();

        info!(execution_id, pid, command = %context.name, "Process spawned successfully");

        // Setup kill channel
        let (kill_tx, kill_rx) = oneshot::channel::<KillMode>();

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Wire event_sender into streaming so logs reach ProcessManager
        let (log_buffer, streaming_handle) = LogStreamer::spawn_log_streaming(
            execution_id,
            pid,
            stdout,
            stderr,
            StreamingConfig::default(),
            event_sender.clone(),
        );

        let status = Arc::new(RwLock::new(ProcessStatus::Running {
            pid,
            start_time: start_time.clone(),
        }));

        // Clone for monitor task
        let status_clone = status.clone();
        let event_sender_clone = event_sender.clone();

        // Sending has to be here (before spawn) to ensuse the order started → log → stopped for fast exiting processes
        // As it is already in the mpsc channel queue when the monitor task starts, so the consumer always sees it first.
        // The logs→stopped invariant is preserved by `streaming_handle.await` inside the monitor task.
        if let Err(e) = event_sender
            .send(ProcessEvent::Started(ProcessStartedEvent {
                execution_id,
                pid,
                command_id: context.command_id,
                command_name: context.name.clone(),
                timestamp: start_time.clone(),
            }))
            .await
        {
            warn!(error = ?e, "Failed to send Started event");
        }

        // Spawn monitor task that owns the child and kill_rx
        tokio::spawn(async move {
            let mut process_handle = ProcessHandle { pid, child };

            let mut was_killed = false;

            let exit_status = tokio::select! {
                // Natural exit
                status = process_handle.child.wait() => status,

                // Kill signal received

                Ok(mode) = kill_rx => {
                    was_killed = true;
                    debug!(execution_id, ?mode, "Kill signal received");

                    let stopping_status = ProcessStatus::Stopping {
                        since: get_utc_timestamp_string(),
                    };

                    let old_status = {
                        let mut s = status_clone.write().await;
                        let old = s.clone();
                        *s = stopping_status.clone();
                        old
                    };

                    let _ = event_sender_clone.send(ProcessEvent::StatusChanged(ProcessStatusChangedEvent {
                        execution_id,
                        old_status,
                        new_status: stopping_status,
                        timestamp: get_utc_timestamp_string(),
                    })).await;

                    match mode {
                        KillMode::Graceful => {
                            // Send SIGTERM
                            if let Err(e) = process_handle.graceful_kill().await {
                                error!(error = %e, "Graceful kill failed");
                            }

                            // Wait 5s for graceful exit
                            match process_handle.wait_timeout(5).await {
                                Ok(None) => {
                                    // Timeout, escalate to force kill
                                    warn!("Graceful kill timed out, escalating to force kill");
                                    let _ = process_handle.force_kill().await;
                                }
                                _ => {} // Exited gracefully or error
                            }
                        }
                        KillMode::Force => {
                            if let Err(e) = process_handle.force_kill().await {
                                error!(error = %e, "Force kill failed");
                            }
                        }
                    }

                    // Reap the process after kill
                    process_handle.child.wait().await
                }
            };

            // wait for last log batch to flush
            let _ = streaming_handle.await;

            // Finalize - emit stopped event regardless of how we got here
            let (new_status, exit_code) = match exit_status {
                Ok(status) => {
                    let code: Option<i32> = status.code();
                    let new_status = if code == Some(0) {
                        ProcessStatus::Stopped {
                            exit_code: 0,
                            completed_at: get_utc_timestamp_string(),
                        }
                    } else {
                        ProcessStatus::Error {
                            exit_code: code,
                            message: format!("Process exited with code {:?}", code),
                        }
                    };
                    (new_status, code)
                }
                Err(e) => {
                    error!(execution_id, error = %e, "Failed to wait for process");
                    (
                        ProcessStatus::Error {
                            exit_code: None,
                            message: e.to_string(),
                        },
                        None,
                    )
                }
            };

            // Update status
            let old_status = {
                let mut s = status_clone.write().await;
                let old = s.clone();
                *s = new_status.clone();
                old
            };

            let exec_status = if was_killed {
                ExecutionStatus::Cancelled
            } else if exit_code == Some(0) {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failed
            };

            // Emit events
            let _ = event_sender_clone
                .send(ProcessEvent::StatusChanged(ProcessStatusChangedEvent {
                    execution_id,
                    old_status,
                    new_status: new_status.clone(),
                    timestamp: get_utc_timestamp_string(),
                }))
                .await;

            let _ = event_sender_clone
                .send(ProcessEvent::Stopped(ProcessStoppedEvent {
                    execution_id,
                    pid,
                    exit_code,
                    status: exec_status,
                    timestamp: get_utc_timestamp_string(),
                }))
                .await;

            debug!(execution_id, exit_code, "Process monitor completed");
        });

        Ok(Self {
            execution_id,
            command_id: context.command_id,
            command_name: context.name.clone(),
            pid,
            status,
            log_buffer,
            start_time,
            context,
            kill_tx: Some(kill_tx),
        })
    }

    /// Send graceful kill signal (returns immediately, monitor handles the wait)
    pub async fn graceful_kill(&mut self) -> Result<(), ProcessKillError> {
        self.kill_tx
            .take()
            .ok_or(ProcessKillError::AlreadyExited)?
            .send(KillMode::Graceful)
            .map_err(|_| ProcessKillError::AlreadyExited)
    }

    /// Send force kill signal (returns immediately, monitor handles the wait)
    pub async fn force_kill(&mut self) -> Result<(), ProcessKillError> {
        self.kill_tx
            .take()
            .ok_or(ProcessKillError::AlreadyExited)?
            .send(KillMode::Force)
            .map_err(|_| ProcessKillError::AlreadyExited)
    }

    pub async fn get_status(&self) -> ProcessStatus {
        self.status.read().await.clone()
    }

    pub async fn get_logs(&self, offset: usize, limit: usize) -> Vec<Arc<LogLineEvent>> {
        self.log_buffer.read().await.get_paginated(offset, limit)
    }

    pub async fn log_count(&self) -> usize {
        self.log_buffer.read().await.len()
    }

    pub async fn clear_logs(&self) {
        self.log_buffer.write().await.clear();
    }

    /// used by stop_all() to decide whether to kill,
    /// stopping process must return false or that would get kill signal twice.
    pub async fn is_running(&self) -> bool {
        // If kill signal already sent, process is not running
        if self.kill_tx.is_none() {
            return false;
        }
        matches!(*self.status.read().await, ProcessStatus::Running { .. })
    }
}
