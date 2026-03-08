use crate::database::{Database, ExecutionHistory, ExecutionStatus, TriggeredBy};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::handlers::serialize_errors::SerializableError;
use crate::process::errors::{ProcessKillError, ProcessManagerError, ProcessSpawnError};
use crate::process::managed_process::{ManagedProcess, ProcessEvent};
use crate::process::models::{LogLineEvent, OrphanedProcess, ProcessInfo, SpawnContext};
use crate::process::shell;
use dashmap::DashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub struct ProcessManager {
    processes: DashMap<i64, ManagedProcess>,
    db: Database,
    app_handle: Option<AppHandle>,
    event_sender: mpsc::Sender<ProcessEvent>,
}

impl ProcessManager {
    pub async fn new(db: Database, app_handle: Option<AppHandle>) -> Arc<Self> {
        let (event_sender, event_receiver) = mpsc::channel(1000);

        let pm = Arc::new(Self {
            processes: DashMap::new(),
            db,
            app_handle,
            event_sender,
        });

        let pm_clone = pm.clone();
        let handle = tokio::runtime::Handle::current();
        handle.spawn(async move {
            pm_clone.process_events(event_receiver).await;
        });

        pm
    }

    pub async fn spawn_command(
        &self,
        context: SpawnContext,
        triggered_by: TriggeredBy,
    ) -> Result<i64, ProcessSpawnError> {
        if let Some(ref shell) = context.shell {
            if !shell::is_valid_shell(shell) {
                return Err(ProcessSpawnError::InvalidShell(format!(
                    "Shell '{shell}' is not allowed. Permitted: {}",
                    shell::get_allowed_shells().join(", ")
                )));
            }
        }

        let command_id = context.command_id;

        let execution_id = self
            .db
            .create_execution_history(&ExecutionHistory::new_with_command(
                command_id,
                triggered_by,
            ))
            .map_err(|e| {
                error!(error = %e, "Failed to create execution history");
                ProcessSpawnError::DatabaseError("Could not create execution history".to_string())
            })?;

        let kill_tree = self
            .db
            .get_setting("kill_process_tree_by_default")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut process =
            ManagedProcess::spawn(execution_id, context, self.event_sender.clone(), kill_tree)
                .await.map_err(|e| {
                if let Err(db_err) = self.db.kill_failed_execution(execution_id) {
                    error!(execution_id, error = %db_err, "Failed to cancel orphaned history row after spawn failure");
                }
                e
            })?;

        if let Err(e) = self.db.update_execution_pid(execution_id, process.pid) {
            error!(execution_id, pid = process.pid, error = %e, "Failed to update PID in DB");
            let _ = process.force_kill().await;

            if let Err(e) = self.db.kill_failed_execution(execution_id) {
                error!(execution_id, error = %e, "Failed to mark failed execution");
            }

            return Err(ProcessSpawnError::DatabaseError(
                "PID update failed".to_string(),
            ));
        }

        self.processes.insert(execution_id, process);
        info!(execution_id, command_id, "Command spawned and tracked");
        Ok(execution_id)
    }

    pub async fn kill_process(
        &self,
        execution_id: i64,
        force: bool,
    ) -> Result<(), ProcessKillError> {
        let mut process = self
            .processes
            .get_mut(&execution_id)
            .ok_or(ProcessKillError::NotFound(execution_id))?;

        if force {
            process.force_kill().await
        } else {
            process.graceful_kill().await
        }
    }

    pub async fn stop_all(&self, force: bool) -> usize {
        let mut count = 0;

        let ids: Vec<i64> = self.processes.iter().map(|e| *e.key()).collect();

        for id in ids {
            if let Some(mut entry) = self.processes.get_mut(&id) {
                let process = entry.value_mut();
                if !process.is_running().await {
                    continue;
                }

                let result = if force {
                    process.force_kill().await
                } else {
                    process.graceful_kill().await
                };

                match result {
                    Ok(_) => count += 1,
                    Err(e) => {
                        warn!(execution_id = id, error = ?e, "Failed to kill process in stop_all")
                    }
                }
            }
        }
        count
    }

