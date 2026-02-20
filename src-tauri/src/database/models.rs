use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(skip_deserializing, default)]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parent_group_id: Option<i64>,
    #[serde(skip_deserializing, default)]
    pub position: i64,
    pub working_directory: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub shell: Option<String>,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    pub icon: Option<String>,
    #[serde(skip_deserializing, default)]
    pub created_at: String,
    #[serde(skip_deserializing, default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub id: i64,
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub description: Option<String>,
    pub group_id: Option<i64>,
    #[serde(skip_deserializing, default)]
    pub position: i64,
    pub working_directory: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub shell: Option<String>,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    #[serde(skip_deserializing, default)]
    pub created_at: String,
    #[serde(skip_deserializing, default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub structure: String, // JSON
    #[serde(skip_deserializing, default)]
    pub created_at: String,
    #[serde(skip_deserializing, default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    pub execution_mode: ExecutionMode,
    #[serde(skip_deserializing, default)]
    pub position: i64,
    #[serde(skip_deserializing, default)]
    pub created_at: String,
    #[serde(skip_deserializing, default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Sequential,  // Run one after another
    Parallel,    // Run all at once (TODO: future implementation)
    Conditional, // Run based on conditions (TODO: future implementation)
}

impl ExecutionMode {
    pub fn as_str(&self) -> &str {
        match self {
            ExecutionMode::Sequential => "sequential",
            ExecutionMode::Parallel => "parallel",
            ExecutionMode::Conditional => "conditional",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "sequential" => Ok(ExecutionMode::Sequential),
            "parallel" => Ok(ExecutionMode::Parallel),
            "conditional" => Ok(ExecutionMode::Conditional),
            _ => Err(format!("Invalid execution mode: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepCondition {
    Always,    // Always run
    OnSuccess, // Run only if previous step succeeded
    OnFailure, // Run only if previous step failed
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    pub id: i64,
    pub workflow_id: i64,
    pub command_id: i64,
    #[serde(skip_deserializing, default)]
    pub position: i64,
    pub condition: StepCondition,
    pub timeout_seconds: Option<u32>,
    pub auto_retry_count: Option<u8>,
    pub enabled: bool,
    pub continue_on_failure: bool,
    #[serde(skip_deserializing, default)]
    pub created_at: String,
    #[serde(skip_deserializing, default)]
    pub updated_at: String,
}

impl StepCondition {
    pub fn as_str(&self) -> &str {
        match self {
            StepCondition::Always => "always",
            StepCondition::OnSuccess => "on_success",
            StepCondition::OnFailure => "on_failure",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "always" => Ok(StepCondition::Always),
            "on_success" => Ok(StepCondition::OnSuccess),
            "on_failure" => Ok(StepCondition::OnFailure),
            _ => Err(format!("Invalid condition: {}", s)),
        }
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionHistory {
    pub id: i64,
    pub command_id: Option<i64>,
    pub workflow_id: Option<i64>,
    pub workflow_step_id: Option<i64>,
    #[serde(skip_deserializing, default)]
    pub pid: Option<i64>,
    #[serde(skip_deserializing, default)]
    pub status: Status,
    #[serde(skip_deserializing, default)]
    pub exit_code: Option<i32>,
    #[serde(skip_deserializing, default)]
    pub started_at: String,
    #[serde(skip_deserializing, default)]
    pub completed_at: Option<String>,
    pub triggered_by: TriggeredBy,
    /// Optional JSON for extra metadata (e.g., workflow context)
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggeredBy {
    Manual,
    Workflow,
    Schedule,
}

impl TriggeredBy {
    pub fn as_str(&self) -> &str {
        match self {
            TriggeredBy::Manual => "manual",
            TriggeredBy::Workflow => "workflow",
            TriggeredBy::Schedule => "schedule",
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "manual" => Ok(TriggeredBy::Manual),
            "workflow" => Ok(TriggeredBy::Workflow),
            "schedule" => Ok(TriggeredBy::Schedule),
            _ => Err(format!("Invalid trigger: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idle,
    #[default]
    Running,
    Success,
    Interrupted,
    Paused,
    Failed,
    TimedOut,
    Cancelled,
    Skipped,
    Completed
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Idle => "idle",
            Status::Running => "running",
            Status::Success => "success",
            Status::Interrupted => "interrupted",
            Status::Paused => "paused",
            Status::Failed => "failed",
            Status::TimedOut => "timed-out",
            Status::Cancelled => "cancelled",
            Status::Skipped => "skipped",
            Status::Completed => "completed"
        }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "idle" => Ok(Status::Idle),
            "running" => Ok(Status::Running),
            "success" => Ok(Status::Success),
            "interrupted" => Ok(Status::Interrupted),
            "paused" => Ok(Status::Paused),
            "failed" => Ok(Status::Failed),
            "timed-out" => Ok(Status::TimedOut),
            "cancelled" => Ok(Status::Cancelled),
            "skipped" => Ok(Status::Skipped),
            "completed" => Ok(Status::Completed),
            _ => Err(format!("Invalid execution mode: {}", s)),
        }
    }
}
