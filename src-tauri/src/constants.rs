pub const LOG_PREFIX: &'static str = "app.log";

pub const CATEGORIES_TABLE: &'static str = "categories";

pub const COMMANDS_TABLE: &'static str = "commands";
pub const COMMAND_GROUP_COLUMN: &'static str = "group_id";

pub const GROUPS_TABLE: &'static str = "groups";
pub const GROUP_PARENT_GROUP_COLUMN: &'static str = "parent_group_id";

pub const CONNECTION_FAILED_MESSAGE: &'static str =
    "Database connection poisoned by previous panic";
pub const DATABASE_LOCKED_MESSAGE: &'static str =
    "Database is locked by another process. Please try again.";
