use crate::constants::{CONNECTION_FAILED_MESSAGE, DATABASE_LOCKED_MESSAGE};
use crate::database::DatabaseError;
use crate::process::errors::ProcessKillError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub reason: String,
    pub message: String,
}

impl From<String> for SerializableError {
    fn from(s: String) -> Self {
        SerializableError{
            reason: "INTERNAL".to_string(),
            message: s,
        }
    }
}



impl From<DatabaseError> for SerializableError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound { entity, id } => SerializableError {
                reason: "NOT_FOUND".to_string(),
                message: format!("{} with ID {} not found", entity, id),
            },
            DatabaseError::InvalidData { field, reason } => SerializableError {
                reason: "INVALID_DATA".to_string(),
                message: format!("Invalid {}: {}", field, reason),
            },
            DatabaseError::CircularReference {
                group_id,
                parent_id,
            } => SerializableError {
                reason: "CIRCULAR_REFERENCE".to_string(),
                message: format!(
                    "Circular reference detected: group {} cannot have parent {} (would create loop)",
                    group_id, parent_id
                ),
            },
            DatabaseError::ForeignKeyViolation {
                field,
                referenced_id,
            } => SerializableError {
                reason: "FOREIGN_KEY_VIOLATION".to_string(),
                message: format!("{} references non-existent ID {}", field, referenced_id),
            },
            DatabaseError::DatabaseLocked => SerializableError {
                reason: "DATABASE_LOCKED".to_string(),
                message: DATABASE_LOCKED_MESSAGE.to_string(),
            },
            DatabaseError::ConnectionFailed => SerializableError{
                reason: "DATABASE_CONNECTION_FAIL".to_string(),
                message: CONNECTION_FAILED_MESSAGE.to_string(),
            },
            DatabaseError::Internal(msg) => SerializableError {
                reason: "INTERNAL".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}

impl From<ProcessKillError> for SerializableError {
    fn from(err: ProcessKillError) -> Self {
        match err {
            ProcessKillError::SignalFailed(message) => SerializableError {
                reason: "SIGNAL_FAILED".to_string(),
                message,
            },
            ProcessKillError::WaitFailed(message) => SerializableError{
                reason:"TIMEOUT".to_string(),
                message
            },
            ProcessKillError::AlreadyExited =>SerializableError {
                reason:"INVALID_OPERATION".to_string(),
                message: "".to_string(),
            },
            ProcessKillError::PermissionDenied => SerializableError{
                reason:"NO_PERMISSION".to_string(),
                message: "".to_string(),

            },
            ProcessKillError::PlatformError(message) => SerializableError{
                reason:"PLATFORM_ERROR".to_string(),
                message
            },
            ProcessKillError::NotFound(id) => SerializableError{
                reason:"NOT_FOUND".to_string(),
                message: format!("{} with ID {} not found", id, id),
            },
        }
    }
}
