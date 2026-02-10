use super::*;
use crate::constants::{COMMANDS_TABLE, WORKFLOWS_TABLE, WORKFLOW_STEPS_TABLE};

#[test]
fn test_workflow_builder() {
    let test_db = TestDb::setup_test_db();
    let category_id = test_db.create_test_category("Test");
    let mut workflow = WorkflowBuilder::new("Test Workflow")
        .with_category(category_id)
        .build();

    workflow.description = Some("Test workflow: Test Workflow".to_string());
    workflow.position = 10;
    workflow.is_favorite = false;
    workflow.execution_mode = ExecutionMode::Parallel;

    let id = test_db.save_workflow_to_db(&workflow);
    assert!(id > 0);

    let retrieved = test_db.db.get_workflow(id).unwrap();
    assert_eq!(retrieved.name, workflow.name);
    assert_eq!(retrieved.description, workflow.description);
    assert_eq!(retrieved.position, Database::POSITION_GAP);
    assert_eq!(retrieved.is_favorite, workflow.is_favorite);
    assert_eq!(retrieved.execution_mode, workflow.execution_mode);
    assert_eq!(retrieved.category_id, workflow.category_id);
}

#[test]
fn test_create_and_get_workflow() {
    let test_db = TestDb::setup_test_db();
    let workflow_name = "Test Workflow";
    let id = test_db.create_test_workflow(workflow_name);
    assert!(id > 0);
    let retrieved = test_db.db.get_workflow(id).unwrap();
    assert_eq!(retrieved.name, workflow_name);
}

#[test]
fn test_create_command_duplicate_name() {
    let test_db = TestDb::setup_test_db();

    let name = "Test Name";
    let id1 = test_db.create_test_workflow(name);
    let id2 = test_db.create_test_workflow(name);

    assert_ne!(id1, id2);
}

#[test]
fn test_create_workflow_empty_name() {
    let test_db = TestDb::setup_test_db();
    let workflow = WorkflowBuilder::new("").build();
    let result = test_db.db.create_workflow(&workflow);

    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_create_workflow_whitespace_name() {
    let test_db = TestDb::setup_test_db();
    let workflow = WorkflowBuilder::new("   ").build();
    let result = test_db.db.create_workflow(&workflow);

    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_get_workflow_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_workflow(9999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            ..
        })
    ));
}

#[test]
fn test_get_workflows_empty() {
    let test_db = TestDb::setup_test_db();
    let workflows = test_db.db.get_workflows(None, false).unwrap();
    assert_eq!(workflows.len(), 0);
}

#[test]
fn test_get_workflows() {
    let test_db = TestDb::setup_test_db();

    test_db.create_test_workflow("Workflow 1");
    test_db.create_test_workflow("Workflow 2");
    test_db.create_test_workflow("Workflow 3");

    let workflows = test_db.db.get_workflows(None, false).unwrap();
    assert_eq!(workflows.len(), 3);
}

#[test]
fn test_get_favorite_workflow() {
    let test_db = TestDb::setup_test_db();

    let mut workflow = WorkflowBuilder::new("Workflow").build();
    test_db.save_workflow_to_db(&workflow);

    workflow.is_favorite = true;
    test_db.save_workflow_to_db(&workflow);
    test_db.save_workflow_to_db(&workflow);

    let favorites = test_db.db.get_workflows(None, true).unwrap();
    assert_eq!(favorites.len(), 2);
}

#[test]
fn test_get_workflows_by_category_and_favorites() {
    let test_db = TestDb::setup_test_db();

    let dev_cat = Some(test_db.create_test_category("Dev"));
    let prod_cat = Some(test_db.create_test_category("Prod"));

    let mut workflow = WorkflowBuilder::new("Workflow").build();

    workflow.category_id = dev_cat;
    test_db.save_workflow_to_db(&workflow);
    workflow.category_id = dev_cat;
    workflow.is_favorite = true;
    test_db.save_workflow_to_db(&workflow);
    workflow.category_id = prod_cat;
    test_db.save_workflow_to_db(&workflow);

    let dev_workflows = test_db.db.get_workflows(dev_cat, false).unwrap();
    assert_eq!(dev_workflows.len(), 2);
    let dev_workflows_favorites = test_db.db.get_workflows(dev_cat, true).unwrap();
    assert_eq!(dev_workflows_favorites.len(), 1);
    let prod_workflows = test_db.db.get_workflows(prod_cat, false).unwrap();
    assert_eq!(prod_workflows.len(), 1);
}

