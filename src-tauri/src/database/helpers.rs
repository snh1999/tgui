use crate::database::Database;
use rusqlite::params;
use std::collections::HashMap;

pub use crate::database::errors::{DatabaseError, Result};

impl Database {
    pub(crate) const POSITION_GAP: i64 = 1000;

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
        let connection = self.conn();
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
        column_name: &'static str,
        group_id: Option<i64>,
    ) -> rusqlite::Result<i64> {
        let query = format!(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM {} WHERE {} IS ?1",
            table, column_name
        );

        Ok(Self::POSITION_GAP
            + (self
                .conn()
                .query_row(&query, params![group_id], |row| row.get::<_, i64>(0))?))
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
        group_column: &'static str,
        item_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
        parent_group_id: Option<i64>,
        mut get_position_parent: F,
    ) -> Result<usize>
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

        if (next_id.is_some() && next_parent != parent_group_id)
            || (prev_id.is_some() && prev_parent != parent_group_id)
        {
            return Err(DatabaseError::InvalidData {
                field: "parent_id",
                reason: "Invalid data, all groups must be from same parent".to_string(),
            });
        }

        let mut new_pos = (prev_pos + next_pos) / 2;

        // Gap exhausted - renumber entire group
        if new_pos == prev_pos || new_pos == next_pos {
            self.renumber_position(table, group_column, parent_group_id)?;

            let (prev_pos, _) = get_position_parent(prev_id, 0)?;
            let (next_pos, _) = get_position_parent(next_id, prev_pos + Self::POSITION_GAP)?;
            new_pos = (prev_pos + next_pos) / 2;
        }

        let query = format!("UPDATE {} SET position = ?1 WHERE id = ?2", table);
        let rows = self.conn().execute(&query, params![new_pos, item_id])?;
        Ok(rows)
    }

    fn renumber_position(
        &self,
        table: &str,
        column_name: &str,
        group_id: Option<i64>,
    ) -> rusqlite::Result<()> {
        let mut connection = self.conn();
        let tx = connection.transaction()?;

        // Fetch all items in current order (by position, then id as tiebreaker)
        let query = format!(
            "SELECT id FROM {} WHERE {} IS ? ORDER BY position, id",
            table, column_name
        );

        let ids: Vec<i64> = tx
            .prepare(&query)?
            .query_map(params![group_id], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;

        let update_query = format!("UPDATE {} SET position = ? WHERE id = ?", table);

        for (index, id) in ids.iter().enumerate() {
            let position = (index as i64 + 1) * Self::POSITION_GAP;
            tx.execute(&update_query, params![position, id])?;
        }

        tx.commit()
    }
}
