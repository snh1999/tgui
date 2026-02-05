use crate::database::{Command, Group};
use std::collections::HashMap;

pub(crate) struct CommandBuilder {
    command: Command,
}

impl CommandBuilder {
    pub(crate) fn new(name: &str, cmd: &str) -> Self {
        Self {
            command: Command {
                id: 0,
                name: name.to_string(),
                command: cmd.to_string(),
                arguments: vec![],
                description: None,
                group_id: None,
                position: 0,
                working_directory: None,
                env_vars: None,
                shell: None,
                category_id: None,
                is_favorite: false,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn with_group(mut self, group_id: i64) -> Self {
        self.command.group_id = Some(group_id);
        self
    }

    pub(crate) fn with_args(mut self, args: Vec<&str>) -> Self {
        self.command.arguments = args.into_iter().map(String::from).collect();
        self
    }

    pub(crate) fn with_env(mut self, key: &str, value: &str) -> Self {
        let env = self.command.env_vars.get_or_insert_with(HashMap::new);
        env.insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn with_category(mut self, category_id: i64) -> Self {
        self.command.category_id = Some(category_id);
        self
    }

    pub(crate) fn build(self) -> Command {
        self.command
    }
}

pub(crate) struct GroupBuilder {
    group: Group,
}

impl GroupBuilder {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            group: Group {
                id: 0,
                name: name.to_string(),
                description: None,
                parent_group_id: None,
                position: 0,
                working_directory: None,
                env_vars: None,
                shell: None,
                category_id: None,
                is_favorite: false,
                icon: None,
                created_at: String::new(),
                updated_at: String::new(),
            },
        }
    }

    pub(crate) fn with_parent(mut self, parent_id: i64) -> Self {
        self.group.parent_group_id = Some(parent_id);
        self
    }

    pub(crate) fn with_category(mut self, category_id: i64) -> Self {
        self.group.category_id = Some(category_id);
        self
    }

    pub(crate) fn with_env(mut self, key: &str, value: &str) -> Self {
        let env = self.group.env_vars.get_or_insert_with(HashMap::new);
        env.insert(key.to_string(), value.to_string());
        self
    }

    pub(crate) fn build(self) -> Group {
        self.group
    }
}
