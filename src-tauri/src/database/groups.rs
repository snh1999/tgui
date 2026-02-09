use super::{Database, DatabaseError, Group, Result};
use crate::constants::{GROUPS_TABLE, GROUP_PARENT_GROUP_COLUMN};
use rusqlite::{named_params, params};
use std::collections::HashSet;
use tracing::{debug, instrument, warn};

impl Database {
    #[instrument(skip(self, group), fields(name = %group.name))]
    pub fn create_group(&self, group: &Group) -> Result<i64> {
        self.validate_group(group)?;
        let env_vars = Self::hashmap_to_string(&group.env_vars)?;

        let position: i64 = self.get_position(
            GROUPS_TABLE,
            Some(GROUP_PARENT_GROUP_COLUMN),
            group.parent_group_id,
        )?;

        self.create(
            GROUPS_TABLE,
            "INSERT INTO groups (name, description, parent_group_id, position, working_directory, env_vars, shell, category_id, is_favorite, icon)
             VALUES (:name, :description, :parent_group_id, :position, :working_directory, :env_vars, :shell, :category_id, :is_favorite, :icon)",
            named_params! {
                ":name": group.name,
                ":description": group.description,
                ":parent_group_id": group.parent_group_id,
                ":position": position,
                ":working_directory": group.working_directory,
                ":env_vars": env_vars,
                ":shell": group.shell,
                ":category_id": group.category_id,
                ":is_favorite": group.is_favorite,
                ":icon": group.icon,
            },
        )
    }

    #[instrument(skip(self), ret)]
    pub fn get_group(&self, id: i64) -> Result<Group> {
        self.query_row(
            GROUPS_TABLE,
            id,
            "SELECT * FROM groups WHERE id = ?1",
            Self::row_to_group,
        )
    }

    #[instrument(skip(self))]
    pub fn get_groups(
        &self,
        parent_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<Vec<Group>> {
        self.get_items_groups_commands(
            GROUPS_TABLE,
            GROUP_PARENT_GROUP_COLUMN,
            parent_id,
            category_id,
            favorites_only,
            Self::row_to_group,
        )
    }

    #[instrument(skip(self))]
    pub fn update_group(&self, group: &Group) -> Result<()> {
        self.validate_group(group)?;

        if let Some(parent_id) = group.parent_group_id {
            self.validate_no_circular_reference(group.id, parent_id)?;
        }

        let env_vars = Self::hashmap_to_string(&group.env_vars)?;

        debug!(
            command_id = group.id,
            has_env_vars = group.env_vars.is_some(),
            "Updating Group"
        );

        self.update(
            GROUPS_TABLE,
            "UPDATE",
            group.id,
            "UPDATE groups SET
            name = :name,
            description = :description,
            parent_group_id = :parent_group_id,
            working_directory = :working_directory,
            env_vars = :env_vars,
            shell = :shell,
            category_id = :category_id,
            icon = :icon
            WHERE id = :id",
            named_params! {
                ":name": group.name,
                ":description": group.description,
                ":parent_group_id": group.parent_group_id,
                ":working_directory": group.working_directory,
                ":env_vars": env_vars,
                ":shell": group.shell,
                ":category_id": group.category_id,
                ":icon": group.icon,
                ":id": group.id
            },
        )
    }

    #[instrument(skip(self))]
    pub fn move_group_between(
        &self,
        group_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        let group = self.get_group(group_id)?;

        self.move_item_between(
            "groups",
            group_id,
            prev_id,
            next_id,
            Some(GROUP_PARENT_GROUP_COLUMN),
            group.parent_group_id,
            |id, default| self.get_group_position_parent(id, default),
        )
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

    #[instrument(skip(self))]
    pub fn delete_group(&self, id: i64) -> Result<()> {
        self.update(
            GROUPS_TABLE,
            "DELETE",
            id,
            "DELETE FROM groups WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn get_group_command_count(&self, id: i64) -> Result<i64> {
        self.query_row(
            GROUPS_TABLE,
            id,
            "SELECT COUNT(*) FROM commands WHERE group_id = ?",
            |row| row.get(0),
        )
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    pub fn toggle_group_favorite(&self, id: i64) -> Result<()> {
        debug!(command_id = id, "Toggling favorite");

        self.update(
            GROUPS_TABLE,
            "UPDATE",
            id,
            "UPDATE groups SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )
    }

    fn row_to_group(row: &rusqlite::Row) -> rusqlite::Result<Group> {
        let env_vars_json: Option<String> = row.get("env_vars")?;
        let env_vars = Self::string_to_hashmap(env_vars_json);

        Ok(Group {
            id: row.get("id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            parent_group_id: row.get("parent_group_id")?,
            position: row.get("position")?,
            working_directory: row.get("working_directory")?,
            env_vars,
            shell: row.get("shell")?,
            category_id: row.get("category_id")?,
            is_favorite: row.get("is_favorite")?,
            icon: row.get("icon")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
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
