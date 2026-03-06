use crate::constants::{CONNECTION_FAILED_MESSAGE, DATABASE_LOCKED_MESSAGE};
use crate::database::DatabaseError;
use crate::process::errors::ProcessKillError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub code: String,
    pub message: String,
}

impl From<String> for SerializableError {
    fn from(s: String) -> Self {
        SerializableError {
            code: "INTERNAL".to_string(),
            message: s,
        }
    }
}

impl From<DatabaseError> for SerializableError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound { entity, id } => SerializableError {
                code: "NOT_FOUND".to_string(),
                message: format!("{} with ID {} not found", entity, id),
            },
            DatabaseError::InvalidData { field, reason } => SerializableError {
                code: "INVALID_DATA".to_string(),
                message: format!("Invalid {}: {}", field, reason),
            },
            DatabaseError::CircularReference {
                group_id,
                parent_id,
            } => SerializableError {
                code: "CIRCULAR_REFERENCE".to_string(),
                message: format!(
                    "Circular reference detected: group {} cannot have parent {} (would create loop)",
                    group_id, parent_id
                ),
            },
            DatabaseError::ForeignKeyViolation {
                field,
                referenced_id,
            } => SerializableError {
                code: "FOREIGN_KEY_VIOLATION".to_string(),
                message: format!("{} references non-existent ID {}", field, referenced_id),
            },
            DatabaseError::DatabaseLocked => SerializableError {
                code: "DATABASE_LOCKED".to_string(),
                message: DATABASE_LOCKED_MESSAGE.to_string(),
            },
            DatabaseError::ConnectionFailed => SerializableError{
                code: "DATABASE_CONNECTION_FAIL".to_string(),
                message: CONNECTION_FAILED_MESSAGE.to_string(),
            },
            DatabaseError::Internal(msg) => SerializableError {
                code: "INTERNAL".to_string(),
                message: format!("Database error: {}", msg),
            },
        }
    }
}

impl From<ProcessKillError> for SerializableError {
    fn from(err: ProcessKillError) -> Self {
        match err {
            ProcessKillError::SignalFailed(message) => SerializableError {
                code: "SIGNAL_FAILED".to_string(),
                message,
            },
            ProcessKillError::WaitFailed(message) => SerializableError {
                code: "TIMEOUT".to_string(),
                message,
            },
            ProcessKillError::AlreadyExited => SerializableError {
                code: "INVALID_OPERATION".to_string(),
                message: "".to_string(),
            },
            ProcessKillError::PermissionDenied => SerializableError {
                code: "NO_PERMISSION".to_string(),
                message: "".to_string(),
            },
            ProcessKillError::PlatformError(message) => SerializableError {
                code: "PLATFORM_ERROR".to_string(),
                message,
            },
            ProcessKillError::NotFound(id) => SerializableError {
                code: "NOT_FOUND".to_string(),
                message: format!("{} with ID {} not found", id, id),
            },
            ProcessKillError::Invalid => SerializableError {
                code: "INVALID".to_string(),
                message: "Invalid data provided".to_string(),
            },
        }
    }
}
