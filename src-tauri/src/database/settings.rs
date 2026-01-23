use super::{Database, DatabaseError, Result};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::LazyLock;

static DEFAULT_SETTINGS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("theme", "system"),
        ("default_shell", "/bin/bash"),
        ("log_buffer_size", "10000"),
        ("max_concurrent_processes", "20"),
        ("auto_scroll_logs", "true"),
        ("warn_before_kill", "true"),
        ("kill_process_tree_by_default", "false"),
    ])
});

impl Database {
    pub fn initialize_settings(&self) -> Result<()> {
        for (key, value) in DEFAULT_SETTINGS.iter() {
            self.conn()?.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }
        Ok(())
    }

    /// no id provided/exists, so id in error is set to a dummy value 0
    pub fn get_setting(&self, key: &str) -> Result<String> {
        self.conn()?
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound {
                    entity: "setting",
                    id: 0,
                },
                _ => e.into(),
            })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.validate_setting(key, value)?;

        self.conn()?.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;

        Ok(())
    }

    pub fn reset_settings(&self) -> Result<()> {
        for (key, value) in DEFAULT_SETTINGS.iter() {
            self.set_setting(key, value)?;
        }
        Ok(())
    }

    /// Get all settings as HashMap
    pub fn get_all_settings(&self) -> Result<HashMap<String, String>> {
        let connection = self.conn()?;
        let mut stmt = connection.prepare("SELECT key, value FROM settings")?;
        let settings = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<HashMap<_, _>>>()?;

        Ok(settings)
    }

    fn validate_setting(&self, key: &str, value: &str) -> Result<()> {
        if !DEFAULT_SETTINGS.contains_key(&key) {
            return Err(DatabaseError::InvalidData {
                field: "key",
                reason: format!("Unknown setting: {}", key),
            });
        }

        match key {
            "log_buffer_size" | "max_concurrent_processes" => value
                .parse::<i32>()
                .map(|_| ())
                .map_err(|_| DatabaseError::InvalidData {
                    field: "value",
                    reason: "Must be a number".to_string(),
                }),

            "auto_scroll_logs" | "warn_before_kill" | "kill_process_tree_by_default" => {
                if value == "true" || value == "false" {
                    Ok(())
                } else {
                    Err(DatabaseError::InvalidData {
                        field: "value",
                        reason: "Must be 'true' or 'false'".to_string(),
                    })
                }
            }

            _ => Ok(()),
        }
    }
}
