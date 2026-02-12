use super::{Category, Database, Result};
use crate::constants::CATEGORIES_TABLE;
use rusqlite::params;
use tracing::instrument;

impl Database {
    #[instrument(skip(self))]
    pub fn create_category(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> Result<i64> {
        self.validate_non_empty("name", &name)?;

        self.create(
            CATEGORIES_TABLE,
            "INSERT INTO categories (name, icon, color) VALUES (?1, ?2, ?3)",
            params![name, icon, color],
        )
    }

    #[instrument(skip(self))]
    pub fn get_category(&self, id: i64) -> Result<Category> {
        self.query_row(
            CATEGORIES_TABLE,
            id,
            "SELECT * FROM categories WHERE id = ?1",
            Self::row_to_category,
        )
    }

    #[instrument(skip(self))]
    pub fn get_categories(&self) -> Result<Vec<Category>> {
        self.query_database(
            "SELECT * FROM categories ORDER BY name",
            [],
            Self::row_to_category,
        )
    }

    #[instrument(skip(self))]
    pub fn update_category(
        &self,
        id: i64,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> Result<()> {
        self.validate_non_empty("name", &name)?;

        self.execute_db(
            CATEGORIES_TABLE,
            "UPDATE",
            id,
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3 WHERE id = ?4",
            params![name, icon, color, id],
        )
    }

    #[instrument(skip(self))]
    pub fn delete_category(&self, id: i64) -> Result<()> {
        self.execute_db(
            CATEGORIES_TABLE,
            "DELETE",
            id,
            "DELETE FROM categories WHERE id = ?1",
            params![id],
        )
    }

    #[instrument(skip(self))]
    pub fn get_category_command_count(&self, id: i64) -> Result<i64> {
        self.query_row(
            CATEGORIES_TABLE,
            id,
            "SELECT COUNT(*) FROM commands WHERE category_id = ?",
            |row| row.get(0),
        )
    }

    fn row_to_category(row: &rusqlite::Row) -> rusqlite::Result<Category> {
        Ok(Category {
            id: row.get("id")?,
            name: row.get("name")?,
            icon: row.get("icon")?,
            color: row.get("color")?,
            created_at: row.get("created_at")?,
        })
    }
}