    pub async fn get_process_info(&self, execution_id: i64) -> Option<ProcessInfo> {
        let process = self.processes.get(&execution_id)?;
        let status = process.get_status().await;
        let log_count = process.log_count().await;

        Some(ProcessInfo {
            execution_id,
            pid: process.pid,
            command_id: process.command_id,
            command_name: process.command_name.clone(),
            command: process.context.executable.clone(),
            status,
            start_time: process.start_time.clone(),
            exit_code: None,
            log_line_count: log_count,
        })
    }

    pub fn detect_and_mark_orphans(&self) -> Vec<OrphanedProcess> {
        let running = match self.db.get_running_commands(None, None) {
            Ok(rows) => rows,
            Err(e) => {
                error!(error = %e, "Failed to query running executions on startup");
                return Vec::new();
            }
        };

        let mut orphans = Vec::new();
        for row in running {
            let pid = match row.pid {
                Some(p) => p as u32,
                None => {
                    if let Err(e) = self.db.update_execution_history_status(
                        row.id,
                        ExecutionStatus::Failed,
                        None,
                    ) {
                        error!(error = %e, "Failed to update execution status");
                    }
                    continue;
                }
            };

            let still_running = Self::pid_is_alive(pid);
            if !still_running {
                if let Err(e) =
                    self.db
                        .update_execution_history_status(row.id, ExecutionStatus::Failed, None)
                {
                    error!(error = %e, "Failed to update execution status");
                }
            } else {
                // Process is alive but untracked — kill it then mark canceled
                #[cfg(unix)]
                {
                    use nix::sys::signal::{kill as nix_kill, Signal};
                    use nix::unistd::Pid;
                    if let Err(e) = nix_kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                        error!(pid, error = %e, "Failed to kill orphaned process");
                    }
                }
                #[cfg(windows)]
                {
                    if let Err(e) = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output()
                    {
                        error!(error = %e, "Failed to kill orphaned process");
                    };
                }

                if let Err(e) = self.db.update_execution_history_status(
                    row.id,
                    ExecutionStatus::Cancelled,
                    None,
                ) {
                    error!(error = %e, "Failed to update execution status");
                }
            }

            orphans.push(OrphanedProcess {
                execution_id: row.id,
                command_id: row.command_id,
                pid: pid as i64,
                still_running,
            });
        }
        orphans
    }

    pub async fn resolve_spawn_context(
        &self,
        command_id: i64,
    ) -> Result<SpawnContext, SerializableError> {
        let cmd = self
            .db
            .get_command(command_id)
            .map_err(SerializableError::from)?;

        let ancestors = cmd
            .group_id
            .and_then(|gid| self.db.get_group_ancestor_chain(gid).ok())
            .unwrap_or_default();

        let working_directory = cmd
            .working_directory
            .clone()
            .or_else(|| ancestors.iter().find_map(|g| g.working_directory.clone()))
            .and_then(|wd| {
                if wd.starts_with("~/") {
                    dirs::home_dir().map(|home| home.join(&wd[2..]))
                } else {
                    Some(PathBuf::from(wd))
                }
            })
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| PathBuf::from("/"));

        if !working_directory.exists() {
            return Err(SerializableError {
                code: "INVALID_DIRECTORY".to_string(),
                message: format!(
                    "Working directory does not exist: {}",
                    working_directory.display()
                ),
            });
        }

        let shell = cmd
            .shell
            .clone()
            .or_else(|| ancestors.iter().find_map(|g| g.shell.clone()))
            .or_else(|| self.db.get_setting("default_shell").ok());

        let mut env_map: HashMap<String, String> = HashMap::new();
        let mut update_env_map = |env_vars: &Option<HashMap<String, String>>| {
            if let Some(ref vars) = env_vars {
                env_map.extend(vars.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        };

        for group in ancestors.iter().rev() {
            update_env_map(&group.env_vars);
        }
        update_env_map(&cmd.env_vars);

        let env_vars: Vec<(String, String)> = env_map.into_iter().collect();

        Ok(SpawnContext {
            command_id: cmd.id,
            name: cmd.name,
            executable: cmd.command,
            arguments: cmd.arguments,
            working_directory,
            env_vars,
            shell,
        })
    }

    fn pid_is_alive(pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::kill as nix_kill;
            use nix::unistd::Pid;
            nix_kill(Pid::from_raw(pid as i32), None).is_ok()
        }
        #[cfg(windows)]
        {
            std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {pid}"), "/NH"])
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()))
                .unwrap_or(false)
        }
    }

    pub async fn get_running_processes(&self) -> Vec<ProcessInfo> {
        let mut result = Vec::new();
        for entry in self.processes.iter() {
            let process = entry.value();
            if process.is_running().await {
                if let Some(info) = self.get_process_info(*entry.key()).await {
                    result.push(info);
                }
            }
        }
        result
    }

    pub async fn get_logs(
        &self,
        execution_id: i64,
        offset: usize,
        limit: usize,
    ) -> Option<Vec<Arc<LogLineEvent>>> {
        let process = self.processes.get(&execution_id)?;
        Some(process.get_logs(offset, limit).await)
    }

    pub async fn clear_logs(&self, execution_id: i64) -> Result<(), ProcessManagerError> {
        let process = self
            .processes
            .get(&execution_id)
            .ok_or(ProcessManagerError::ProcessNotFound(execution_id))?;
        process.clear_logs().await;
        Ok(())
    }

    async fn process_events(self: Arc<Self>, mut receiver: mpsc::Receiver<ProcessEvent>) {
        while let Some(event) = receiver.recv().await {
            match event {
                ProcessEvent::Started(evt) => {
                    self.emit_event("process-started", &evt);
                }
                ProcessEvent::StatusChanged(evt) => {
                    self.emit_event("process-status-changed", &evt);
                }
                ProcessEvent::Stopped(evt) => {
                    self.emit_event("process-stopped", &evt);

                    if let Err(e) = self.db.update_execution_history_status(
                        evt.execution_id,
                        evt.status,
                        evt.exit_code,
                    ) {
                        error!(execution_id = evt.execution_id, error = %e, "Failed to update execution status");
                    }
                    let execution_id = evt.execution_id;
                    let this = self.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        this.processes.remove(&execution_id);
                        debug!(execution_id, "Removed completed process from active map");
                    });
                }
                ProcessEvent::LogBatch(lines) => {
                    self.emit_log_batch(&lines);
                }
            }
        }
    }

    fn emit_log_batch(&self, lines: &[Arc<LogLineEvent>]) {
        if let Some(ref handle) = self.app_handle {
            let event_name = if lines.len() == 1 {
                "log-line"
            } else {
                "log-batch"
            };
            let payload = if lines.len() == 1 {
                serde_json::to_value(&lines[0])
            } else {
                serde_json::to_value(&lines)
            };

            match payload {
                Ok(p) => {
                    let _ = handle.emit(event_name, p);
                }
                Err(e) => {
                    error!(error = %e, "Failed to serialize log batch");
                }
            }
        }
    }

    fn emit_event<T: serde::Serialize>(&self, event_name: &str, payload: &T) {
        if let Some(ref handle) = self.app_handle {
            if let Err(e) = handle.emit(event_name, payload) {
                error!(event = event_name, error = %e, "Failed to emit event");
            }
        }
    }

    pub async fn running_count(&self) -> usize {
        let mut count = 0;
        for entry in self.processes.iter() {
            if entry.value().is_running().await {
                count += 1;
            }
        }
        count
    }
}
