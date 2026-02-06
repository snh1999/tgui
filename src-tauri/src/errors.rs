use crate::database::DatabaseError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl From<DatabaseError> for SerializableError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound { entity, id } => SerializableError {
                code: "NOT_FOUND".to_string(),
                message: format!("{} with ID {} not found", entity, id),
                entity: Some(entity.to_string()),
                id: Some(id),
                field: None,
            },
            DatabaseError::InvalidData { field, reason } => SerializableError {
                code: "INVALID_DATA".to_string(),
                message: format!("Invalid {}: {}", field, reason),
                entity: None,
                id: None,
                field: Some(field.to_string()),
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
                entity: Some("group".to_string()),
                id: Some(group_id),
                field: None,
            },
            DatabaseError::ForeignKeyViolation {
                field,
                referenced_id,
            } => SerializableError {
                code: "FOREIGN_KEY_VIOLATION".to_string(),
                message: format!("{} references non-existent ID {}", field, referenced_id),
                entity: None,
                id: Some(referenced_id),
                field: Some(field.to_string()),
            },
            DatabaseError::DatabaseLocked => SerializableError {
                code: "DATABASE_LOCKED".to_string(),
                message: "Database is locked by another process. Please try again.".to_string(),
                entity: None,
                id: None,
                field: None,
            },
            DatabaseError::Internal(msg) => SerializableError {
                code: "INTERNAL_ERROR".to_string(),
                message: format!("Database error: {}", msg),
                entity: None,
                id: None,
                field: None,
            },
        }
    }
}
