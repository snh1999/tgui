use super::*;
mod categories;
mod commands;
mod groups;
mod integration;
mod settings;
mod workflows;

use crate::database::builders::{
    CommandBuilder, GroupBuilder, WorkflowBuilder, WorkflowStepBuilder,
};
use tempfile::TempDir;

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
}
