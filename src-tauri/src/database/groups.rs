use super::{CategoryFilter, Command, Database, DatabaseError, Group, GroupFilter, GroupNode, Result};
use crate::constants::{COMMANDS_TABLE, COMMAND_GROUP_COLUMN, GROUPS_TABLE, GROUP_PARENT_GROUP_COLUMN};
use crate::database::helpers::QueryBuilder;
use rusqlite::{named_params, params};
use std::collections::{HashMap, HashSet};
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

        let working_directory = if let Some(dir) = &group.working_directory {
            Some(self.normalize_path(dir)?)
        } else {
            None
        };

        self.create(
            GROUPS_TABLE,
            "INSERT INTO groups (name, description, parent_group_id, position, working_directory, env_vars, shell, category_id, is_favorite, icon, color)
             VALUES (:name, :description, :parent_group_id, :position, :working_directory, :env_vars, :shell, :category_id, :is_favorite, :icon, :color)",
            named_params! {
                ":name": group.name,
                ":description": group.description,
                ":parent_group_id": group.parent_group_id,
                ":position": position,
                ":working_directory": working_directory,
                ":env_vars": env_vars,
                ":shell": group.shell,
                ":category_id": group.category_id,
                ":is_favorite": group.is_favorite,
                ":icon": group.icon,
                ":color": group.color,
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
        parent_id: GroupFilter,
        category_id: CategoryFilter,
        favorites_only: bool,
    ) -> Result<Vec<Group>> {
        let mut query_builder = QueryBuilder::new();
        match parent_id {
            GroupFilter::Group(id) => {
                query_builder.add_condition("parent_group_id = ?", id);
            }
            GroupFilter::None => {
                query_builder.add_condition_without_param("parent_group_id IS NULL");
            }
            GroupFilter::All => {}
        }

        match category_id {
            CategoryFilter::Category(id) => {
                query_builder.add_condition("category_id = ?", id);
            }
            CategoryFilter::None => {
                query_builder.add_condition_without_param("category_id IS NULL");
            }
            CategoryFilter::All => {}
        }

        if favorites_only {
            query_builder.add_condition_without_param("is_favorite = 1");
        }

        let (where_clause, param_refs) = query_builder.build();

        let query = format!("SELECT * FROM groups {where_clause} ORDER BY position");

        self.query_database(&query, param_refs.as_slice(), Self::row_to_group)
    }

    #[instrument(skip(self))]
    pub fn get_groups_count(
        &self,
        group_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<i64> {
        self.get_items_groups_commands_count(
            GROUPS_TABLE,
            GROUP_PARENT_GROUP_COLUMN,
            group_id,
            category_id,
            favorites_only,
        )
    }

    pub fn search_groups(&self, search_term: &str) -> Result<Vec<Group>> {
        let pattern = format!("%{}%", search_term);
        self.query_database(
            "SELECT * FROM groups WHERE name LIKE ? OR description LIKE ? ORDER BY name",
            params![&pattern, &pattern],
            Self::row_to_group,
        )
    }

    #[instrument(skip(self))]
    pub fn update_group(&self, group: &Group) -> Result<()> {
        self.validate_group(group)?;

        let old_group = self.get_group(group.id)?;
        if old_group.parent_group_id != group.parent_group_id {
            if let Some(parent_id) = group.parent_group_id {
                self.validate_no_circular_reference(group.id, parent_id)?;
            }
            self.update_parent_group(
                GROUPS_TABLE,
                GROUP_PARENT_GROUP_COLUMN,
                group.id,
                group.parent_group_id,
            )?;
        }

        let env_vars = Self::hashmap_to_string(&group.env_vars)?;
        let working_directory = if let Some(dir) = &group.working_directory {
            Some(self.normalize_path(dir)?)
        } else {
            None
        };

        debug!(
            command_id = group.id,
            has_env_vars = group.env_vars.is_some(),
            "Updating Group"
        );

        self.execute_db(
            GROUPS_TABLE,
            group.id,
            "UPDATE groups SET
            name = :name,
            description = :description,
            working_directory = :working_directory,
            env_vars = :env_vars,
            shell = :shell,
            category_id = :category_id,
            icon = :icon,
            color = :color
            WHERE id = :id",
            named_params! {
                ":name": group.name,
                ":description": group.description,
                ":working_directory": working_directory,
                ":env_vars": env_vars,
                ":shell": group.shell,
                ":category_id": group.category_id,
                ":icon": group.icon,
                ":color": group.color,
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
        self.move_item_between(
            "groups",
            group_id,
            prev_id,
            next_id,
            Some(GROUP_PARENT_GROUP_COLUMN),
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
        self.execute_db(
            GROUPS_TABLE,
            id,
            "DELETE FROM groups WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn get_group_tree(&self, root_id: i64) -> Result<GroupNode> {
        let flat = self.query_database(
            "WITH RECURSIVE tree AS (
            SELECT * FROM groups WHERE id = ?1
            UNION ALL
            SELECT g.* FROM groups g
            JOIN tree t ON g.parent_group_id = t.id
        )
        SELECT * FROM tree ORDER BY position",
            params![root_id],
            Self::row_to_group,
        )?;

        if flat.len() == 0 {
            return Err(DatabaseError::NotFound {
                id: root_id,
                entity: GROUPS_TABLE,
            });
        }

        let mut children_map: HashMap<i64, Vec<i64>> = HashMap::new();
        for g in &flat {
            if let Some(pid) = g.parent_group_id {
                children_map.entry(pid).or_default().push(g.id);
            }
        }

        let mut nodes: HashMap<i64, Group> = flat.into_iter().map(|g| (g.id, g)).collect();

        fn build(
            id: i64,
            nodes: &mut HashMap<i64, Group>,
            children_map: &HashMap<i64, Vec<i64>>,
        ) -> Result<GroupNode> {
            let group = nodes.remove(&id).ok_or(DatabaseError::InvalidData {
                field: "group_id",
                reason: format!("Group {} missing during tree assembly", id),
            })?;
            let children = children_map
                .get(&id)
                .map(|ids| {
                    ids.iter()
                        .map(|&cid| build(cid, nodes, children_map))
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            Ok(GroupNode { group, children })
        }

        build(root_id, &mut nodes, &children_map)
    }

    /// Walks the parent chain from group_id upward.
    /// Returns groups ordered closest-first (direct parent first, root last).
    pub fn get_group_ancestor_chain(&self, group_id: i64) -> Result<Vec<Group>> {
        self.query_database(
            "WITH RECURSIVE chain AS (
            SELECT * FROM groups WHERE id = ?1
            UNION ALL
            SELECT g.* FROM groups g
            JOIN chain c ON g.id = c.parent_group_id
        )
        SELECT * FROM chain",
            params![group_id],
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

        self.execute_db(
            GROUPS_TABLE,
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
            color: row.get("color")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    fn validate_group(&self, group: &Group) -> Result<()> {
        self.validate_field_length("name", &group.name, Self::MAX_NAME_LENGTH)?;
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

        let chain = self.get_group_ancestor_chain(parent_id)?;

        // TODO: is it the same code?
        // if chain.iter().any(|g| g.id == group_id) {
        //     return Err(DatabaseError::CircularReference { group_id, parent_id });
        // }
        // Ok(())

        let mut visited = HashSet::new();

        for i in 0..chain.len() {
            if chain[i].id == group_id || !visited.insert(chain[i].id) {
                return Err(DatabaseError::CircularReference {
                    group_id,
                    parent_id,
                });
            }
        }

        Ok(())
    }

    pub fn get_groups_by_directory(&self, directory: Option<&str>) -> Result<Vec<Group>> {
        let normalized_path = if let Some(dir) = directory {
            Some(self.normalize_path(dir)?)
        } else {
            None
        };
        self.query_database(
            "SELECT * FROM groups WHERE working_directory IS ?1 ORDER BY position",
            params![normalized_path],
            Self::row_to_group,
        )
    }

    #[instrument(skip(self, ids))]
    pub fn replace_groups_directory(
        &self,
        ids: Vec<i64>,
        new_directory: Option<&str>,
    ) -> Result<usize> {
        self.replace_directory(ids, new_directory, GROUPS_TABLE)
    }

    #[instrument(skip(self, ids))]
    pub fn duplicate_groups(
        &self,
        ids: Vec<i64>,
        name_prefix: &str,
        recursive: bool,
    ) -> Result<Vec<i64>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        self.validate_batch_query_ids(&tx, &ids, GROUPS_TABLE)?;

        let mut all_groups: Vec<(Group, bool)> = Vec::new();
        let mut queue: Vec<i64> = ids.to_vec();
        let mut visited: HashSet<i64> = HashSet::new();
        let mut is_root = true;

        while !queue.is_empty() {
            let current_ids = std::mem::take(&mut queue);
            let placeholders = current_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let groups: Vec<Group> = tx
                .prepare(&format!("SELECT * FROM groups WHERE id IN ({})", placeholders))?
                .query_map(rusqlite::params_from_iter(&current_ids), Self::row_to_group)?
                .collect::<rusqlite::Result<Vec<_>>>()?;

            for group in groups {
                if visited.insert(group.id) {
                    if recursive {
                        // Queue children
                        let child_ids: Vec<i64> = tx
                            .prepare("SELECT id FROM groups WHERE parent_group_id = ?1")?
                            .query_map(params![group.id], |row| row.get(0))?
                            .collect::<rusqlite::Result<Vec<_>>>()?;
                        queue.extend(child_ids);
                    }
                    all_groups.push((group, is_root));
                }
            }
            is_root = false; // Only first level are roots
        }

        let mut id_mapping =   HashMap::<i64, i64>::new();

        let mut new_ids = Vec::with_capacity(all_groups.len());


        for (original, is_root) in &all_groups {
            let new_name = if *is_root {
                format!("{}{}", name_prefix, original.name)
            } else {
                original.name.clone()
            };

            // Remap parent: if parent was duplicated, point to new parent; else keep original
            let new_parent = original.parent_group_id
                .map(|pid| id_mapping.get(&pid).copied().unwrap_or(pid));

            let new_position = Self::get_position_with_conn(
                &tx,
                GROUPS_TABLE,
                Some("parent_group_id"),
                new_parent,
            )?;

            let env_vars = Self::hashmap_to_string(&original.env_vars)?;

            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO groups
                 (name, description, parent_group_id, position, working_directory, env_vars, shell, category_id, is_favorite, icon, color)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )?;
                stmt.execute(params![
                new_name,
                original.description,
                new_parent,
                new_position,
                original.working_directory,
                env_vars,
                original.shell,
                original.category_id,
                false,
                original.icon,
                original.color,
            ])?;
            }

            let new_id = tx.last_insert_rowid();
            id_mapping.insert(original.id, new_id);
            new_ids.push(new_id);



            if recursive {
                let commands = self.get_commands_in_tx(&tx, original.id)?;
                self.duplicate_commands_under_parents(&tx, commands, Some(new_id), "")?;
            }
        }

        tx.commit()?;
        Ok(new_ids)
    }

    fn get_commands_in_tx(&self, tx: &rusqlite::Transaction, group_id: i64) -> Result<Vec<Command>> {
        let mut stmt = tx.prepare("SELECT * FROM commands WHERE group_id = ?1")?;
        let commands = stmt
            .query_map(params![group_id], Self::row_to_command)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(commands)
    }
}
