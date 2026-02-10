use crate::database::{Command, ExecutionMode, Group, StepCondition, Workflow, WorkflowStep};
use std::collections::HashMap;

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

    pub(crate) fn with_execution_mode(mut self, execution_mode: ExecutionMode) -> Self {
        self.workflow.execution_mode = execution_mode;
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
