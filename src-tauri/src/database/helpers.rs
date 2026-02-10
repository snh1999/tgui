pub use crate::database::errors::{DatabaseError, Result};
use crate::database::Database;
use crate::db_span;
use rusqlite::params;
use serde_json::Error;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

impl Database {
    pub(crate) const POSITION_GAP: i64 = 1000;

    pub(crate) fn create<P: rusqlite::Params>(
        &self,
        table: &'static str,
        sql: &str,
        params: P,
    ) -> Result<i64> {
        self.execute(
            sql,
            params,
            "INSERT",
            table,
            Some(&format!("Failed to create {}", table)),
        )?;
        // TODO explore the unlikely possibility of another insert
        let row_id = self.conn()?.last_insert_rowid();
        info!(id = row_id, entity = table, "created successfully");
        Ok(row_id)
    }

    pub(crate) fn update<P: rusqlite::Params>(
        &self,
        table: &'static str,
        operation: &'static str,
        id: i64,
        sql: &str,
        params: P,
    ) -> Result<()> {
        let rows_affected = self.execute(
            sql,
            params,
            operation,
            table,
            Some(&format!("{} operation failed on {}", operation, table)),
        )?;

        if rows_affected == 0 {
            error!(id = id, table = table, "Not found");
            return Err(DatabaseError::NotFound { entity: table, id });
        }

        info!(id = id, "{} operation successful on {}", operation, table);
        Ok(())
    }

    fn execute<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
        operation: &'static str,
        table: &'static str,
        error_message: Option<&str>,
    ) -> Result<usize> {
        let span = db_span!(operation, table, 0);
        let _enter = span.enter();

        self.conn()?.execute(sql, params).map_err(|e| {
            if let Some(msg) = error_message {
                error!(error = %e, message = msg);
            }
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
        debug!(id = id, "{}", format!("Fetching {}", table));

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

        debug!(id = id, "{}", format!("{} found", table));

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
        let mut query = format!("SELECT COALESCE(MAX(position), -1) + 1 FROM {} ", table);

        if let Some(parent_column) = parent_column {
            query.push_str(&format!(" WHERE {} IS ?1", parent_column));
        }

        let params = if parent_column.is_some() {
            params![parent_id]
        } else {
            params![]
        };

        let position = Self::POSITION_GAP
            + (self
                .conn()?
                .query_row(&query, params, |row| row.get::<_, i64>(0))?);

        Ok(position)
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
                    return Err(DatabaseError::InvalidData {
                        field: "env_vars",
                        reason: format!(
                            "Invalid key '{}': only alphanumeric, underscore, dash",
                            key
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_non_empty(&self, field: &'static str, value: &str) -> Result<()> {
        if value.trim().is_empty() {
            Err(DatabaseError::InvalidData {
                field,
                reason: format!("{} cannot be empty", field),
            })
        } else {
            Ok(())
        }
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
        parent_id: Option<i64>,
        mut get_position_parent: F,
    ) -> Result<()>
    where
        F: FnMut(Option<i64>, i64) -> Result<(i64, Option<i64>)>,
    {
        if prev_id.is_none() && next_id.is_none() {
            return Err(DatabaseError::InvalidData {
                field: "item_id",
                reason: "Invalid positons. Either prev_id or next_id must be provided".to_string(),
            });
        }

        let (prev_pos, prev_parent) = get_position_parent(prev_id, 0)?;
        let (next_pos, next_parent) = get_position_parent(next_id, prev_pos + Self::POSITION_GAP)?;

        if (next_id.is_some() && next_parent != parent_id)
            || (prev_id.is_some() && prev_parent != parent_id)
        {
            return Err(DatabaseError::InvalidData {
                field: "parent_id",
                reason: "Invalid data, all groups must be from same parent".to_string(),
            });
        }

        let mut new_pos = (prev_pos + next_pos) / 2;

        // Gap exhausted - renumber entire group
        if new_pos == prev_pos || new_pos == next_pos {
            self.renumber_position(table, parent_column, parent_id)?;

            let (prev_pos, _) = get_position_parent(prev_id, 0)?;
            let (next_pos, _) = get_position_parent(next_id, prev_pos + Self::POSITION_GAP)?;
            new_pos = (prev_pos + next_pos) / 2;
        }

        let query = format!("UPDATE {} SET position = ?1 WHERE id = ?2", table);
        let rows = self.conn()?.execute(&query, params![new_pos, item_id])?;

        if rows == 0 {
            return Err(DatabaseError::NotFound {
                entity: table,
                id: item_id,
            });
        }

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

        let mut query = format!("SELECT id FROM {}", table);

        if let Some(parent_column_name) = parent_column_name {
            query.push_str(&format!(" WHERE {} IS ? ", parent_column_name));
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

        let update_query = format!("UPDATE {} SET position = ? WHERE id = ?", table);

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

    pub(crate) fn get_items_groups_commands<T, F>(
        &self,
        table: &'static str,
        column: &'static str,
        id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
        row_mapper: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut sql_statement = format!("SELECT * FROM {} WHERE {} IS ?1", table, column);
        params.push(Box::new(id));

        if let Some(cid) = category_id {
            sql_statement.push_str(&format!(" AND category_id = ?{}", params.len() + 1));
            params.push(Box::new(cid));
        }

        if favorites_only {
            sql_statement.push_str(" AND is_favorite = 1");
        }

        sql_statement.push_str(" ORDER BY position");

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        self.query_database(&sql_statement, &*param_refs, row_mapper)
    }
}
