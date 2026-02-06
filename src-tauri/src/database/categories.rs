use super::{Category, Database, DatabaseError, Result};
use rusqlite::params;

impl Database {
    pub fn create_category(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> Result<i64> {
        self.validate_non_empty("name", &name)?;

        self.conn()?.execute(
            "INSERT INTO categories (name, icon, color) VALUES (?1, ?2, ?3)",
            params![name, icon, color],
        )?;

        Ok(self.conn()?.last_insert_rowid())
    }

    pub fn get_category(&self, id: i64) -> Result<Category> {
        self.conn()?
            .query_row(
                "SELECT * FROM categories WHERE id = ?1",
                params![id],
                |row| Self::row_to_category(row),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound {
                    entity: "category",
                    id,
                },
                _ => e.into(),
            })
    }

    pub fn get_categories(&self) -> Result<Vec<Category>> {
        self.query_database(
            "SELECT * FROM categories ORDER BY name",
            [],
            Self::row_to_category,
        )
    }

    pub fn update_category(
        &self,
        id: i64,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> Result<()> {
        if name.trim().is_empty() {
            return Err(DatabaseError::InvalidData {
                field: "name",
                reason: "Category name cannot be empty".to_string(),
            });
        }

        let rows_affected = self.conn()?.execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3 WHERE id = ?4",
            params![name, icon, color, id],
        )?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "category",
                id,
            });
        }

        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> Result<()> {
        let rows_affected = self
            .conn()?
            .execute("DELETE FROM categories WHERE id = ?1", params![id])?;

        if rows_affected == 0 {
            return Err(DatabaseError::NotFound {
                entity: "category",
                id,
            });
        }

        Ok(())
    }

    pub fn get_category_command_count(&self, id: i64) -> Result<i64> {
        self.conn()?
            .query_row(
                "SELECT COUNT(*) FROM commands WHERE category_id = ?",
                params![id],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    fn row_to_category(row: &rusqlite::Row) -> rusqlite::Result<Category> {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}
