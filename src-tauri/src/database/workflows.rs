use super::{
    Command, Database, DatabaseError, ExecutionMode, Result, StepCondition, Workflow, WorkflowStep,
};
use crate::constants::{WORKFLOWS_TABLE, WORKFLOW_STEPS_TABLE};
use rusqlite::{named_params, params};
use tracing::{debug, info, instrument, warn};

impl Database {
    #[instrument(skip(self, workflow), fields(name = %workflow.name))]
    pub fn create_workflow(&self, workflow: &Workflow) -> Result<i64> {
        self.validate_non_empty("name", &workflow.name)?;

        let position: i64 = self.get_position(WORKFLOWS_TABLE, None, None)?;

        self.create(
            WORKFLOWS_TABLE,
            "INSERT INTO workflows (name, description, category_id, is_favorite, execution_mode, position)
             VALUES (:name, :description, :category_id, :is_favorite, :execution_mode, :position)",
            named_params! {
                ":name": workflow.name,
                ":description": workflow.description,
                ":category_id": workflow.category_id,
                ":is_favorite": workflow.is_favorite,
                ":execution_mode": workflow.execution_mode.as_str(),
                ":position": position,
            },
        )
    }

    #[instrument(skip(self))]
    pub fn get_workflow(&self, id: i64) -> Result<Workflow> {
        self.query_row(
            WORKFLOWS_TABLE,
            id,
            "SELECT * FROM workflows WHERE id = ?1",
            Self::row_to_workflow,
        )
    }

