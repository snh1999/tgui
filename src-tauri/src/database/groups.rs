use super::{Database, DatabaseError, Group, Result};
use rusqlite::params;
use std::collections::HashSet;

impl Database {
    pub fn create_group(&self, group: &Group) -> Result<i64> {
        self.validate_group(group)?;

        let env_vars_json = group
            .env_vars
            .as_ref()
            .map(|vars| serde_json::to_string(vars))
            .transpose()?;

        let position: i64 =
            self.get_position("groups", "parent_group_id", group.parent_group_id)?;

        self.conn()?.execute(
            "INSERT INTO groups (name, description, parent_group_id, position, working_directory, env_vars, shell, category_id, is_favorite, icon)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                group.name,
                group.description,
                group.parent_group_id,
                position,
                group.working_directory,
                env_vars_json,
                group.shell,
                group.category_id,
                group.is_favorite,
                group.icon,
            ],
        )?;

        Ok(self.conn()?.last_insert_rowid())
    }

    pub fn get_group(&self, id: i64) -> Result<Group> {
        self.conn()?
            .query_row("SELECT * FROM groups WHERE id = ?1", params![id], |row| {
                Self::row_to_group(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound {
                    entity: "group",
                    id,
                },
                _ => e.into(),
            })
    }

    pub fn get_groups(
        &self,
        parent_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<Vec<Group>> {
        self.get_items(
            "groups",
            "parent_group_id",
            parent_id,
            category_id,
            favorites_only,
            Self::row_to_group,
        )
    }

    pub fn update_group(&self, group: &Group) -> Result<()> {
        self.validate_group(group)?;

        if let Some(parent_id) = group.parent_group_id {
            self.validate_no_circular_reference(group.id, parent_id)?;
        }

        let env_vars_json = group
            .env_vars
            .as_ref()
            .map(|vars| serde_json::to_string(vars))
            .transpose()?;

        let rows_affected = self.conn()?.execute(
            "UPDATE groups SET
            name = ?1,
            description = ?2,
            parent_group_id = ?3,
            working_directory = ?4,
            env_vars = ?5,
            shell = ?6,
            category_id = ?7,
icon = ?8
            WHERE id = ?9",
            params![
                group.name,
                group.description,
                group.parent_group_id,
                group.working_directory,
                env_vars_json,
                group.shell,
                group.category_id,
                group.icon,
                group.id
            ],
        )?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "group",
                id: group.id,
            });
        }

        Ok(())
    }

    pub fn move_group_between(
        &self,
        group_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        let group = self.get_group(group_id)?;

        let rows = self.move_item_between(
            "groups",
            "parent_group_id",
            group_id,
            prev_id,
            next_id,
            group.parent_group_id,
            |id, default| self.get_group_position_parent(id, default),
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound {
                entity: "group",
                id: group_id,
            });
        }

        Ok(())
    }

    fn get_group_position_parent(
        &self,
        group_id: Option<i64>,
        default_val: i64,
    ) -> Result<(i64, Option<i64>)> {
        Ok(group_id
            .map(|id| self.get_group(id).map(|c| (c.position, c.parent_group_id)))
            .transpose()?
            .unwrap_or((default_val, None)))
    }

    pub fn delete_group(&self, id: i64) -> Result<()> {
        let rows_affected = self
            .conn()?
            .execute("DELETE FROM groups WHERE id = ?1", params![id])?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "group",
                id,
            });
        }

        Ok(())
    }

    pub fn get_group_command_count(&self, id: i64) -> Result<i64> {
        self.conn()?
            .query_row(
                "SELECT COUNT(*) FROM commands WHERE group_id = ?",
                params![id],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    pub fn get_group_tree(&self, root_id: i64) -> Result<Vec<Group>> {
        self.query_database(
            "WITH RECURSIVE tree AS (
            SELECT * FROM groups WHERE id = ?1
            UNION ALL
            SELECT g.* FROM groups g
            JOIN tree t ON g.parent_group_id = t.id
        )
        SELECT * FROM tree ORDER BY position",
            params![root_id],
            Self::row_to_group,
        )
    }

    pub fn get_group_path(&self, group_id: i64) -> Result<Vec<String>> {
        let sql = "
        WITH RECURSIVE group_path AS (
            SELECT id, name, parent_group_id FROM groups WHERE id = ?
            UNION ALL
            SELECT g.id, g.name, g.parent_group_id FROM groups g
            JOIN group_path p ON g.id = p.parent_group_id
        )
        SELECT name FROM group_path;
        ";

        let mut path: Vec<String> = self
            .conn()?
            .prepare(sql)?
            .query_map(params![group_id], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;

        path.reverse();
        Ok(path)
    }

    pub fn toggle_group_favorite(&self, id: i64) -> Result<()> {
        let rows_affected = self.conn()?.execute(
            "UPDATE groups SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "group",
                id,
            });
        }

        Ok(())
    }

    fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<Group> {
        let env_vars_json: Option<String> = row.get(6)?;

        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            parent_group_id: row.get(3)?,
            position: row.get(4)?,
            working_directory: row.get(5)?,
            env_vars: env_vars_json.and_then(|json| serde_json::from_str(&json).ok()),
            shell: row.get(7)?,
            category_id: row.get(8)?,
            is_favorite: row.get(9)?,
            icon: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    fn validate_group(&self, group: &Group) -> Result<()> {
        self.validate_non_empty("name", &group.name)?;
        self.validate_env_var_keys(&group.env_vars)?;

        Ok(())
    }

    /// group walks up the parent chain to detect cycle, it is a cycle if
    /// case 1 - we find the group being updated
    /// case 2 - if we find same parent more than once
    fn validate_no_circular_reference(&self, group_id: i64, parent_id: i64) -> Result<()> {
        if group_id == parent_id {
            return Err(DatabaseError::CircularReference {
                group_id,
                parent_id,
            });
        }

        let mut current = Some(parent_id);
        let mut visited = HashSet::new();

        while let Some(id) = current {
            if id == group_id {
                return Err(DatabaseError::CircularReference {
                    group_id,
                    parent_id,
                });
            }

            if !visited.insert(id) {
                return Err(DatabaseError::CircularReference {
                    group_id,
                    parent_id,
                });
            }

            let parent_group = self.get_group(id)?;
            current = parent_group.parent_group_id;
        }

        Ok(())
    }
}
