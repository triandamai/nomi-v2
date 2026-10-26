//! A project's migrations: checked (lint), applied in order and recorded, and its schema read
//! back as a description and as TypeScript types.

pub mod lint;
pub mod sql;

use sha2::{Digest, Sha256};

/// Migrations are `0001_init.sql`, `0002_orders.sql`…: a zero-padded number, then a name.
pub fn valid_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".sql") else { return false };
    let Some((number, rest)) = stem.split_once('_') else { return false };
    number.len() >= MIN_NUMBER_DIGITS
        && number.chars().all(|c| c.is_ascii_digit())
        && !rest.is_empty()
        && rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

const MIN_NUMBER_DIGITS: usize = 4;

/// What a migration is recorded with, so an edit after it was applied is noticed.
pub fn checksum(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

/// The 1-based line of a 1-based character `position` in `content` (as Postgres reports errors).
pub fn line_at(content: &str, position: usize) -> usize {
    1 + content.chars().take(position.saturating_sub(1)).filter(|c| *c == '\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_numbers_and_lines() {
        assert!(valid_name("0001_init.sql"));
        assert!(valid_name("0012_add_orders.sql"));
        assert!(!valid_name("init.sql"));
        assert!(!valid_name("001_init.sql"));
        assert!(!valid_name("0001_Init.sql"));
        assert_eq!(checksum("a"), checksum("a"));
        assert_ne!(checksum("a"), checksum("a\n"));
        assert_eq!(line_at("one\ntwo\nthree", 9), 3);
        assert_eq!(line_at("one", 1), 1);
    }
}

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
