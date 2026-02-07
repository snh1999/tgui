use super::{Command, Database, DatabaseError, Result};
use crate::constants::{COMMANDS_TABLE, COMMAND_GROUP_COLUMN};
use rusqlite::params;
use tracing::{debug, instrument, warn};

impl Database {
    #[instrument(skip(self, cmd), fields(name = %cmd.name))]
    pub fn create_command(&self, cmd: &Command) -> Result<i64> {
        self.validate_command(cmd)?;
        let arguments_json = serde_json::to_string(&cmd.arguments)?;
        let env_vars_json = Self::hashmap_to_string(&cmd.env_vars)?;

        let position = self.get_position(COMMANDS_TABLE, COMMAND_GROUP_COLUMN, cmd.group_id)?;
        debug!(calculated_position = position, "Command position");

        self.create(
            COMMANDS_TABLE,
            "INSERT INTO
            commands (name, command, arguments, description, group_id, position, working_directory, env_vars, shell, category_id, is_favorite)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                cmd.name,
                cmd.command,
                arguments_json,
                cmd.description,
                cmd.group_id,
                position,
                cmd.working_directory,
                env_vars_json,
                cmd.shell,
                cmd.category_id,
                cmd.is_favorite,
            ],
        )
    }

    #[instrument(skip(self))]
    pub fn get_command(&self, id: i64) -> Result<Command> {
        self.query_row(
            COMMANDS_TABLE,
            id,
            "SELECT * FROM commands WHERE id = ?1",
            Self::row_to_command,
        )
    }

    #[instrument(skip(self))]
    pub fn get_commands(
        &self,
        group_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<Vec<Command>> {
        self.get_items_groups_commands(
            COMMANDS_TABLE,
            COMMAND_GROUP_COLUMN,
            group_id,
            category_id,
            favorites_only,
            Self::row_to_command,
        )
    }

    #[instrument(skip(self))]
    pub fn search_commands(&self, search_term: &str) -> Result<Vec<Command>> {
        debug!(search_term_length = search_term.len(), "Searching commands");

        let pattern = format!("%{}%", search_term);
        self.query_database(
            "SELECT * FROM commands
         WHERE name LIKE ?1 OR command LIKE ?1 OR description LIKE ?1
         ORDER BY is_favorite DESC, updated_at DESC",
            params![pattern],
            Self::row_to_command,
        )
    }

    #[instrument(skip(self))]
    pub fn update_command(&self, cmd: &Command) -> Result<()> {
        self.validate_command(cmd)?;

        let arguments = serde_json::to_string(&cmd.arguments)?;
        let env_vars = Self::hashmap_to_string(&cmd.env_vars)?;

        debug!(
            command_id = cmd.id,
            has_env_vars = cmd.env_vars.is_some(),
            "Updating command"
        );

        self.update(
            COMMANDS_TABLE,
            "UPDATE",
            cmd.id,
            "UPDATE commands SET
            name = ?1,
            command = ?2,
            arguments = ?3,
            description = ?4,
            group_id = ?5,
            working_directory = ?6,
            env_vars = ?7,
            shell = ?8,
            category_id = ?9,
            is_favorite = ?10
            WHERE id = ?11",
            params![
                cmd.name,
                cmd.command,
                arguments,
                cmd.description,
                cmd.group_id,
                cmd.working_directory,
                env_vars,
                cmd.shell,
                cmd.category_id,
                cmd.is_favorite,
                cmd.id
            ],
        )
    }

    fn get_position_parent_command(
        &self,
        cmd_id: Option<i64>,
        default_val: i64,
    ) -> Result<(i64, Option<i64>)> {
        Ok(cmd_id
            .map(|id| self.get_command(id).map(|c| (c.position, c.group_id)))
            .transpose()?
            .unwrap_or((default_val, None)))
    }

    #[instrument(skip(self))]
    pub fn move_command_between(
        &self,
        cmd_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        let cmd = self.get_command(cmd_id)?;

        let rows = self.move_item_between(
            COMMANDS_TABLE,
            COMMAND_GROUP_COLUMN,
            cmd_id,
            prev_id,
            next_id,
            cmd.group_id,
            |id, default| self.get_position_parent_command(id, default),
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound {
                entity: COMMANDS_TABLE,
                id: cmd_id,
            });
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn delete_command(&self, id: i64) -> Result<()> {
        self.update(
            COMMANDS_TABLE,
            "DELETE",
            id,
            "DELETE FROM commands WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn toggle_command_favorite(&self, id: i64) -> Result<()> {
        debug!(command_id = id, "Toggling favorite");
        self.update(
            COMMANDS_TABLE,
            "UPDATE",
            id,
            "UPDATE commands SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )
    }

    /// the arguments/env_vars data is not being cross validated because
    /// 1. the data has already validation via rust type system (check create and update)
    /// 2. this would render the application in a stuck state as every get operation depends on this function
    /// -- FE requires updating to even view the command, and to view the command we require fetching the command
    /// -- If the default value is returned, user at least can retrieve the command/update with new value
    /// -- NOTE: we have to check before running the commands
    fn row_to_command(row: &rusqlite::Row) -> rusqlite::Result<Command> {
        let args_json: String = row.get(3)?;
        let env_vars_json: Option<String> = row.get(8)?;

        let arguments = serde_json::from_str(&args_json).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to parse arguments, using default");
            Vec::new()
        });

        let env_vars = env_vars_json.and_then(|json| {
            serde_json::from_str(&json).ok().or_else(|| {
                warn!("Failed to parse env_vars, using None");
                None
            })
        });

        Ok(Command {
            id: row.get(0)?,
            name: row.get(1)?,
            command: row.get(2)?,
            arguments,
            description: row.get(4)?,
            group_id: row.get(5)?,
            position: row.get(6)?,
            working_directory: row.get(7)?,
            env_vars,
            shell: row.get(9)?,
            category_id: row.get(10)?,
            is_favorite: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }

    /// Name, Command must not be empty
    /// env var keys (alphanumeric + underscore + dash only)
    fn validate_command(&self, cmd: &Command) -> Result<()> {
        self.validate_non_empty("name", &cmd.name)?;
        self.validate_non_empty("command", &cmd.command)?;
        self.validate_env_var_keys(&cmd.env_vars)?;

        Ok(())
    }
}