#[test]
fn test_update_workflow_not_found() {
    let test_db = TestDb::setup_test_db();
    let mut workflow = WorkflowBuilder::new("Workflow").build();
    workflow.id = 9999;

    let result = test_db.db.update_workflow(&workflow);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id: 9999
        })
    ));
}

#[test]
fn test_update_workflow_name_validation() {
    let test_db = TestDb::setup_test_db();
    let mut workflow = WorkflowBuilder::new("Workflow").build();

    test_db.save_workflow_to_db(&workflow);

    workflow.name = "".to_string();

    let result = test_db.db.update_workflow(&workflow);
    assert!(matches!(
        result,
        Err(DatabaseError::InvalidData { field: "name", .. })
    ));
}

#[test]
fn test_update_workflow_preserve_position() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.save_workflow_to_db(&WorkflowBuilder::new("Workflow").build());

    let mut original = test_db.db.get_workflow(id).unwrap();

    let original_position = original.position;
    original.position = 100;

    test_db.db.update_workflow(&original).unwrap();

    let updated = test_db.db.get_workflow(id).unwrap();
    assert_eq!(original_position, updated.position);
}

#[test]
fn test_update_workflow() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_workflow("Workflow");
    let cat_id = Some(test_db.create_test_category("Test"));

    let mut workflow = test_db.db.get_workflow(id).unwrap();
    workflow.name = "Updated Name".to_string();
    workflow.description = Some("Updated description".to_string());
    workflow.execution_mode = ExecutionMode::Parallel;
    workflow.category_id = cat_id;
    workflow.is_favorite = true;

    test_db.db.update_workflow(&workflow).unwrap();

    let retrieved = test_db.db.get_workflow(id).unwrap();
    assert_eq!(retrieved.name, "Updated Name");
    assert_eq!(retrieved.description, workflow.description);
    assert_eq!(retrieved.execution_mode, ExecutionMode::Parallel);
    assert_eq!(retrieved.category_id, workflow.category_id);
    assert_eq!(retrieved.is_favorite, workflow.is_favorite);
}

#[test]
fn test_delete_workflow() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_workflow("To Delete");

    test_db.db.delete_workflow(id).unwrap();

    let result = test_db.db.get_workflow(id);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id
        })
    ));
}

#[test]
fn test_delete_workflow_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.delete_workflow(9999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id: 9999
        })
    ));
}

#[test]
fn test_toggle_workflow_favorite() {
    let test_db = TestDb::setup_test_db();
    let id = test_db.create_test_workflow("Workflow");

    let original = test_db.db.get_workflow(id).unwrap();

    test_db.db.toggle_favorite_workflow(id).unwrap();
    let updated = test_db.db.get_workflow(id).unwrap();

    assert_eq!(original.position, updated.position);
}

#[test]
fn test_toggle_workflow_favorite_not_found() {
    let test_db = TestDb::setup_test_db();

    let result = test_db.db.toggle_favorite_workflow(999);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id: 999
        })
    ));
}

#[test]
fn test_get_workflow_count() {
    let test_db = TestDb::setup_test_db();

    let cat_id = Some(test_db.create_test_category("Test"));
    let cat_id_2 = Some(test_db.create_test_category("Test2"));

    let mut workflow = WorkflowBuilder::new("Test Workflow").build();

    test_db.save_workflow_to_db(&workflow);
    workflow.category_id = cat_id;
    test_db.save_workflow_to_db(&workflow);
    test_db.save_workflow_to_db(&workflow);

    let count_cat1 = test_db.db.get_workflow_count(cat_id).unwrap();
    assert_eq!(count_cat1, 2);

    let count_none = test_db.db.get_workflow_count(None).unwrap();
    assert_eq!(count_none, 1);

    let count_none = test_db.db.get_workflow_count(cat_id_2).unwrap();
    assert_eq!(count_none, 0);
}

