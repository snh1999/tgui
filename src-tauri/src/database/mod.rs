use rusqlite::Connection;
use std::cell::{Ref, RefCell, RefMut};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub mod categories;
pub mod commands;
pub mod groups;
pub(crate) mod helpers;
mod settings;

pub mod builders;

pub mod models;
pub use models::*;

pub mod errors;
pub use errors::{DatabaseError, Result};

#[cfg(test)]
mod tests;

pub struct Database {
    conn: RefCell<Connection>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.pragma_update(None, "foreign_keys", &"ON")?;
        conn.pragma_update(None, "journal_mode", &"WAL")?;

        #[cfg(unix)]
        Self::set_file_permissions(path)?;

        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;

        let db = Self {
            conn: RefCell::new(conn),
        };
        db.initialize_settings()?;
        Ok(db)
    }

    pub(crate) fn conn(&self) -> Ref<'_, Connection> {
        self.conn.borrow()
    }

    pub(crate) fn conn_mut(&self) -> RefMut<'_, Connection> {
        self.conn.borrow_mut()
    }

    #[cfg(unix)]
    fn set_file_permissions(path: &Path) -> Result<()> {
        let metadata =
            fs::metadata(path).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600); // rw-------
        fs::set_permissions(path, permissions)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        Ok(())
    }

    pub fn get_schema_version(&self) -> Result<i32> {
        let version: i32 =
            self.conn()
                .query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;
        Ok(version)
    }
}
