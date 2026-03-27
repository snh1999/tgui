use super::{
    CategoryFilter, Command, Database, ExecutionHistory, ExecutionStatus, GroupFilter, Result,
    TriggeredBy, WithHistory,
};
use crate::constants::{COMMANDS_TABLE, COMMAND_GROUP_COLUMN};
use crate::database::helpers::QueryBuilder;
use rusqlite::{named_params, params};
use tracing::{debug, instrument, warn};

impl Database {
    const EXECUTION_HISTORY_SELECT: &str = "SELECT c.*,
    h.id as h_id, h.status as h_status, h.exit_code as h_exit_code,
    h.started_at as h_started_at, h.completed_at as h_completed_at,
    h.triggered_by as h_triggered_by, h.context as h_context,
    h.pid as h_pid, h.workflow_id as h_workflow_id,
    h.workflow_step_id as h_workflow_step_id,
    h.command_id as h_command_id
    FROM commands c";

    #[instrument(skip(self, cmd), fields(name = %cmd.name))]
    pub fn create_command(&self, cmd: &Command) -> Result<i64> {
        self.validate_command(cmd)?;
        let arguments_json = serde_json::to_string(&cmd.arguments)?;
        let env_vars_json = Self::hashmap_to_string(&cmd.env_vars)?;

        let position =
            self.get_position(COMMANDS_TABLE, Some(COMMAND_GROUP_COLUMN), cmd.group_id)?;

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
    pub fn get_commands_count(
        &self,
        group_id: Option<i64>,
        category_id: Option<i64>,
        favorites_only: bool,
    ) -> Result<i64> {
        self.get_items_groups_commands_count(
            COMMANDS_TABLE,
            COMMAND_GROUP_COLUMN,
            group_id,
            category_id,
            favorites_only,
        )
    }

    #[instrument(skip(self))]
    pub fn get_commands(
        &self,
        group_id: GroupFilter,
        category_id: CategoryFilter,
        favorites_only: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<WithHistory<Command>>> {
        let mut query_builder = QueryBuilder::new();

        match group_id {
            GroupFilter::Group(id) => {
                query_builder.add_condition("c.group_id = ?", id);
            }
            GroupFilter::None => {
                query_builder.add_condition_without_param("c.group_id IS NULL");
            }
            GroupFilter::All => {}
        }

        match category_id {
            CategoryFilter::Category(id) => {
                query_builder.add_condition("c.category_id = ?", id);
            }
            CategoryFilter::None => {
                query_builder.add_condition_without_param("c.category_id IS NULL");
            }
            CategoryFilter::All => {}
        }

        if favorites_only {
            query_builder.add_condition_without_param("c.is_favorite = 1");
        }

        let (where_clause, param_refs) = query_builder.build();

        let pagination = match (limit, offset) {
            (Some(l), Some(o)) => format!("LIMIT {} OFFSET {}", l, o),
            (Some(l), None) => format!("LIMIT {}", l),
            (None, Some(o)) => format!("LIMIT -1 OFFSET {}", o),
            (None, None) => String::new(),
        };

        let query = format!(
            "{}
         LEFT JOIN (
             SELECT *, ROW_NUMBER() OVER (PARTITION BY command_id ORDER BY started_at DESC) as rn
             FROM execution_history
             WHERE workflow_id IS NULL
         ) h ON c.id = h.command_id AND h.rn = 1
         {where_clause}
         ORDER BY c.position
         {pagination}",
            Self::EXECUTION_HISTORY_SELECT
        );

        self.conn()?
            .prepare(&query)?
            .query_map(param_refs.as_slice(), Self::row_to_command_with_history)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn row_to_command_with_history(row: &rusqlite::Row) -> rusqlite::Result<WithHistory<Command>> {
        let command = Self::row_to_command(row)?;
        let history = match row.get::<_, Option<i64>>("h_id")? {
            Some(_) => Some(Self::row_to_execution_history_with_prefix(row)?),
            None => None,
        };
        Ok(WithHistory {
            item: command,
            history,
        })
    }

    fn row_to_execution_history_with_prefix(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<ExecutionHistory> {
        let status_str: String = row.get("h_status")?;
        let status = ExecutionStatus::from_str(&status_str).unwrap_or(ExecutionStatus::Completed);

        let triggered_by_str: String = row.get("h_triggered_by")?;
        let triggered_by = TriggeredBy::from_str(&triggered_by_str).unwrap_or(TriggeredBy::Manual);

        Ok(ExecutionHistory {
            id: row.get("h_id")?,
            command_id: row.get("h_command_id")?, // fix 3: aliased column
            workflow_id: row.get("h_workflow_id")?,
            workflow_step_id: row.get("h_workflow_step_id")?,
            pid: row.get("h_pid")?,
            status,
            exit_code: row.get("h_exit_code")?,
            started_at: row.get("h_started_at")?,
            completed_at: row.get("h_completed_at")?,
            triggered_by,
            context: row.get("h_context")?,
        })
    }

    #[instrument(skip(self))]
    pub fn get_recent_commands(&self, limit: i64) -> Result<Vec<WithHistory<Command>>> {
        let query = format!(
            "{}
            LEFT JOIN (
                SELECT *, ROW_NUMBER() OVER (
                    PARTITION BY command_id
                    ORDER BY started_at DESC, id DESC
                ) as rn
                FROM execution_history
                WHERE command_id IS NOT NULL
            ) h ON c.id = h.command_id AND h.rn = 1
            WHERE EXISTS (
                SELECT 1 FROM execution_history
                WHERE command_id = c.id
            )
            ORDER BY h.started_at DESC NULLS LAST
            LIMIT :limit
        ",
            Self::EXECUTION_HISTORY_SELECT
        );

        self.conn()?
            .prepare(&query)?
            .query_map(
                named_params! { ":limit": limit },
                Self::row_to_command_with_history,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    #[instrument(skip(self))]
    pub fn search_commands(&self, search_term: &str) -> Result<Vec<Command>> {
        debug!(search_term_length = search_term.len(), "Searching commands");

        let pattern = format!("%{}%", search_term);
        self.query_database(
            "SELECT * FROM commands
         WHERE name LIKE ?1 OR command LIKE ?1 OR description LIKE ?1
         ORDER BY is_favorite DESC, position",
            params![pattern],
            Self::row_to_command,
        )
    }

    // TODO: consider using transaction
    #[instrument(skip(self))]
    pub fn update_command(&self, cmd: &Command) -> Result<()> {
        self.validate_command(cmd)?;
        let old_cmd = self.get_command(cmd.id)?;

        if old_cmd.group_id != cmd.group_id {
            self.update_parent_group(COMMANDS_TABLE, COMMAND_GROUP_COLUMN, cmd.id, cmd.group_id)?;
        }

        let arguments = serde_json::to_string(&cmd.arguments)?;
        let env_vars = Self::hashmap_to_string(&cmd.env_vars)?;

        debug!(
            command_id = cmd.id,
            has_env_vars = cmd.env_vars.is_some(),
            "Updating command"
        );

        self.execute_db(
            COMMANDS_TABLE,
            "UPDATE",
            cmd.id,
            "UPDATE commands SET
            name = :name,
            command = :command,
            arguments = :arguments,
            description = :description,
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
        self.move_item_between(
            COMMANDS_TABLE,
            cmd_id,
            prev_id,
            next_id,
            Some(COMMAND_GROUP_COLUMN),
            |id, default| self.get_position_parent_command(id, default),
        )
    }

    #[instrument(skip(self))]
    pub fn delete_command(&self, id: i64) -> Result<()> {
        self.execute_db(
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
        self.execute_db(
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
        self.validate_field_length("name", &cmd.name, Self::MAX_NAME_LENGTH)?;
        self.validate_field_length("command", &cmd.command, Self::MAX_COMMAND_LENGTH)?;
        if let Some(desc) = &cmd.description {
            self.validate_field_length("description", desc, Self::MAX_DESCRIPTION_LENGTH)?;
        }

        self.validate_env_var_keys(&cmd.env_vars)?;
        Ok(())
    }
}