    #[instrument(skip(self))]
    pub fn get_workflows(
        &self,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<Vec<Workflow>> {
        let mut sql_statement = "SELECT * FROM workflows WHERE 1=1".to_string();

        if category_id.is_some() {
            sql_statement.push_str(&" AND category_id = ?".to_string());
        }

        if favorites_only {
            sql_statement.push_str(" AND is_favorite = 1");
        }

        sql_statement.push_str(" ORDER BY position");

        let params = if category_id.is_some() {
            params![category_id]
        } else {
            params![]
        };

        self.query_database(&sql_statement, params, |row| Self::row_to_workflow(row))
    }

    #[instrument(skip(self, workflow))]
    pub fn update_workflow(&self, workflow: &Workflow) -> Result<()> {
        self.validate_non_empty("name", &workflow.name)?;

        self.execute_db(
            WORKFLOWS_TABLE,
            workflow.id,
            "UPDATE workflows SET
                name = :name,
                description = :description,
                category_id = :category_id,
                execution_mode = :execution_mode,
                is_favorite = :is_favorite
             WHERE id =:id",
            named_params! {
                ":name":  workflow.name,
                ":description": workflow.description,
                ":category_id":workflow.category_id,
                ":execution_mode":workflow.execution_mode.as_str(),
                ":is_favorite": workflow.is_favorite,
                ":id":workflow.id
            },
        )
    }

    #[instrument(skip(self))]
    pub fn delete_workflow(&self, id: i64) -> Result<()> {
        self.execute_db(
            WORKFLOWS_TABLE,
            id,
            "DELETE FROM workflows WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn toggle_favorite_workflow(&self, id: i64) -> Result<()> {
        self.execute_db(
            WORKFLOWS_TABLE,
            id,
            "UPDATE workflows SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn get_workflow_count(&self, category_id: Option<i64>) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM workflows WHERE category_id IS ?";

        self.conn()?
            .query_row(&query, [category_id], |row| row.get(0))
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(self))]
    pub fn move_workflow_between(
        &self,
        workflow_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        self.move_item_between(
            WORKFLOWS_TABLE,
            workflow_id,
            prev_id,
            next_id,
            None,
            None,
            |id, default| self.get_position_parent_workflow(id, default),
        )
    }

    fn get_position_parent_workflow(
        &self,
        workflow_id: Option<i64>,
        default_val: i64,
    ) -> Result<(i64, Option<i64>)> {
        Ok(workflow_id
            .map(|id| self.get_workflow(id).map(|w| (w.position, None)))
            .transpose()?
            .unwrap_or((default_val, None)))
    }

    #[instrument(skip(self, flow_step), fields(workflow_id = flow_step.workflow_id, command_id = flow_step.command_id))]
    pub fn create_workflow_step(&self, flow_step: &WorkflowStep) -> Result<i64> {
        self.get_workflow(flow_step.workflow_id)?;
        self.get_command(flow_step.command_id)?;

        let position = self.get_position(
            WORKFLOW_STEPS_TABLE,
            Some("workflow_id"),
            Some(flow_step.workflow_id),
        )?;

        self.create(
            WORKFLOW_STEPS_TABLE,
            "INSERT INTO workflow_steps (workflow_id, command_id, position, condition, timeout_seconds, auto_retry_count, enabled, continue_on_failure)
             VALUES (:workflow_id, :command_id, :position, :condition, :timeout_seconds, :auto_retry_count, :enabled, :continue_on_failure)",
            named_params! {
                ":workflow_id": flow_step.workflow_id,
                ":command_id": flow_step.command_id,
                ":position": position,
                ":condition": flow_step.condition.as_str(),
                ":timeout_seconds": flow_step.timeout_seconds,
                ":auto_retry_count": flow_step.auto_retry_count,
                ":enabled": flow_step.enabled,
                ":continue_on_failure": flow_step.continue_on_failure
            },
        )
    }

    #[instrument(skip(self), ret)]
    pub fn get_workflow_step(&self, id: i64) -> Result<WorkflowStep> {
        self.query_row(
            WORKFLOW_STEPS_TABLE,
            id,
            "SELECT * FROM workflow_steps WHERE id = ?1",
            Self::row_to_workflow_step,
        )
    }

    #[instrument(skip(self))]
    pub fn get_workflow_steps(
        &self,
        workflow_id: Option<i64>,
        command_id: Option<i64>,
        enabled_only: bool,
    ) -> Result<Vec<WorkflowStep>> {
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut query = "SELECT * FROM workflow_steps WHERE 1=1".to_string();

        if let Some(workflow_id) = workflow_id {
            query.push_str(" AND workflow_id = ?1 ");
            params.push(Box::new(workflow_id));
        }
        if let Some(command_id) = command_id {
            query.push_str(&format!(" AND command_id = ?{} ", params.len() + 1));
            params.push(Box::new(command_id));
        }
        if enabled_only {
            query.push_str(" AND enabled = 1 ");
        }
        query.push_str(" ORDER BY position");

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.query_database(&query, &*param_refs, Self::row_to_workflow_step)
    }

    #[instrument(skip(self))]
    pub fn get_workflow_steps_command_populated(
        &self,
        workflow_id: i64,
        enabled_only: bool,
    ) -> Result<Vec<(WorkflowStep, Command)>> {
        let query = if enabled_only {
            "SELECT 
            ws.id as ws_id, ws.workflow_id, ws.command_id, ws.position as ws_position,
            ws.condition, ws.timeout_seconds, ws.auto_retry_count, ws.enabled, 
            ws.continue_on_failure, ws.created_at as ws_created_at, ws.updated_at as ws_updated_at,
            c.id as cmd_id, c.name, c.command, c.arguments, c.description, c.group_id,
            c.position as cmd_position, c.working_directory, c.env_vars, c.shell,
            c.category_id, c.is_favorite, c.created_at as cmd_created_at, c.updated_at as cmd_updated_at
         FROM workflow_steps ws
         JOIN commands c ON ws.command_id = c.id
         WHERE ws.workflow_id = ?1 AND ws.enabled = 1
         ORDER BY ws.position"
        } else {
            "SELECT 
            ws.id as ws_id, ws.workflow_id, ws.command_id, ws.position as ws_position,
            ws.condition, ws.timeout_seconds, ws.auto_retry_count, ws.enabled, 
            ws.continue_on_failure, ws.created_at as ws_created_at, ws.updated_at as ws_updated_at,
            c.id as cmd_id, c.name, c.command, c.arguments, c.description, c.group_id,
            c.position as cmd_position, c.working_directory, c.env_vars, c.shell,
            c.category_id, c.is_favorite, c.created_at as cmd_created_at, c.updated_at as cmd_updated_at
         FROM workflow_steps ws
         JOIN commands c ON ws.command_id = c.id
         WHERE ws.workflow_id = ?1
         ORDER BY ws.position"
        };

        self.query_database(query, params![workflow_id], |row| {
            let condition_str: String = row.get("condition")?;
            let condition = StepCondition::from_str(&condition_str).unwrap_or_else(|e| {
                warn!(error = %e, "Invalid condition, defaulting to always");
                StepCondition::Always
            });

            let argument_str: String = row.get("arguments")?;
            let arguments = serde_json::from_str(&argument_str).unwrap_or_else(|e| {
                warn!(error = %e, "Failed to parse arguments, using default");
                Vec::new()
            });
            let env_vars_str: Option<String> = row.get("env_vars")?;
            let env_vars = Self::string_to_hashmap(env_vars_str);

            let step = WorkflowStep {
                id: row.get("ws_id")?,
                workflow_id: row.get("workflow_id")?,
                command_id: row.get("command_id")?,
                position: row.get("ws_position")?,
                condition,
                timeout_seconds: row.get("timeout_seconds")?,
                auto_retry_count: row.get("auto_retry_count")?,
                enabled: row.get("enabled")?,
                continue_on_failure: row.get("continue_on_failure")?,
                created_at: row.get("ws_created_at")?,
                updated_at: row.get("ws_updated_at")?,
            };

            let cmd = Command {
                id: row.get("cmd_id")?,
                name: row.get("name")?,
                command: row.get("command")?,
                arguments,
                description: row.get("description")?,
                group_id: row.get("group_id")?,
                position: row.get("cmd_position")?,
                working_directory: row.get("working_directory")?,
                env_vars,
                shell: row.get("shell")?,
                category_id: row.get("category_id")?,
                is_favorite: row.get("is_favorite")?,
                created_at: row.get("cmd_created_at")?,
                updated_at: row.get("cmd_updated_at")?,
            };

            Ok((step, cmd))
        })
    }

    #[instrument(skip(self, flow_step))]
    pub fn update_workflow_step(&self, flow_step: &WorkflowStep) -> Result<()> {
        debug!(
            step_id = flow_step.id,
            workflow_id = flow_step.workflow_id,
            "Updating workflow step"
        );

        self.get_command(flow_step.command_id)?;

        self.execute_db(
            WORKFLOW_STEPS_TABLE,
            flow_step.id,
            "UPDATE workflow_steps SET
                command_id = :command_id,
                condition = :condition,
                timeout_seconds = :timeout_seconds,
                auto_retry_count = :auto_retry_count,
                enabled = :enabled,
                continue_on_failure = :continue_on_failure
             WHERE id = :id",
            named_params! {
                ":command_id": flow_step.command_id,
                ":condition": flow_step.condition.as_str(),
                ":timeout_seconds": flow_step.timeout_seconds,
                ":auto_retry_count": flow_step.auto_retry_count,
                ":enabled": flow_step.enabled,
                ":continue_on_failure": flow_step.continue_on_failure,
                ":id": flow_step.id
            },
        )
    }

    #[instrument(skip(self))]
    pub fn delete_workflow_step(&self, id: i64) -> Result<()> {
        debug!(step_id = id, "Deleting workflow step");

        self.execute_db(
            WORKFLOW_STEPS_TABLE,
            id,
            "DELETE FROM workflow_steps WHERE id = ?1",
            params![id],
        )?;

        info!(step_id = id, "Workflow step deleted");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn move_workflow_step_between(
        &self,
        step_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        let step = self.get_workflow_step(step_id)?;

        self.move_item_between(
            WORKFLOW_STEPS_TABLE,
            step_id,
            prev_id,
            next_id,
            Some("workflow_id"),
            Some(step.workflow_id),
            |id, default| self.get_position_parent_step(id, default),
        )
    }

    #[instrument(skip(self))]
    fn get_position_parent_step(
        &self,
        step_id: Option<i64>,
        default_val: i64,
    ) -> Result<(i64, Option<i64>)> {
        Ok(step_id
            .map(|id| {
                self.get_workflow_step(id)
                    .map(|s| (s.position, Some(s.workflow_id)))
            })
            .transpose()?
            .unwrap_or((default_val, None)))
    }

    #[instrument(skip(self))]
    pub fn toggle_workflow_step_enabled(&self, id: i64) -> Result<()> {
        debug!(step_id = id, "Toggling workflow step enabled");

        self.execute_db(
            WORKFLOW_STEPS_TABLE,
            id,
            "UPDATE workflow_steps SET enabled = NOT enabled WHERE id = ?1",
            params![id],
        )?;

        info!(step_id = id, "Workflow step enabled toggled");
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_workflow_step_count(&self, workflow_id: i64) -> Result<i64> {
        self.conn()?
            .query_row(
                "SELECT COUNT(*) FROM workflow_steps WHERE workflow_id = ?1",
                params![workflow_id],
                |row| row.get(0),
            )
            .map_err(DatabaseError::from)
    }

    fn row_to_workflow(row: &rusqlite::Row) -> rusqlite::Result<Workflow> {
        let execution_mode_str: String = row.get("execution_mode")?;
        let execution_mode = ExecutionMode::from_str(&execution_mode_str).unwrap_or_else(|e| {
            warn!(error = %e, "Invalid execution mode, defaulting to sequential");
            ExecutionMode::Sequential
        });

        Ok(Workflow {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            category_id: row.get("category_id")?,
            is_favorite: row.get("is_favorite")?,
            execution_mode,
            position: row.get("position")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    fn row_to_workflow_step(row: &rusqlite::Row) -> rusqlite::Result<WorkflowStep> {
        let condition_str: String = row.get("condition")?;
        let condition = StepCondition::from_str(&condition_str).unwrap_or_else(|e| {
            warn!(error = %e, "Invalid condition, defaulting to always");
            StepCondition::Always
        });

        Ok(WorkflowStep {
            id: row.get("id")?,
            workflow_id: row.get("workflow_id")?,
            command_id: row.get("command_id")?,
            position: row.get("position")?,
            condition,
            timeout_seconds: row.get("timeout_seconds")?,
            auto_retry_count: row.get("auto_retry_count")?,
            enabled: row.get("enabled")?,
            continue_on_failure: row.get("continue_on_failure")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