#[test]
fn test_move_workflow_between() {
    let test_db = TestDb::setup_test_db();

    let id1 = test_db.create_test_workflow("w1");
    let id2 = test_db.create_test_workflow("w2");
    let id3 = test_db.create_test_workflow("w3");

    test_db
        .db
        .move_workflow_between(id1, Some(id2), Some(id3))
        .unwrap();

    let w1_after = test_db.db.get_workflow(id1).unwrap();
    let w2_after = test_db.db.get_workflow(id2).unwrap();
    let w3_after = test_db.db.get_workflow(id3).unwrap();

    assert!(w2_after.position < w1_after.position);
    assert!(w1_after.position < w3_after.position);
}

// ==================== WORKFLOW STEPS TESTS ====================

#[test]
fn test_workflow_steps_builder() {
    let test_db = TestDb::setup_test_db();
    let workflow_id = test_db.create_test_workflow("Workflow");
    let command_id = test_db.create_test_command("test command", "echo test", None);
    let workflow_step = WorkflowStepBuilder::new(workflow_id, command_id).build();

    let id = test_db.db.create_workflow_step(&workflow_step).unwrap();
    assert!(id > 0);

    let retrieved = test_db.db.get_workflow_step(id).unwrap();
    assert_eq!(retrieved.workflow_id, workflow_step.workflow_id);
    assert_eq!(retrieved.command_id, workflow_step.command_id);
    assert_eq!(retrieved.position, Database::POSITION_GAP);
    assert_eq!(retrieved.enabled, workflow_step.enabled);
    assert_eq!(retrieved.condition, workflow_step.condition);
    assert_eq!(retrieved.timeout_seconds, workflow_step.timeout_seconds);
    assert_eq!(retrieved.auto_retry_count, workflow_step.auto_retry_count);
    assert_eq!(
        retrieved.continue_on_failure,
        workflow_step.continue_on_failure
    );
}

#[test]
fn test_create_and_get_workflow_step() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Workflow");
    let command_id = test_db.create_test_command("test command", "echo test", None);

    let id = test_db.create_test_workflow_step(workflow_id, command_id);

    assert!(id > 0);

    let retrieved = test_db.db.get_workflow_step(id).unwrap();
    assert_eq!(retrieved.id, id);
    assert_eq!(retrieved.workflow_id, workflow_id);
    assert_eq!(retrieved.command_id, command_id);
}

#[test]
fn test_create_workflow_step_invalid_workflow() {
    let test_db = TestDb::setup_test_db();
    let command_id = test_db.create_test_command("Test", "echo test", None);

    let workflow_step = WorkflowStepBuilder::new(9999, command_id).build();

    let result = test_db.db.create_workflow_step(&workflow_step);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOWS_TABLE,
            id: 9999
        })
    ));
}

#[test]
fn test_create_workflow_step_invalid_command() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Workflow");
    let workflow_step = WorkflowStepBuilder::new(workflow_id, 9999).build();

    let result = test_db.db.create_workflow_step(&workflow_step);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: COMMANDS_TABLE,
            id: 9999
        })
    ));
}

#[test]
fn test_get_workflow_step_not_found() {
    let test_db = TestDb::setup_test_db();
    let result = test_db.db.get_workflow_step(1);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOW_STEPS_TABLE,
            id: 1
        })
    ));
}

