use super::*;
mod categories;
mod commands;
mod execution_history;
mod groups;
mod integration;
mod settings;
mod workflows;

use tempfile::TempDir;

use crate::database::{Command, ExecutionMode, Group, StepCondition, Workflow, WorkflowStep};
use std::collections::HashMap;

pub struct TestDb {
    pub db: Database,
    _temp_dir: Option<TempDir>,
}

impl TestDb {
    fn setup_test_db() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        TestDb {
            db,
            _temp_dir: Some(temp_dir),
        }
    }

    fn create_test_category(&self, name: &str) -> i64 {
        self.db.create_category(name, None, None).unwrap()
    }

    fn create_test_group(&self, name: &str) -> i64 {
        let group = GroupBuilder::new(name).build();
        self.save_group_to_db(&group)
    }

    fn save_group_to_db(&self, group: &Group) -> i64 {
        self.db.create_group(group).unwrap()
    }

    fn create_test_command(&self, name: &str, command: &str, group_id: Option<i64>) -> i64 {
        let mut command_builder = CommandBuilder::new(name, command);
        if let Some(group_id) = group_id {
            command_builder = command_builder.with_group(group_id);
        }
        self.save_command_to_db(&command_builder.build())
    }

    fn save_command_to_db(&self, command: &Command) -> i64 {
        self.db.create_command(command).unwrap()
    }

    fn create_test_workflow(&self, name: &str) -> i64 {
        let workflow = WorkflowBuilder::new(name).build();
        self.save_workflow_to_db(&workflow)
    }

    fn save_workflow_to_db(&self, workflow: &Workflow) -> i64 {
        self.db.create_workflow(&workflow).unwrap()
    }

    fn create_test_workflow_step(&self, workflow_id: i64, command_id: i64) -> i64 {
        self.db
            .create_workflow_step(&WorkflowStepBuilder::new(workflow_id, command_id).build())
            .unwrap()
    }

    fn save_execution_history(&self, history: &ExecutionHistory) -> i64 {
        self.db.create_execution_history(history).unwrap()
    }
}

pub(crate) struct CommandBuilder {
    command: Command,
}

impl CommandBuilder {
    pub(crate) fn new(name: &str, cmd: &str) -> Self {
        Self {
            command: Command {
                id: 0,
                name: name.to_string(),
                command: cmd.to_string(),
                arguments: vec![],
                description: None,
                group_id: None,
                position: 0,
                working_directory: None,
                env_vars: None,
                shell: None,
                category_id: None,
                is_favorite: false,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn with_group(mut self, group_id: i64) -> Self {
        self.command.group_id = Some(group_id);
        self
    }

    pub(crate) fn with_args(mut self, args: Vec<&str>) -> Self {
        self.command.arguments = args.into_iter().map(String::from).collect();
        self
    }

    pub(crate) fn with_env(mut self, key: &str, value: &str) -> Self {
        let env = self.command.env_vars.get_or_insert_with(HashMap::new);
        env.insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn with_category(mut self, category_id: i64) -> Self {
        self.command.category_id = Some(category_id);
        self
    }

    pub(crate) fn build(self) -> Command {
        self.command
    }
}

pub(crate) struct GroupBuilder {
    group: Group,
}

impl GroupBuilder {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            group: Group {
                id: 0,
                name: name.to_string(),
                description: None,
                parent_group_id: None,
                position: 0,
                working_directory: None,
                env_vars: None,
                shell: None,
                category_id: None,
                is_favorite: false,
                icon: None,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn with_parent(mut self, parent_id: i64) -> Self {
        self.group.parent_group_id = Some(parent_id);
        self
    }

    pub(crate) fn with_category(mut self, category_id: i64) -> Self {
        self.group.category_id = Some(category_id);
        self
    }

    pub(crate) fn with_env(mut self, key: &str, value: &str) -> Self {
        let env = self.group.env_vars.get_or_insert_with(HashMap::new);
        env.insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn build(self) -> Group {
        self.group
    }
}

pub(crate) struct WorkflowBuilder {
    workflow: Workflow,
}

impl WorkflowBuilder {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            workflow: Workflow {
                id: 0,
                name: name.to_string(),
                description: None,
                category_id: None,
                is_favorite: false,
                execution_mode: ExecutionMode::Sequential,
                position: 0,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn with_category(mut self, category_id: i64) -> Self {
        self.workflow.category_id = Some(category_id);
        self
    }

    pub(crate) fn build(self) -> Workflow {
        self.workflow
    }
}

pub(crate) struct WorkflowStepBuilder {
    workflow_step: WorkflowStep,
}

impl WorkflowStepBuilder {
    pub(crate) fn new(workflow_id: i64, command_id: i64) -> Self {
        Self {
            workflow_step: WorkflowStep {
                id: 0,
                workflow_id,
                command_id,
                position: 0,
                condition: StepCondition::Always,
                timeout_seconds: None,
                auto_retry_count: None,
                enabled: true,
                continue_on_failure: false,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn build(self) -> WorkflowStep {
        self.workflow_step
    }
}

pub(crate) struct ExecutionHistoryBuilder {
    execution_history: ExecutionHistory,
}

impl ExecutionHistoryBuilder {
    pub(crate) fn new() -> Self {
        Self {
            execution_history: ExecutionHistory {
                id: 0,
                command_id: None,
                workflow_id: None,
                workflow_step_id: None,
                pid: None,
                status: Status::Running,
                exit_code: None,
                started_at: time::OffsetDateTime::now_utc().to_string(),
                completed_at: None,
                triggered_by: TriggeredBy::Manual,
                context: Some("test context".to_string()),
            },
        }
    }

    pub(crate) fn with_command(mut self, command_id: i64) -> Self {
        self.execution_history.command_id = Some(command_id);
        self
    }

    pub(crate) fn with_workflow(mut self, workflow_id: i64) -> Self {
        self.execution_history.workflow_id = Some(workflow_id);
        self
    }

    pub(crate) fn with_trigger(mut self, triggered_by: TriggeredBy) -> Self {
        self.execution_history.triggered_by = triggered_by;
        self
    }
    pub(crate) fn with_workflow_step(
        mut self,
        command_id: i64,
        workflow_id: i64,
        workflow_step_id: i64,
    ) -> Self {
        self.execution_history.workflow_step_id = Some(workflow_step_id);
        self.with_command(command_id)
            .with_workflow(workflow_id)
            .with_trigger(TriggeredBy::Workflow)
    }

    pub(crate) fn build(mut self) -> ExecutionHistory {
        self.execution_history
    }
}
