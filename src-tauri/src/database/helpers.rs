pub use crate::database::errors::{DatabaseError, Result};
use crate::database::Database;
use rusqlite::params;
use serde_json::Error;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, error, info, warn};

pub(crate) struct QueryBuilder {
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            conditions: vec![],
            params: vec![],
        }
    }

    pub fn add_condition(
        &mut self,
        condition: &str,
        param: impl rusqlite::ToSql + 'static,
    ) -> &mut Self {
        self.conditions.push(condition.to_string());
        self.params.push(Box::new(param));
        self
    }

    pub fn add_condition_without_param(&mut self, condition: &str) -> &mut Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn build(&self) -> (String, Vec<&dyn rusqlite::ToSql>) {
        let where_clause = if self.conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        };
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            self.params.iter().map(|p| p.as_ref()).collect();
        (where_clause, param_refs)
    }
}

impl Database {
    pub(crate) const POSITION_GAP: i64 = 1000;
    pub(crate) const MAX_NAME_LENGTH: usize = 255;
    pub(crate) const MAX_COMMAND_LENGTH: usize = 10000;
    pub(crate) const MAX_DESCRIPTION_LENGTH: usize = 2000;

    pub(crate) fn validate_field_length(
        &self,
        field: &'static str,
        value: &str,
        max: usize,
    ) -> Result<()> {
        if value.trim().is_empty() {
            error!("Empty field: {field}");
            return Err(DatabaseError::InvalidData {
                field,
                reason: format!("{field} cannot be empty"),
            });
        } else if value.len() > max {
            return Err(DatabaseError::InvalidData {
                field,
                reason: format!("{} exceeds maximum length of {}", field, max),
            });
        }
        Ok(())
    }

    pub(crate) fn create<P: rusqlite::Params>(
        &self,
        table: &'static str,
        sql: &str,
        params: P,
    ) -> Result<i64> {
        let sql = format!("{} RETURNING id", sql);

        let row_id: i64 = self
            .conn()?
            .query_row(&sql, params, |row| row.get(0))
            .map_err(|e| {
                error!(error = %e, table = table, "Failed to create");
                DatabaseError::from(e)
            })?;

        info!(id = row_id, entity = table, "created successfully");
        Ok(row_id)
    }

    pub(crate) fn execute_db<P: rusqlite::Params>(
        &self,
        table: &'static str,
        id: i64,
        sql: &str,
        params: P,
    ) -> Result<()> {
        let rows_affected = self.execute_db_raw(table, sql, params)?;

        if rows_affected == 0 {
            error!(id = id, table = table, "Not found");
            return Err(DatabaseError::NotFound { entity: table, id });
        }

        info!(id = id, table = table, "Database operation successful");
        Ok(())
    }

    pub(crate) fn execute_db_raw<P: rusqlite::Params>(
        &self,
        table: &'static str,
        sql: &str,
        params: P,
    ) -> Result<usize> {
        self.conn()?.execute(sql, params).map_err(|e| {
            error!(error = %e, table=table, "Database operation failed");
            DatabaseError::from(e)
        })
    }