#[test]
fn test_get_workflow_steps() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let workflow_id_2 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);
    test_db.create_test_workflow_step(workflow_id_2, command_id_2);

    let result = test_db.db.get_workflow_steps(None, None, false).unwrap();
    assert_eq!(result.len(), 4);

    let result = test_db
        .db
        .get_workflow_steps(Some(workflow_id_1), None, false)
        .unwrap();
    assert_eq!(result.len(), 3);

    let result = test_db
        .db
        .get_workflow_steps(None, Some(command_id_2), false)
        .unwrap();
    assert_eq!(result.len(), 3);

    let result = test_db
        .db
        .get_workflow_steps(Some(workflow_id_1), Some(command_id_2), false)
        .unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_get_workflow_steps_enabled() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let workflow_id_2 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);

    let mut workflow_steps = WorkflowStepBuilder::new(workflow_id_1, command_id_1).build();
    workflow_steps.enabled = false;
    test_db.db.create_workflow_step(&workflow_steps).unwrap();

    let mut workflow_steps = WorkflowStepBuilder::new(workflow_id_1, command_id_2).build();
    workflow_steps.enabled = false;
    test_db.db.create_workflow_step(&workflow_steps).unwrap();

    test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    test_db.create_test_workflow_step(workflow_id_2, command_id_2);

    let result = test_db.db.get_workflow_steps(None, None, true).unwrap();
    assert_eq!(result.len(), 2);

    let result = test_db
        .db
        .get_workflow_steps(None, Some(command_id_1), true)
        .unwrap();
    assert_eq!(result.len(), 1);

    let result = test_db
        .db
        .get_workflow_steps(Some(workflow_id_1), None, true)
        .unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_get_workflow_steps_populated() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let workflow_id_2 = test_db.create_test_workflow("Test");
    let (cmd_1_name, cmd_2_name) = ("Test", "Test2");
    let command_id_1 = test_db.create_test_command(cmd_1_name, "Echo test", None);
    let command_id_2 = test_db.create_test_command(cmd_2_name, "Echo test", None);
    let command_id_3 = test_db.create_test_command(cmd_2_name, "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_3);
    test_db.create_test_workflow_step(workflow_id_2, command_id_1);
    test_db.create_test_workflow_step(workflow_id_2, command_id_2);

    let result = test_db
        .db
        .get_workflow_steps_command_populated(workflow_id_2, false)
        .unwrap();

    assert_eq!(result.len(), 2);

    let (step1, cmd1) = &result[0];
    assert_eq!(step1.workflow_id, workflow_id_2);
    assert_eq!(step1.command_id, command_id_1);
    assert_eq!(cmd1.id, command_id_1);
    assert_eq!(cmd1.name, cmd_1_name);

    let (step2, cmd2) = &result[1];
    assert_eq!(step2.workflow_id, workflow_id_2);
    assert_eq!(step2.command_id, command_id_2);
    assert_eq!(cmd2.id, command_id_2);
    assert_eq!(cmd2.name, cmd_2_name);

    let result = test_db
        .db
        .get_workflow_steps_command_populated(workflow_id_1, false)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0.workflow_id, workflow_id_1);
    assert_eq!(result[0].0.command_id, command_id_3);
    assert_eq!(result[0].1.id, command_id_3);
}

#[test]
fn test_get_workflow_steps_enabled_only() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let workflow_id_2 = test_db.create_test_workflow("Test");
    let name = "Test";
    let command_id_1 = test_db.create_test_command(name, "Echo test", None);
    let command_id_2 = test_db.create_test_command(name, "Echo test", None);
    let command_id_3 = test_db.create_test_command(name, "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_3);
    test_db.create_test_workflow_step(workflow_id_2, command_id_1);
    test_db.create_test_workflow_step(workflow_id_2, command_id_1);
    let id = test_db.create_test_workflow_step(workflow_id_2, command_id_2);
    test_db.db.toggle_workflow_step_enabled(id).unwrap();
    let id = test_db.create_test_workflow_step(workflow_id_2, command_id_1);
    test_db.db.toggle_workflow_step_enabled(id).unwrap();

    let all_steps = test_db
        .db
        .get_workflow_steps_command_populated(workflow_id_2, false)
        .unwrap();
    assert_eq!(all_steps.len(), 4);

    let enabled_only = test_db
        .db
        .get_workflow_steps_command_populated(workflow_id_2, true)
        .unwrap();
    assert_eq!(enabled_only.len(), 2);
}

