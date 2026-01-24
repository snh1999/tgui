use super::{Command, Database, DatabaseError, Result};
use rusqlite::params;

impl Database {
    pub fn create_command(&self, cmd: &Command) -> Result<i64> {
        self.validate_command(cmd)?;
        let arguments_json = serde_json::to_string(&cmd.arguments)?;
        let env_vars_json = cmd
            .env_vars
            .as_ref()
            .map(|vars| serde_json::to_string(vars))
            .transpose()?;

        let position = self.get_position("commands", "group_id", cmd.group_id)?;

        self.conn().execute(
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
        )?;

        Ok(self.conn().last_insert_rowid())
    }

    pub fn get_command(&self, id: i64) -> Result<Command> {
        self.conn()
            .query_row("SELECT * FROM commands WHERE id = ?1", params![id], |row| {
                Self::row_to_command(row)
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound {
                    entity: "command",
                    id,
                },
                _ => e.into(),
            })
    }

    pub fn get_commands(
        &self,
        group_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<Vec<Command>> {
        let mut sql_statement =
            "SELECT * FROM commands WHERE group_id IS ?1 AND category_id IS ?2".to_string();

        if favorites_only {
            sql_statement.push_str(" AND is_favorite = 1");
        }

        sql_statement.push_str(" ORDER BY position");

        self.query_database(
            &sql_statement,
            params![group_id, category_id],
            Self::row_to_command,
        )
    }

    pub fn search_commands(&self, search_term: &str) -> Result<Vec<Command>> {
        let pattern = format!("%{}%", search_term);
        self.query_database(
            "SELECT * FROM commands
         WHERE name LIKE ?1 OR command LIKE ?1 OR description LIKE ?1
         ORDER BY is_favorite DESC, updated_at DESC",
            params![pattern],
            Self::row_to_command,
        )
    }

    pub fn update_command(&self, cmd: &Command) -> Result<()> {
        self.validate_command(cmd)?;

        let args_json = serde_json::to_string(&cmd.arguments)?;
        let env_vars_json = cmd
            .env_vars
            .as_ref()
            .map(|vars| serde_json::to_string(vars))
            .transpose()?;

        let rows_affected = self.conn().execute(
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
                args_json,
                cmd.description,
                cmd.group_id,
                cmd.working_directory,
                env_vars_json,
                cmd.shell,
                cmd.category_id,
                cmd.is_favorite,
                cmd.id
            ],
        )?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "command",
                id: cmd.id,
            });
        }

        Ok(())
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

    /// Move command between two positions (calculates midpoint)
    /// prev_id None means move to top
    /// next_id None means move to bottom
    pub fn move_command_between(
        &self,
        cmd_id: i64,
        prev_id: Option<i64>,
        next_id: Option<i64>,
    ) -> Result<()> {
        let cmd = self.get_command(cmd_id)?;

        let rows = self.move_item_between(
            "commands",
            "group_id",
            cmd_id,
            prev_id,
            next_id,
            cmd.group_id,
            |id, default| self.get_position_parent_command(id, default),
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound {
                entity: "command",
                id: cmd_id,
            });
        }
        Ok(())
    }

    pub fn delete_command(&self, id: i64) -> Result<()> {
        let rows_affected = self
            .conn()
            .execute("DELETE FROM commands WHERE id = ?1", params![id])?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "command",
                id,
            });
        }

        Ok(())
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<()> {
        let rows_affected = self.conn().execute(
            "UPDATE commands SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "command",
                id,
            });
        }

        Ok(())
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

        Ok(Command {
            id: row.get(0)?,
            name: row.get(1)?,
            command: row.get(2)?,
            arguments: serde_json::from_str(&args_json).unwrap_or_default(),
            description: row.get(4)?,
            group_id: row.get(5)?,
            position: row.get(6)?,
            working_directory: row.get(7)?,
            env_vars: env_vars_json.and_then(|json| serde_json::from_str(&json).ok()),
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
