use crate::constants::{CONNECTION_FAILED_MESSAGE, DATABASE_LOCKED_MESSAGE};
use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    NotFound {
        entity: &'static str,
        id: i64,
    },
    InvalidData {
        field: &'static str,
        reason: String,
    },
    CircularReference {
        group_id: i64,
        parent_id: i64,
    },
    ForeignKeyViolation {
        field: &'static str,
        referenced_id: i64,
    },
    DatabaseLocked,
    ConnectionFailed,
    Internal(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(f, "{} with ID {} not found", entity, id),
            Self::InvalidData { field, reason } => write!(f, "Invalid {}: {}", field, reason),
            Self::CircularReference {
                group_id,
                parent_id,
            } => write!(
                f,
                "Circular reference detected: group {} cannot have parent {} (would create loop)",
                group_id, parent_id
            ),
            Self::ForeignKeyViolation {
                field,
                referenced_id,
            } => write!(f, "{} references non-existent ID {}", field, referenced_id),
            Self::DatabaseLocked => write!(f, "{}", DATABASE_LOCKED_MESSAGE),
            Self::ConnectionFailed => write!(f, "{}", CONNECTION_FAILED_MESSAGE),
            Self::Internal(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound {
                entity: "record",
                id: 0,
            },
            rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("FOREIGN KEY") => {
                Self::ForeignKeyViolation {
                    field: "unknown",
                    referenced_id: 0,
                }
            }
            rusqlite::Error::SqliteFailure(_, Some(msg)) if msg.contains("database is locked") => {
                Self::DatabaseLocked
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidData {
            field: "json",
            reason: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
