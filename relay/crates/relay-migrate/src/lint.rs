//! The checks a migration passes before Relay applies it: the same rules as the CLI
//! (`relay/cli/src/lint.ts`), so what `nomi-relay migrate lint` accepts, Relay accepts.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

use crate::sql::{split_statements, unquote, Statement};
use crate::MigrationFile;

pub const RULE_RLS: &str = "row-level-security";
pub const RULE_DESTRUCTIVE: &str = "destructive";
pub const RULE_OUTSIDE: &str = "outside-project";

/// Marks a table meant to be readable without row-level security.
pub const PUBLIC_MARKER: &str = "nomi: public";
/// Marks a drop meant to lose data.
pub const DESTRUCTIVE_MARKER: &str = "nomi: destructive";
/// The only schema a migration may name; unqualified names land in the project's own schema.
const OWN_SCHEMA: &str = "public";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Issue {
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

const IDENT: &str = r#"(?:"[^"]+"|[a-z_][a-z0-9_$]*)"#;

struct Patterns {
    create_table: Regex,
    enable_rls: Regex,
    qualified_ddl: Regex,
    qualified_on: Regex,
    forbidden: Vec<(Regex, &'static str)>,
}

fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let qualified = format!(r"(?:({IDENT})\.)?({IDENT})");
        Patterns {
            create_table: Regex::new(&format!(r"^create (?:unlogged )?table (?:if not exists )?{qualified}")).unwrap(),
            enable_rls: Regex::new(&format!(r"^alter table (?:if exists )?(?:only )?{qualified} enable row level security")).unwrap(),
            qualified_ddl: Regex::new(&format!(
                r"^(?:create|alter|drop) (?:or replace )?(?:unique )?(?:unlogged )?(?:materialized )?(?:table|view|function|procedure|index|type|trigger|sequence|domain|aggregate) (?:if (?:not )?exists )?(?:concurrently )?(?:only )?({IDENT})\."
            ))
            .unwrap(),
            qualified_on: Regex::new(&format!(r"^create (?:policy|trigger|(?:unique )?index) .*? on (?:only )?({IDENT})\.{IDENT}")).unwrap(),
            forbidden: vec![
                (Regex::new(r"^(?:create|alter|drop) schema\b").unwrap(), "creating, changing or dropping a schema"),
                (Regex::new(r"^(?:create|alter|drop) (?:role|user|group)\b").unwrap(), "managing database roles"),
                (Regex::new(r"^(?:create|alter|drop) database\b").unwrap(), "managing databases"),
                (Regex::new(r"^(?:set|reset) (?:local )?(?:role|session authorization|search_path)\b").unwrap(), "switching role or search_path"),
                (Regex::new(r"^alter system\b").unwrap(), "changing server settings"),
                (Regex::new(r"^(?:create|alter|drop) event trigger\b").unwrap(), "event triggers"),
                (Regex::new(r"^copy .* (?:from|to) program\b").unwrap(), "running programs on the server"),
            ],
        }
    })
}

/// Words after `drop` that change a column without losing data.
const SAFE_DROPS: [&str; 5] = ["constraint", "default", "not null", "identity", "expression"];

fn drops_data(code: &str) -> bool {
    if code.starts_with("drop table ") {
        return true;
    }
    if !code.starts_with("alter table ") {
        return false;
    }
    code.match_indices(" drop ").any(|(at, word)| {
        let rest = &code[at + word.len()..];
        let rest = rest.strip_prefix("column ").unwrap_or(rest);
        !SAFE_DROPS.iter().any(|safe| rest == *safe || rest.starts_with(&format!("{safe} ")) || rest.starts_with(&format!("{safe},")))
    })
}

fn has_marker(statement: &Statement, marker: &str) -> bool {
    statement.comments.to_lowercase().contains(marker)
}