#[test]
fn test_update_workflow_step() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Test");
    let cmd1_id = test_db.create_test_command("Test", "echo test", None);
    let cmd2_id = test_db.create_test_command("Test2", "echo test", None);

    let flow_step_id = test_db.create_test_workflow_step(workflow_id, cmd1_id);

    let mut updated = test_db.db.get_workflow_step(flow_step_id).unwrap();
    updated.command_id = cmd2_id;
    updated.condition = StepCondition::OnSuccess;
    updated.timeout_seconds = Some(60);
    updated.auto_retry_count = Some(3);
    updated.continue_on_failure = true;

    test_db.db.update_workflow_step(&updated).unwrap();

    let retrieved = test_db.db.get_workflow_step(flow_step_id).unwrap();
    assert_eq!(retrieved.command_id, updated.command_id);
    assert_eq!(retrieved.condition, updated.condition);
    assert_eq!(retrieved.timeout_seconds, updated.timeout_seconds);
    assert_eq!(retrieved.auto_retry_count, updated.auto_retry_count);
    assert!(retrieved.continue_on_failure);
}

#[test]
fn test_update_workflow_step_to_running() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Test");
    let cmd1_id = test_db.create_test_command("Test", "echo test", None);
    let cmd2_id = test_db.create_test_command("Test2", "echo test", None);

    let flow_step_id = test_db.create_test_workflow_step(workflow_id, cmd1_id);

    let mut updated = test_db.db.get_workflow_step(flow_step_id).unwrap();
    updated.condition = StepCondition::OnSuccess;
    updated.timeout_seconds = Some(60);
    updated.auto_retry_count = Some(3);
    updated.continue_on_failure = true;

    test_db.db.update_workflow_step(&updated).unwrap();

    let retrieved = test_db.db.get_workflow_step(flow_step_id).unwrap();
    assert_eq!(retrieved.command_id, updated.command_id);
    assert_eq!(retrieved.condition, updated.condition);
    assert_eq!(retrieved.timeout_seconds, updated.timeout_seconds);
    assert_eq!(retrieved.auto_retry_count, updated.auto_retry_count);
    assert!(retrieved.continue_on_failure);
}

#[test]
fn test_update_workflow_step_not_found() {
    let test_db = TestDb::setup_test_db();

    let flow_id = test_db.create_test_workflow("Test");
    let cmd_id = test_db.create_test_command("Test", "echo test", None);

    let flow_step = WorkflowStepBuilder::new(flow_id, cmd_id).build();

    let result = test_db.db.update_workflow_step(&flow_step);

    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOW_STEPS_TABLE,
            ..
        })
    ))
}

#[test]
fn test_update_workflow_step_invalid_command_id() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Test");
    let cmd1_id = test_db.create_test_command("Test", "echo test", None);

    let flow_step_id = test_db.create_test_workflow_step(workflow_id, cmd1_id);

    let mut updated = test_db.db.get_workflow_step(flow_step_id).unwrap();
    updated.command_id = 9999;

    let result = test_db.db.update_workflow_step(&updated);

    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: COMMANDS_TABLE,
            id: 9999,
            ..
        })
    ))
}

#[test]
fn test_delete_workflow_step() {
    let test_db = TestDb::setup_test_db();

    let workflow_id = test_db.create_test_workflow("Test");
    let command_id = test_db.create_test_command("Test", "Echo test", None);

    let id = test_db.create_test_workflow_step(workflow_id, command_id);
    test_db.db.delete_workflow_step(id).unwrap();

    let result = test_db.db.get_workflow_step(id);
    assert!(matches!(
        result,
        Err(DatabaseError::NotFound {
            entity: WORKFLOW_STEPS_TABLE,
            id
        })
    ));
}

