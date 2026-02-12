use crate::database::{Command, Database, Workflow, WorkflowStep};
use crate::handlers::serialize_errors::SerializableError;
use tauri::State;

#[tauri::command]
pub fn create_workflow(
    db: State<'_, Database>,
    workflow: Workflow,
) -> Result<i64, SerializableError> {
    db.create_workflow(&workflow).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow(db: State<'_, Database>, id: i64) -> Result<Workflow, SerializableError> {
    db.get_workflow(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflows(
    db: State<'_, Database>,
    category_id: Option<i64>,
    favorites_only: bool,
) -> Result<Vec<Workflow>, SerializableError> {
    db.get_workflows(category_id, favorites_only)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn update_workflow(
    db: State<'_, Database>,
    workflow: Workflow,
) -> Result<(), SerializableError> {
    db.update_workflow(&workflow).map_err(|err| err.into())
}

#[tauri::command]
pub fn delete_workflow(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.delete_workflow(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn toggle_favorite_workflow(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.toggle_favorite_workflow(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_count_for_category(
    db: State<'_, Database>,
    category_id: Option<i64>,
) -> Result<i64, SerializableError> {
    db.get_workflow_count(category_id).map_err(|err| err.into())
}

#[tauri::command]
pub fn move_workflow_between(
    db: State<'_, Database>,
    workflow_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), SerializableError> {
    db.move_workflow_between(workflow_id, prev_id, next_id)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn create_workflow_step(
    db: State<'_, Database>,
    flow_steps: WorkflowStep,
) -> Result<i64, SerializableError> {
    db.create_workflow_step(&flow_steps)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_step(
    db: State<'_, Database>,
    id: i64,
) -> Result<WorkflowStep, SerializableError> {
    db.get_workflow_step(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_steps(
    db: State<'_, Database>,
    workflow_id: Option<i64>,
    command_id: Option<i64>,
    enabled_only: bool,
) -> Result<Vec<WorkflowStep>, SerializableError> {
    db.get_workflow_steps(workflow_id, command_id, enabled_only)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_steps_command_populated(
    db: State<'_, Database>,
    workflow_id: i64,
    enabled_only: bool,
) -> Result<Vec<(WorkflowStep, Command)>, SerializableError> {
    db.get_workflow_steps_command_populated(workflow_id, enabled_only)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn update_workflow_step(
    db: State<'_, Database>,
    workflow: WorkflowStep,
) -> Result<(), SerializableError> {
    db.update_workflow_step(&workflow).map_err(|err| err.into())
}

#[tauri::command]
pub fn delete_workflow_step(db: State<'_, Database>, id: i64) -> Result<(), SerializableError> {
    db.delete_workflow_step(id).map_err(|err| err.into())
}

#[tauri::command]
pub fn move_workflow_step_between(
    db: State<'_, Database>,
    workflow_id: i64,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> Result<(), SerializableError> {
    db.move_workflow_step_between(workflow_id, prev_id, next_id)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn toggle_workflow_step_enabled(
    db: State<'_, Database>,
    id: i64,
) -> Result<(), SerializableError> {
    db.toggle_workflow_step_enabled(id)
        .map_err(|err| err.into())
}

#[tauri::command]
pub fn get_workflow_step_count(db: State<'_, Database>, id: i64) -> Result<i64, SerializableError> {
    db.get_workflow_step_count(id).map_err(|err| err.into())
}
