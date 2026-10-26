//! A project's migrations: checked (lint), applied in order and recorded, and its schema read
//! back as a description and as TypeScript types.

pub mod lint;
pub mod sql;

/// One file of a project's `nomi/migrations/`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MigrationFile {
    pub path: String,
    pub content: String,
}

impl MigrationFile {
    /// `0001_init.sql` for `nomi/migrations/0001_init.sql`.
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}
