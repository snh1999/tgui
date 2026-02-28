use super::{
    Database, DatabaseError, ExecutionHistory, ExecutionStatus, Result, TriggeredBy,
};
use crate::constants::{EXECUTION_HISTORY_LIMIT, EXECUTION_HISTORY_TABLE};
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
                ":command_id":       history.command_id,
                ":workflow_id":      history.workflow_id,
                ":workflow_step_id": history.workflow_step_id,
                ":triggered_by":     triggered_by,
                ":context":          history.context,
            },
        )
    }

    fn validate_execution_history_input(&self, history: &ExecutionHistory) -> Result<()> {
        if let Some(command_id) = history.command_id {
            self.get_command(command_id)?;
        }

        if let Some(workflow_id) = history.workflow_id {
            self.get_workflow(workflow_id)?;
        }

        if let Some(workflow_step_id) = history.workflow_step_id {
            let workflow_step = self.get_workflow_step(workflow_step_id)?;
            if Some(workflow_step.command_id) != history.command_id || Some(workflow_step.workflow_id) != history.workflow_id {
                return Err(DatabaseError::InvalidData {
                    field: "workflow_step_id",
                    reason: "Invalid workflow_step reference".to_string()
                })
            }
        }

        let (cmd, flow, flow_step) = (
            history.command_id.is_some(),
            history.workflow_id.is_some(),
            history.workflow_step_id.is_some(),
        );
        let is_valid = (cmd && flow && flow_step) || (cmd && !flow && !flow_step) || (!cmd && flow && !flow_step);

        if !is_valid {
            return Err(DatabaseError::InvalidData {
                reason: "Invalid combination: must be (command only), (workflow only), or (all three)".into(),
                field: "command_id/workflow_id/workflow_step_id",
            });
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
    pub fn get_running_commands(
        &self,
        command_id: Option<i64>,
        workflow_id: Option<i64>,
    ) -> Result<Vec<ExecutionHistory>> {
        let mut query = "SELECT * FROM execution_history WHERE status = 'running'".to_string();

        if command_id.is_some() && workflow_id.is_some() {
            return Err(DatabaseError::InvalidData {
                field: "workflow_id",
                reason: "Invalid method call, only one query params allowed".to_string(),
            });
        }

        let params = if command_id.is_some() {
            query.push_str(" AND command_id = ?");
            params![command_id]
        } else if workflow_id.is_some() {
            query.push_str(" AND workflow_id = ?");
            params![workflow_id]
        } else {
            params![]
        };

        self.query_database(&query, params, Self::row_to_execution_history)
    }

    /// Store the OS PID once the process has actually been spawned (called immediately after `child.spawn()` succeeds).
    #[instrument(skip(self))]
    pub fn update_execution_pid(&self, id: i64, pid: u32) -> Result<()> {
        debug!(execution_id = id, pid, "Storing PID");

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
                reason: format!("Invalid status of {}", status_str),
            });
        }

        debug!(execution_id = id, status = status_str, exit_code, "Finalising execution");
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

    // TODO: delete workflow/step based
    /// Keeps the most recent `keep_last` executions for a command.
    #[instrument(skip(self))]
    pub fn cleanup_command_history(&self, command_id: i64, keep_last: i64) -> Result<()> {
        debug!(command_id, keep_last, "Cleaning up old execution history");
        self.execute_db_raw(
            EXECUTION_HISTORY_TABLE,
            "DELETE FROM execution_history
             WHERE command_id = ?1
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

    #[instrument(skip(self))]
    pub fn get_command_execution_stats(&self, command_id: i64, status: Option<ExecutionStatus>) -> Result<i64> {
        let query = match status {
            Some(status) => format!("SELECT COUNT(*) AS total FROM execution_history WHERE command_id = ?1 AND status = '{}'", status.as_str()),
            None => "SELECT COUNT(*) AS total FROM execution_history WHERE command_id = ?1".to_string(),
        };

        self.query_row(
            EXECUTION_HISTORY_TABLE,
            command_id,
            &query,
            |row| row.get(0)
        )
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
