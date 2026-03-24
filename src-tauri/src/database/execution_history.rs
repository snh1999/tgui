use super::{
    Database, DatabaseError, ExecutionHistory, ExecutionStats, ExecutionStatus, Result,
    StatsTarget, TriggeredBy,
};
use crate::constants::{EXECUTION_HISTORY_LIMIT, EXECUTION_HISTORY_TABLE};
use crate::database::helpers::QueryBuilder;
use rusqlite::{named_params, params};
use tracing::{debug, instrument, warn};

impl Database {
    #[instrument(skip(self, history))]
    pub fn create_execution_history(&self, history: &ExecutionHistory) -> Result<i64> {
        let triggered_by = history.triggered_by.as_str();
        self.validate_execution_history_input(history)?;

        self.create(
            EXECUTION_HISTORY_TABLE,
            "INSERT INTO execution_history
                (command_id, workflow_id, workflow_step_id, triggered_by, context, status)
             VALUES (:command_id, :workflow_id, :workflow_step_id, :triggered_by, :context, 'running')",
            named_params! {
                ":command_id": history.command_id,
                ":workflow_id": history.workflow_id,
                ":workflow_step_id": history.workflow_step_id,
                ":triggered_by": triggered_by,
                ":context": history.context,
            },
        )
    }

    fn validate_execution_history_input(&self, history: &ExecutionHistory) -> Result<()> {
        let (cmd, flow, flow_step) = (
            history.command_id.is_some(),
            history.workflow_id.is_some(),
            history.workflow_step_id.is_some(),
        );
        let is_valid = (cmd && flow && flow_step)
            || (cmd && !flow && !flow_step)
            || (!cmd && flow && !flow_step);

        if !is_valid {
            return Err(DatabaseError::InvalidData {
                reason:
                    "Invalid combination: must be (command only), (workflow only), or (all three)"
                        .into(),
                field: "command_id/workflow_id/workflow_step_id",
            });
        }

        if let Some(command_id) = history.command_id {
            self.get_command(command_id)?;
            let history = self.get_latest_execution_for_command(command_id);
            if let Some(history) = history {
                if history.status == ExecutionStatus::Running {
                    return Err(DatabaseError::InvalidData {
                        field: "command",
                        reason: "command is already running".to_string(),
                    });
                }
            }
        }

        if let Some(workflow_id) = history.workflow_id {
            self.get_workflow(workflow_id)?;
        }

        if let Some(workflow_step_id) = history.workflow_step_id {
            let workflow_step = self.get_workflow_step(workflow_step_id)?;
            if Some(workflow_step.command_id) != history.command_id
                || Some(workflow_step.workflow_id) != history.workflow_id
            {
                return Err(DatabaseError::InvalidData {
                    field: "workflow_step_id",
                    reason: "Invalid workflow_step reference".to_string(),
                });
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_execution_history(&self, id: i64) -> Result<ExecutionHistory> {
        self.query_row(
            EXECUTION_HISTORY_TABLE,
            id,
            "SELECT * FROM execution_history WHERE id = ?1",
            Self::row_to_execution_history,
        )
    }

    // TODO: weigh in on workflow id is null
    #[instrument(skip(self))]
    pub fn get_command_execution_history(
        &self,
        command_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<ExecutionHistory>> {
        let limit = limit.unwrap_or(EXECUTION_HISTORY_LIMIT);
        self.query_database(
            "SELECT * FROM execution_history
             WHERE command_id = ?1
             ORDER BY started_at DESC
             LIMIT ?2",
            params![command_id, limit],
            Self::row_to_execution_history,
        )
    }

    #[instrument(skip(self))]
    pub fn get_workflow_execution_history(
        &self,
        workflow_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<ExecutionHistory>> {
        let limit = limit.unwrap_or(EXECUTION_HISTORY_LIMIT);
        self.query_database(
            "SELECT * FROM execution_history
             WHERE workflow_id = ?1
             ORDER BY started_at DESC
             LIMIT ?2",
            params![workflow_id, limit],
            Self::row_to_execution_history,
        )
    }

    #[instrument(skip(self))]
    pub fn get_running_commands(&self) -> Result<Vec<ExecutionHistory>> {
        self.query_database(
            "SELECT * FROM execution_history
            WHERE status = 'running' AND command_id IS NOT NULL AND workflow_id IS NULL",
            params![],
            Self::row_to_execution_history,
        )
    }

    #[instrument(skip(self))]
    pub fn get_latest_execution_for_command(&self, command_id: i64) -> Option<ExecutionHistory> {
        let history = self.query_row(
            EXECUTION_HISTORY_TABLE,
            command_id,
            "SELECT * FROM execution_history WHERE command_id = ?1 ORDER BY started_at DESC LIMIT 1",
            Self::row_to_execution_history,
        );

        if let Ok(history) = history {
            Some(history)
        } else {
            None
        }
    }

    /// Store the OS PID once the process has actually been spawned (called immediately after `child.spawn()` succeeds).
    #[instrument(skip(self))]
    pub fn update_execution_pid(&self, id: i64, pid: u32) -> Result<()> {
        debug!(execution_id = id, pid, "Storing PID");
        let history = self.get_execution_history(id)?;

        if history.status != ExecutionStatus::Running {
            return Err(DatabaseError::InvalidData {
                field: "pid",
                reason: "command is not running".to_string(),
            });
        }

        self.execute_db(
            EXECUTION_HISTORY_TABLE,
            id,
            "UPDATE execution_history SET pid = ?1 WHERE id = ?2",
            params![pid, id],
        )
    }

    #[instrument(skip(self))]
    pub fn update_execution_history_status(
        &self,
        id: i64,
        status: ExecutionStatus,
        exit_code: Option<i32>,
    ) -> Result<()> {
        let status_str = status.as_str();

        let history = self.get_execution_history(id)?;
        if history.status != ExecutionStatus::Running || status == ExecutionStatus::Running {
            return Err(DatabaseError::InvalidData {
                field: "status",
                reason: format!("Invalid status of {status_str}"),
            });
        }

        debug!(
            execution_id = id,
            status = status_str,
            exit_code,
            "Finalising execution"
        );
        self.execute_db(
            "execution_history",
            id,
            "UPDATE execution_history
             SET status =:status, exit_code =:exit_code
             WHERE id =:id",
            named_params! {
                ":status": status_str,
                ":exit_code": exit_code,
                ":id": id,
            },
        )
    }

    pub fn kill_failed_execution(&self, id: i64) -> Result<()> {
        self.update_execution_history_status(id, ExecutionStatus::Failed, None)
    }

    #[instrument(skip(self))]
    pub fn delete_execution_history(&self, id: i64) -> Result<()> {
        self.execute_db(
            "execution_history",
            id,
            "DELETE FROM execution_history WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn cleanup_history_older_than(&self, days: i64) -> Result<()> {
        debug!(days, "Deleting execution history older than days");
        self.execute_db_raw(
            EXECUTION_HISTORY_TABLE,
            "DELETE FROM execution_history
             WHERE started_at < datetime('now', ?1) AND status != 'running'",
            params![format!("-{days} days")],
        )?;
        Ok(())
    }

    // TODO: delete workflow/step based
    /// Keeps the most recent `keep_last` executions for a command.
    #[instrument(skip(self))]
    pub fn cleanup_command_history(&self, command_id: i64, keep_last: i64) -> Result<()> {
        debug!(command_id, keep_last, "Cleaning up old execution history");
        self.execute_db_raw(
            EXECUTION_HISTORY_TABLE,
            "DELETE FROM execution_history
             WHERE command_id = ?1 AND workflow_id IS NULL
               AND id NOT IN (
                   SELECT id FROM execution_history
                   WHERE command_id = ?1
                   ORDER BY started_at DESC
                   LIMIT ?2
               )",
            params![command_id, keep_last],
        )?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_execution_stats(
        &self,
        target: StatsTarget,
        days: Option<i64>,
    ) -> Result<ExecutionStats> {
        let mut query_builder = QueryBuilder::new();

        match target {
            StatsTarget::Command(id) => {
                query_builder.add_condition("command_id = ?1 AND workflow_id IS NULL", id);
            }
            StatsTarget::Workflow(id) => {
                query_builder.add_condition("workflow_id = ?1 AND command_id IS NULL", id);
            }
            StatsTarget::Global => {
                query_builder.add_condition_without_param("workflow_step_id IS NULL");
            }
        }

        if let Some(days) = days {
            query_builder
                .add_condition("started_at >= datetime('now', ?)", format!("-{days} days"));
        }

        let (where_clause, param_refs) = query_builder.build();

        let query = format!(
            "SELECT
            COUNT(*) as total_count,
            COUNT(CASE WHEN status = 'success' THEN 1 END) as success_count,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_count,
            COUNT(CASE WHEN status = 'cancelled' THEN 1 END) as cancelled_count,
            COUNT(CASE WHEN status = 'timeout' THEN 1 END) as timeout_count,
            COUNT(CASE WHEN status = 'running' THEN 1 END) as running_count,
            COUNT(CASE WHEN status = 'paused' THEN 1 END) as paused_count,
            COUNT(CASE WHEN status = 'skipped' THEN 1 END) as skipped_count,
            AVG(
                CASE
                    WHEN completed_at IS NOT NULL AND started_at IS NOT NULL
                    THEN (julianday(completed_at) - julianday(started_at)) * 86400000
                    ELSE NULL
                END
            ) as avg_duration_ms,
            MAX(started_at) as last_executed_at,
            MIN(started_at) as first_executed_at
        FROM execution_history
        {where_clause}"
        );

        let connection = self.conn()?;
        let mut stmt = connection.prepare(&query)?;

        stmt.query_row(param_refs.as_slice(), |row| {
            let total_count: i64 = row.get("total_count")?;
            let success_count: i64 = row.get("success_count")?;

            Ok(ExecutionStats {
                total_count,
                success_count,
                failed_count: row.get("failed_count")?,
                cancelled_count: row.get("cancelled_count")?,
                timeout_count: row.get("timeout_count")?,
                running_count: row.get("running_count")?,
                paused_count: row.get("paused_count")?,
                skipped_count: row.get("skipped_count")?,
                success_rate: if total_count > 0 {
                    (success_count as f64 / total_count as f64 * 100.0).round() / 100.0
                } else {
                    0.0
                },
                average_duration_ms: row
                    .get::<_, Option<f64>>("avg_duration_ms")?
                    .map(|d| d.round() as i64),
                last_executed_at: row.get("last_executed_at")?,
                first_executed_at: row.get("first_executed_at")?,
            })
        })
        .map_err(Into::into)
    }

    fn row_to_execution_history(row: &rusqlite::Row) -> rusqlite::Result<ExecutionHistory> {
        let status_str: String = row.get("status")?;
        let status = ExecutionStatus::from_str(&status_str).unwrap_or_else(|e| {
            warn!(error = %e, "Invalid execution mode, defaulting to completed");
            ExecutionStatus::Completed
        });

        let workflow_id: Option<i64> = row.get("workflow_id")?;

        let triggered_by_str: String = row.get("triggered_by")?;
        let triggered_by = TriggeredBy::from_str(&triggered_by_str).unwrap_or_else(|e| {
            warn!(error = %e, "Invalid trigger mode, moving to default value");
            if workflow_id.is_some() {
                TriggeredBy::Workflow
            } else {
                TriggeredBy::Manual
            }
        });

        Ok(ExecutionHistory {
            id: row.get("id")?,
            command_id: row.get("command_id")?,
            workflow_id,
            workflow_step_id: row.get("workflow_step_id")?,
            pid: row.get("pid")?,
            status,
            exit_code: row.get("exit_code")?,
            started_at: row.get("started_at")?,
            completed_at: row.get("completed_at")?,
            triggered_by,
            context: row.get("context")?,
        })
    }
}