    /// query_row can be used only for query by id (only param)
    pub(crate) fn query_row<T, F>(
        &self,
        table: &'static str,
        id: i64,
        sql_query: &str,
        mut row_mapper: F,
    ) -> Result<T>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let item = self
            .conn()?
            .query_row(sql_query, params![id], |row| row_mapper(row))
            .map_err(|e| {
                error!(error = %e, "Query failed");

                return match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        DatabaseError::NotFound { entity: table, id }
                    }
                    _ => e.into(),
                };
            })?;

        debug!(id = id, table = table, "Fetched entry");

        Ok(item)
    }

    pub(crate) fn query_database<T, F, P>(
        &self,
        sql_query: &str,
        params: P,
        mut row_mapper: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
        P: rusqlite::Params,
    {
        let connection = self.conn()?;
        let mut stmt = connection.prepare(sql_query)?;
        let results = stmt
            .query_map(params, |row| row_mapper(row))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(results)
    }

    /// initial position is Self::POSITION_GAP instead of zero,
    /// starting from 0 forces renumbering every move to beginning operation
    pub(crate) fn get_position(
        &self,
        table: &'static str,
        parent_column: Option<&'static str>,
        parent_id: Option<i64>,
    ) -> Result<i64> {
        let conn = self.conn()?;
        let position = Self::get_position_with_conn(&conn, table, parent_column, parent_id)?;
        debug!(
            calculated_position = position,
            called_by = table,
            "Command position"
        );
        Ok(position)
    }

    pub(crate) fn get_position_with_conn(
        conn: &rusqlite::Connection,
        table: &'static str,
        parent_column: Option<&'static str>,
        parent_id: Option<i64>,
    ) -> Result<i64> {
        let mut query = format!("SELECT COALESCE(MAX(position), 0) FROM {table}");

        if let Some(parent_column) = parent_column {
            query.push_str(&format!(" WHERE {parent_column} IS ?1"));
        }

        let params = if parent_column.is_some() {
            params![parent_id]
        } else {
            params![]
        };

        let position =
            Self::POSITION_GAP + conn.query_row(&query, params, |row| row.get::<_, i64>(0))?;

        Ok(position)
    }

    pub(crate) fn update_parent_group(
        &self,
        table: &'static str,
        parent_column: &'static str,
        id: i64,
        parent_id: Option<i64>,
    ) -> Result<()> {
        let position = self.get_position(table, Some(parent_column), parent_id)?;
        let query =
            format!("UPDATE {table} SET {parent_column} = ?1, position = ?2 WHERE id = {id}");
        self.execute_db(table, id, &query, params![parent_id, position])?;
        Ok(())
    }

    pub(crate) fn validate_env_var_keys(
        &self,
        env_vars: &Option<HashMap<String, String>>,
    ) -> Result<()> {
        if let Some(vars) = env_vars {
            for key in vars.keys() {
                if !key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    error!("Invalid env variable key: {}", key);
                    return Err(DatabaseError::InvalidData {
                        field: "env_vars",
                        reason: format!("Invalid key '{key}': only alphanumeric, underscore, dash"),
                    });
                }
            }
        }
        Ok(())
    }

    /// Move command between two positions (calculates midpoint)
    /// prev_id None means move to top
    /// next_id None means move to bottom
    pub(crate) fn move_item_between<F>(
        &self,
        table: &'static str,
        item_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
        parent_column: Option<&'static str>,
        mut get_position_parent: F,
    ) -> Result<()>
    where
        F: FnMut(Option<i64>, i64) -> Result<(i64, Option<i64>)>,
    {
        if prev_id.is_none() && next_id.is_none() {
            debug!("Invalid method call, Cannot move item between non-empty items");
            return Err(DatabaseError::InvalidData {
                field: "item_id",
                reason: "Invalid positons. Either prev_id or next_id must be provided".to_string(),
            });
        }

        let (prev_pos, prev_parent) = get_position_parent(prev_id, 0)?;
        let (next_pos, next_parent) = get_position_parent(next_id, prev_pos + Self::POSITION_GAP)?;
        let (current_pos, parent_id) = get_position_parent(Some(item_id), 0)?;

        if (next_id.is_some() && next_parent != parent_id)
            || (prev_id.is_some() && prev_parent != parent_id)
        {
            debug!("Invalid method call, Cannot move between different parent items");
            return Err(DatabaseError::InvalidData {
                field: "parent_id",
                reason: "Invalid data, all groups must be from same parent".to_string(),
            });
        }

        if prev_pos == current_pos || next_pos == current_pos {
            debug!("Item already at target position, skipping");
            return Ok(());
        }

        let mut new_pos = (prev_pos + next_pos) / 2;

        // Gap exhausted - renumber entire group
        if new_pos == prev_pos || new_pos == next_pos {
            self.renumber_position(table, parent_column, parent_id)?;

            let (prev_pos, _) = get_position_parent(prev_id, 0)?;
            let (next_pos, _) = get_position_parent(next_id, prev_pos + Self::POSITION_GAP)?;
            new_pos = (prev_pos + next_pos) / 2;
        }

        let query = format!("UPDATE {table} SET position = ?1 WHERE id = ?2");
        self.execute_db(table, item_id, &query, params![new_pos, item_id])?;

        info!(entity = table, id = item_id, "Workflow position updated");

        Ok(())
    }

    fn renumber_position(
        &self,
        table: &'static str,
        parent_column_name: Option<&'static str>,
        parent_id: Option<i64>,
    ) -> Result<()> {
        let mut connection = self.conn()?;
        let tx = connection.transaction()?;

        let mut query = format!("SELECT id FROM {table}");

        if let Some(parent_column_name) = parent_column_name {
            query.push_str(&format!(" WHERE {parent_column_name} IS ? "));
        }
        query.push_str(" ORDER BY position, id");

        let params = if parent_column_name.is_some() {
            params![parent_id]
        } else {
            params![]
        };

        let ids: Vec<i64> = tx
            .prepare(&query)?
            .query_map(params, |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;

        let update_query = format!("UPDATE {table} SET position = ? WHERE id = ?");

        for (index, id) in ids.iter().enumerate() {
            let position = (index as i64 + 1) * Self::POSITION_GAP;
            tx.execute(&update_query, params![position, id])?;
        }

        tx.commit().map_err(DatabaseError::from)
    }

    pub(crate) fn hashmap_to_string(
        hashmap: &Option<HashMap<String, String>>,
    ) -> std::result::Result<Option<String>, Error> {
        hashmap
            .as_ref()
            .map(|val| serde_json::to_string(val))
            .transpose()
    }

    pub(crate) fn string_to_hashmap(vars_json: Option<String>) -> Option<HashMap<String, String>> {
        vars_json.and_then(|json| {
            serde_json::from_str(&json).ok().or_else(|| {
                warn!("Failed to parse env_vars, using None");
                None
            })
        })
    }

    pub(crate) fn get_items_groups_commands_count(
        &self,
        table: &'static str,
        column: &'static str,
        id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<i64> {
        let mut query_builder = QueryBuilder::new();
        query_builder.add_condition(&format!("{column} IS ?"), id);

        if let Some(cid) = category_id {
            query_builder.add_condition("category_id = ?", cid);
        }
        if favorites_only {
            query_builder.add_condition_without_param("is_favorite = 1");
        }

        let (where_clause, param_refs) = query_builder.build();
        let sql_statement = format!("SELECT COUNT(*) FROM {table} {where_clause}");

        let count = self
            .query_database(&sql_statement, param_refs.as_slice(), |row| row.get(0))?
            .into_iter()
            .next()
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;
        Ok(count)
    }

    pub(crate) fn normalize_path(&self, path: &str) -> Result<String> {
        if path.is_empty() {
            return Err(DatabaseError::InvalidData {
                field: "path",
                reason: "path must not be empty".to_string(),
            });
        }

        let expanded: String = if path.starts_with('~') {
            let home = dirs::home_dir().ok_or(DatabaseError::InvalidData {
                field: "working_directory",
                reason: "Could not determine home directory".to_string(),
            })?;
            let rest = path.strip_prefix("~/").unwrap_or("");
            if rest.is_empty() {
                home.to_string_lossy().into_owned()
            } else {
                home.join(rest).to_string_lossy().into_owned()
            }
        } else {
            path.to_owned()
        };

        let canonical = Path::new(&expanded)
            .canonicalize()
            .map_err(|e| DatabaseError::InvalidData {
                field: "working_directory",
                reason: format!("Invalid path '{}': {}", path, e),
            })?;

        if !canonical.is_dir() {
            return Err(DatabaseError::InvalidData {
                field: "working_directory",
                reason: format!("Path is not a directory: '{}'", path),
            });
        }
        
        Ok(canonical.to_string_lossy().into_owned())
    }

    pub(crate) fn validate_batch_query_ids(
        &self,
        conn: &rusqlite::Connection,
        ids: &Vec<i64>,
        table: &'static str,
    )-> Result<()> {
        let id_str = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT COUNT(*) FROM {table} WHERE id IN ({})", id_str);
        let count: i64 = conn.query_row(
            &query,
            rusqlite::params_from_iter(ids),
            |row| row.get(0),
        )?;

        let id_set= HashSet::<i64>::from_iter(ids.iter().cloned());

        if count != id_set.len() as i64 {
            Err(DatabaseError::InvalidData {
                field: "ids",
                reason: "One or more IDs do not exist".to_string(),
            })?
        }

        Ok(())
    }

    pub(crate) fn replace_directory(
        &self,
        ids: Vec<i64>,
        new_directory: Option<&str>,
        table_name: &'static str,
    ) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }

        let normalized_path = if let Some(dir) = new_directory {
            Some(self.normalize_path(dir)?)
        } else {
            None
        };
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let mut conn = self.conn()?;
        let tx = conn.transaction()?;

        self.validate_batch_query_ids(&tx, &ids, table_name)?;

        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&normalized_path];
        params.extend(ids.iter().map(|i| i as &dyn rusqlite::ToSql));

        let affected = tx.execute(
            &format!(
                "UPDATE {table_name} SET working_directory = ?1 WHERE id IN ({})",
                placeholders
            ),
            params.as_slice(),
        )?;

        tx.commit()?;
        Ok(affected)
    }

    pub fn get_unique_directories(&self) -> Result<Vec<String>> {
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT DISTINCT working_directory FROM commands
         WHERE working_directory IS NOT NULL
         UNION
         SELECT DISTINCT working_directory FROM groups
         WHERE working_directory IS NOT NULL
         ORDER BY 1"
        )?;

        let dirs = stmt
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;

        Ok(dirs)
    }
}
