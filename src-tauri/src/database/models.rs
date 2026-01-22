use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parent_group_id: Option<i64>,
    pub position: i64,
    pub working_directory: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub shell: Option<String>,
    pub category_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: i64,
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub description: Option<String>,
    pub group_id: Option<i64>,
    pub position: i64,
    pub working_directory: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub shell: Option<String>,
    pub category_id: Option<i64>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub structure: String, // JSON
    pub created_at: String,
    pub updated_at: String,
}