#[test]
fn test_delete_workflow_cascade() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);

    let steps = test_db
        .db
        .get_workflow_steps(Some(workflow_id_1), None, false)
        .unwrap();
    assert_eq!(steps.len(), 2);

    test_db.db.delete_workflow(workflow_id_1).unwrap();

    let steps = test_db
        .db
        .get_workflow_steps(Some(workflow_id_1), None, false)
        .unwrap();
    assert_eq!(steps.len(), 0);
}

#[test]
fn test_command_delete_workflow_step_cascade() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);

    let steps = test_db
        .db
        .get_workflow_steps(None, Some(command_id_2), false)
        .unwrap();
    assert_eq!(steps.len(), 2);

    test_db.db.delete_command(command_id_2).unwrap();

    let steps = test_db
        .db
        .get_workflow_steps(None, Some(command_id_2), false)
        .unwrap();
    assert_eq!(steps.len(), 0);
}

#[test]
fn test_toggle_workflow_step_enabled() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);

    let id = test_db.create_test_workflow_step(workflow_id_1, command_id_1);

    let original = test_db.db.get_workflow_step(id).unwrap();
    assert!(original.enabled);

    test_db.db.toggle_workflow_step_enabled(id).unwrap();
    let updated = test_db.db.get_workflow_step(id).unwrap();
    assert!(!updated.enabled);
}

#[test]
fn test_get_workflow_step_count() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);

    test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);
    test_db.create_test_workflow_step(workflow_id_1, command_id_2);

    let count = test_db.db.get_workflow_step_count(workflow_id_1).unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_move_workflow_step_between() {
    let test_db = TestDb::setup_test_db();

    let workflow_id_1 = test_db.create_test_workflow("Test");
    let command_id_1 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_2 = test_db.create_test_command("Test", "Echo test", None);
    let command_id_3 = test_db.create_test_command("Test", "Echo test", None);

    let id1 = test_db.create_test_workflow_step(workflow_id_1, command_id_1);
    let id2 = test_db.create_test_workflow_step(workflow_id_1, command_id_2);
    let id3 = test_db.create_test_workflow_step(workflow_id_1, command_id_2);

    // Move step1 between step2 and step3
    test_db
        .db
        .move_workflow_step_between(id1, Some(id2), Some(id3))
        .unwrap();

    let s1 = test_db.db.get_workflow_step(id1).unwrap();
    let s2 = test_db.db.get_workflow_step(id2).unwrap();
    let s3 = test_db.db.get_workflow_step(id3).unwrap();

    assert!(s2.position < s1.position);
    assert!(s1.position < s3.position);
}
//TODO test update time

// #[test]
// fn test_workflow_step_conditions() {
//     let test_db = TestDb::setup_test_db();
//
//     let workflow = create_new_workflow(&db, "Test", None);
//     let workflow_id = db.create_workflow(&workflow).unwrap();
//     let cmd = create_test_command(&db, "cmd");
//
//     let mut step_always = WorkflowStep {
//         id: 0,
//         workflow_id,
//         command_id: cmd,
//         position: 0,
//         condition: StepCondition::Always,
//         timeout_seconds: None,
//         auto_retry_count: 0,
//         enabled: true,
//         continue_on_failure: false,
//         created_at: String::new(),
//         updated_at: String::new(),
//     };
//
//     let id_always = db.create_workflow_step(&step_always).unwrap();
//     let retrieved_always = db.get_workflow_step(id_always).unwrap();
//     assert_eq!(retrieved_always.condition, StepCondition::Always);
//
//     step_always.id = 0;
//     step_always.condition = StepCondition::OnSuccess;
//     let id_success = db.create_workflow_step(&step_always).unwrap();
//     let retrieved_success = db.get_workflow_step(id_success).unwrap();
//     assert_eq!(retrieved_success.condition, StepCondition::OnSuccess);
//
//     step_always.id = 0;
//     step_always.condition = StepCondition::OnFailure;
//     let id_failure = db.create_workflow_step(&step_always).unwrap();
//     let retrieved_failure = db.get_workflow_step(id_failure).unwrap();
//     assert_eq!(retrieved_failure.condition, StepCondition::OnFailure);
// }
