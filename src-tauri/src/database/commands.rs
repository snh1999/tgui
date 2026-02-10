use super::{Command, Database, DatabaseError, Result};
use crate::constants::{COMMANDS_TABLE, COMMAND_GROUP_COLUMN};
use rusqlite::{named_params, params};
use tracing::{debug, instrument, warn};

impl Database {
    #[instrument(skip(self, cmd), fields(name = %cmd.name))]
    pub fn create_command(&self, cmd: &Command) -> Result<i64> {
        self.validate_command(cmd)?;
        let arguments_json = serde_json::to_string(&cmd.arguments)?;
        let env_vars_json = Self::hashmap_to_string(&cmd.env_vars)?;

        let position =
            self.get_position(COMMANDS_TABLE, Some(COMMAND_GROUP_COLUMN), cmd.group_id)?;

        debug!(calculated_position = position, "Command position");

        self.create(
            COMMANDS_TABLE,
            "INSERT INTO
            commands (name, command, arguments, description, group_id, position, working_directory, env_vars, shell, category_id, is_favorite)
             VALUES (:name, :command, :arguments, :description, :group_id, :position, :working_directory, :env_vars, :shell, :category_id, :is_favorite)",
            named_params! {
                ":name": cmd.name,
                ":command": cmd.command,
                ":arguments": arguments_json,
                ":description": cmd.description,
                ":group_id": cmd.group_id,
                ":position": position,
                ":working_directory": cmd.working_directory,
                ":env_vars": env_vars_json,
                ":shell": cmd.shell,
                ":category_id": cmd.category_id,
                ":is_favorite": cmd.is_favorite,
            },
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
            name = :name,
            command = :command,
            arguments = :arguments,
            description = :description,
            group_id = :group_id,
            working_directory = :working_directory,
            env_vars = :env_vars,
            shell = :shell,
            category_id = :category_id,
            is_favorite = :is_favorite
            WHERE id = :id",
            named_params! {
                ":name": cmd.name,
                ":command": cmd.command,
                ":arguments": arguments,
                ":description": cmd.description,
                ":group_id": cmd.group_id,
                ":working_directory": cmd.working_directory,
                ":env_vars": env_vars,
                ":shell": cmd.shell,
                ":category_id": cmd.category_id,
                ":is_favorite": cmd.is_favorite,
                ":id": cmd.id
            },
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

        self.move_item_between(
            COMMANDS_TABLE,
            cmd_id,
            prev_id,
            next_id,
            Some(COMMAND_GROUP_COLUMN),
            cmd.group_id,
            |id, default| self.get_position_parent_command(id, default),
        )
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
        let args_str: String = row.get("arguments")?;
        let env_vars_str: Option<String> = row.get("env_vars")?;

        let arguments = serde_json::from_str(&args_str).unwrap_or_else(|e| {
            warn!(error = %e, "Failed to parse arguments, using default");
            Vec::new()
        });

        let env_vars = Self::string_to_hashmap(env_vars_str);

        Ok(Command {
            id: row.get("id")?,
            name: row.get("name")?,
            command: row.get("command")?,
            arguments,
            description: row.get("description")?,
            group_id: row.get("group_id")?,
            position: row.get("position")?,
            working_directory: row.get("working_directory")?,
            env_vars,
            shell: row.get("shell")?,
            category_id: row.get("category_id")?,
            is_favorite: row.get("is_favorite")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
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