/// Checks `files` as a set (row-level security can be enabled in a later migration).
pub fn lint(files: &[MigrationFile]) -> Vec<Issue> {
    let p = patterns();
    let parsed: Vec<(&MigrationFile, Vec<Statement>)> = files.iter().map(|f| (f, split_statements(&f.content))).collect();
    let rls_enabled: HashSet<String> = parsed
        .iter()
        .flat_map(|(_, statements)| statements.iter())
        .filter_map(|s| p.enable_rls.captures(&s.code).map(|c| unquote(&c[2])))
        .collect();

    let mut issues = Vec::new();
    for (file, statements) in &parsed {
        for statement in statements {
            let issue = |rule: &'static str, message: String| Issue { file: file.name().to_string(), line: statement.line, rule, message };
            if let Some(table) = p.create_table.captures(&statement.code) {
                let name = unquote(&table[2]);
                if !rls_enabled.contains(&name) && !has_marker(statement, PUBLIC_MARKER) {
                    issues.push(issue(
                        RULE_RLS,
                        format!("table \"{name}\" has no row-level security: add \"alter table {name} enable row level security;\" and policies, or mark it \"-- {PUBLIC_MARKER}\""),
                    ));
                }
            }
            if drops_data(&statement.code) && !has_marker(statement, DESTRUCTIVE_MARKER) {
                issues.push(issue(RULE_DESTRUCTIVE, format!("this drops data: mark it \"-- {DESTRUCTIVE_MARKER}\" if that's intended")));
            }
            let schema = p.qualified_ddl.captures(&statement.code).or_else(|| p.qualified_on.captures(&statement.code)).map(|c| unquote(&c[1]));
            if let Some(schema) = schema.filter(|s| s != OWN_SCHEMA) {
                issues.push(issue(RULE_OUTSIDE, format!("migrations may only define things in the project's own schema, not \"{schema}\"")));
            }
            for (pattern, what) in &p.forbidden {
                if pattern.is_match(&statement.code) {
                    issues.push(issue(RULE_OUTSIDE, format!("migrations can't do {what}")));
                }
            }
        }
    }
    issues
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::MigrationFile;

    fn lint_sql(sql: &str, more: &[&str]) -> Vec<Issue> {
        let mut files = vec![MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: sql.into() }];
        for (i, extra) in more.iter().enumerate() {
            files.push(MigrationFile { path: format!("nomi/migrations/000{}_x.sql", i + 2), content: (*extra).into() });
        }
        lint(&files)
    }

    fn rules(sql: &str) -> Vec<&'static str> {
        lint_sql(sql, &[]).into_iter().map(|i| i.rule).collect()
    }

    #[test]
    fn passes_the_example_from_the_design_doc() {
        let sql = "create table task (\n  id bigint generated always as identity primary key,\n  owner uuid not null default nomi.uid() references nomi.users(id) on delete cascade,\n  title text not null\n);\nalter table task enable row level security;\ncreate policy \"own tasks\" on task using (owner = nomi.uid()) with check (owner = nomi.uid());";
        assert!(lint_sql(sql, &[]).is_empty());
    }

    #[test]
    fn flags_a_table_without_row_level_security() {
        let issues = lint_sql("create table note (id int);", &[]);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, RULE_RLS);
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].file, "0001_init.sql");
    }

    #[test]
    fn accepts_row_level_security_enabled_in_a_later_migration_or_a_public_marker() {
        assert!(lint_sql("create table note (id int);", &["alter table only \"note\" enable row level security;"]).is_empty());
        assert!(lint_sql("-- nomi: public\ncreate table country (code text);", &[]).is_empty());
    }

    #[test]
    fn requires_a_marker_on_drops_that_lose_data() {
        assert_eq!(rules("drop table note;"), vec![RULE_DESTRUCTIVE]);
        assert_eq!(rules("alter table task drop column title;"), vec![RULE_DESTRUCTIVE]);
        assert!(rules("-- nomi: destructive\ndrop table note;").is_empty());
        assert!(rules("alter table task drop constraint task_title_key;").is_empty());
        assert!(rules("alter table task alter column title drop not null;").is_empty());
    }

    #[test]
    fn rejects_statements_that_reach_outside_the_project() {
        assert!(rules("create table nomi.evil (id int);").contains(&RULE_OUTSIDE));
        assert_eq!(rules("create policy p on nomi.users using (true);"), vec![RULE_OUTSIDE]);
        assert_eq!(rules("create schema other;"), vec![RULE_OUTSIDE]);
        assert_eq!(rules("set role postgres;"), vec![RULE_OUTSIDE]);
        assert!(rules("create function public.f() returns int as $$ select 1 $$ language sql;").is_empty());
    }
}
