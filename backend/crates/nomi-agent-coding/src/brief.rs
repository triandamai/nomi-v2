//! Koda's brief: what he'd otherwise spend his first rounds looking up (read_plan, list_files,
//! read_guide), put in front of him once at the start of a build.

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::plans::{latest_plan_by, open_checklist_items};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};

use crate::guides;
use nomi_agent_core::relay_ops::{RelayEnv, RelayOps, RelayPackages, DEFAULT_SDK_PACKAGE, DEFAULT_SDK_VERSION};

/// More files than this are summarised as a count: list_files shows them all.
const MAX_LISTED_FILES: usize = 150;
const MAX_CHECK_OUTPUT: usize = 3000;

/// The project named by the newest "Project <uuid>:" in what the person (or the agent handing
/// the task over) said, else the newest project in this chat — `None` when there's neither, or
/// the person can't open it.
pub(crate) async fn project_for(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, messages: &[LlmMessage]) -> Option<Uuid> {
    let named = messages
        .iter()
        .filter(|m| m.role == LlmRole::User)
        .flat_map(|m| m.content.iter())
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .flat_map(|text| text.match_indices("Project ").map(move |(at, _)| &text[at + "Project ".len()..]))
        .filter_map(|rest| rest.split_once(':').and_then(|(id, _)| id.trim().parse::<Uuid>().ok()))
        .last();
    let project_id = match named {
        Some(id) => id,
        None => sqlx::query_scalar("SELECT id FROM projects WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1")
            .bind(session_id)
            .fetch_optional(&mut **conn)
            .await
            .ok()
            .flatten()?,
    };
    let can_open: bool = sqlx::query_scalar("SELECT can_open_project($1, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await
        .unwrap_or(false);
    can_open.then_some(project_id)
}

/// "The last publish to production failed at build › build_image: <message>", from Shipyard's
/// failure when it said where, else from the publish's error.
fn failed_publish_line(environment: &str, failure: Option<&serde_json::Value>, error: Option<&str>) -> String {
    let field = |name: &str| failure.and_then(|f| f.get(name)).and_then(serde_json::Value::as_str).filter(|v| !v.is_empty());
    let message = field("message").or(error).unwrap_or("no reason given");
    match (field("phase"), field("step")) {
        (Some(phase), Some(step)) => format!("The last publish to {environment} failed at {phase} › {step}: {message}"),
        _ => format!("The last publish to {environment} failed: {message}"),
    }
}

/// What a Relay project's backend has: its dev tables with their columns, from Relay itself.
async fn relay_section(relay: &dyn RelayOps, user_id: Uuid, project_id: Uuid) -> String {
    let mut section = String::from(
        "## Backend: Nomi Relay\nThis project's database, sign-in and storage come from Relay (guide below): write migrations in \
         nomi/migrations/, use the nomi-relay client, and never add Drizzle or your own auth.\n",
    );
    if relay.backend_starting(project_id).await {
        section.push_str(
            "The dev backend is being set up in the background right now: write your migrations in \
             nomi/migrations/, nomi/backend.json, and nomi/functions/ as usual — Nomi applies them \
             to dev automatically as soon as it's ready. Do not call describe_backend or wait for \
             dev to finish starting before writing the app.\n\n",
        );
        return section;
    }
    match relay.describe_backend(user_id, project_id, RelayEnv::Dev).await {
        Ok(described) => {
            let tables: Vec<String> = serde_json::from_str::<serde_json::Value>(&described)
                .ok()
                .and_then(|schema| schema.get("tables").and_then(serde_json::Value::as_array).cloned())
                .unwrap_or_default()
                .iter()
                .map(|table| {
                    let name = table.get("name").and_then(serde_json::Value::as_str).unwrap_or("?");
                    let columns: Vec<&str> = table.get("columns").and_then(serde_json::Value::as_array).into_iter().flatten().filter_map(|c| c.get("name").and_then(serde_json::Value::as_str)).collect();
                    format!("- {name} ({})", columns.join(", "))
                })
                .collect();
            if tables.is_empty() {
                section.push_str("No tables in dev yet: the first migration is nomi/migrations/0001_init.sql.\n\n");
            } else {
                section.push_str(&format!("Tables in dev now:\n{}\n\n", tables.join("\n")));
            }
        }
        Err(e) => section.push_str(&format!("(Couldn't read the dev backend: {e}. Call describe_backend.)\n\n")),
    }
    section
}

/// The relay guide names the default packages; this Nomi's admin may publish them under other
/// names or ranges, so the brief says the ones to use.
fn with_packages(guide: &str, packages: &RelayPackages) -> String {
    let pinned = format!("\"{}\": \"{}\"", DEFAULT_SDK_PACKAGE, DEFAULT_SDK_VERSION);
    let guide = guide.replace(&pinned, &format!("\"{}\": \"{}\"", packages.sdk, packages.sdk_version));
    let guide = if packages.sdk == DEFAULT_SDK_PACKAGE { guide } else { guide.replace(DEFAULT_SDK_PACKAGE, &packages.sdk) };
    format!(
        "{guide}\n\nPackages on this Nomi: the client is `{}` (`\"{}\": \"{}\"` in dependencies); the command line is `{}` \
         (`\"{}\": \"{}\"` in devDependencies, installs the `nomi-relay` command for `migrate lint`, `migrate new` and `functions new`).",
        packages.sdk, packages.sdk, packages.sdk_version, packages.cli, packages.cli, packages.cli_version
    )
}

fn clip(text: &str, max: usize) -> &str {
    match text.char_indices().nth(max) {
        Some((at, _)) => &text[..at],
        None => text,
    }
}

pub async fn task_brief(
    conn: &mut PoolConnection<Postgres>,
    s3: Option<&nomi_storage::S3Config>,
    session_id: Uuid,
    user_id: Uuid,
    messages: &[LlmMessage],
    relay: &dyn RelayOps,
) -> Option<String> {
    let project_id = project_for(conn, session_id, user_id, messages).await?;
    let (name, stack, files_version, checked_version, check_ok, check_output): (String, String, i64, Option<i64>, Option<bool>, Option<String>) =
        sqlx::query_as("SELECT name, stack, files_version, checked_version, check_ok, check_output FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_optional(&mut **conn)
            .await
            .ok()
            .flatten()?;
    let files: Vec<(String, i32)> = sqlx::query_as("SELECT path, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(&mut **conn)
        .await
        .unwrap_or_default();

    let mut brief = format!(
        "# Your brief\n\nProject {project_id} \"{name}\", stack: {stack}. This is already looked up for you: start from it \
         rather than calling read_plan, list_files or read_guide for what's here.\n\n"
    );

    if files.is_empty() {
        brief.push_str("## Files\nNo files yet: you're starting from scratch.\n\n");
    } else {
        brief.push_str(&format!("## Files ({})\n", files.len()));
        for (path, size) in files.iter().take(MAX_LISTED_FILES) {
            brief.push_str(&format!("{path} ({size} B)\n"));
        }
        if files.len() > MAX_LISTED_FILES {
            brief.push_str(&format!("…and {} more (list_files shows them all)\n", files.len() - MAX_LISTED_FILES));
        }
        brief.push('\n');
    }

    // Names only: the values stay with Shipyard.
    let secrets: Vec<String> = sqlx::query_scalar("SELECT name FROM project_secrets WHERE project_id = $1 ORDER BY name")
        .bind(project_id)
        .fetch_all(&mut **conn)
        .await
        .unwrap_or_default();
    if !secrets.is_empty() {
        brief.push_str(&format!(
            "## Secrets
The person stored these for the published app, read on the server from the environment \
             (process.env, or $env/dynamic/private in SvelteKit): {}. Never put their values in code or ask for them; \
             in the preview they're missing, so handle that plainly.\n\n",
            secrets.join(", ")
        ));
    }

    // A publish that failed, until a later one to that environment worked: it's the first thing
    // to know when the person asks for a fix (Nomi also posted it in the chat).
    let latest_publishes: Vec<(String, String, Option<serde_json::Value>, Option<String>)> = sqlx::query_as(
        "SELECT DISTINCT ON (environment) environment, status, failure, error FROM project_publishes \
         WHERE project_id = $1 AND status <> 'running' ORDER BY environment, created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_default();
    for (environment, status, failure, error) in latest_publishes {
        if status == "failed" {
            brief.push_str(&format!("## {}\n\n", failed_publish_line(&environment, failure.as_ref(), error.as_deref())));
        }
    }

    // Only a check of the files as they are now says anything about them.
    if check_ok == Some(false) && checked_version == Some(files_version) {
        if let Some(output) = check_output.filter(|o| !o.trim().is_empty()) {
            brief.push_str(&format!("## The last check of these files failed\n{}\n\n", clip(output.trim(), MAX_CHECK_OUTPUT)));
        }
    }

    match latest_plan_by(conn, s3, session_id, "planning").await {
        Some(plan) => match plan.content {
            Some(content) => {
                brief.push_str(&format!(
                    "## The plan Rena agreed with the person: \"{}\" (version {})\n{}\n\n",
                    plan.title,
                    plan.version,
                    content.trim()
                ));
                let open = open_checklist_items(&content);
                brief.push_str("## Done when\n");
                for item in &open {
                    brief.push_str(&format!("- {item}\n"));
                }
                if open.is_empty() {
                    brief.push_str("- Everything the plan describes is built.\n");
                }
                brief.push_str("- npm run check and npm run build pass (run them; don't assume).\n\n");
            }
            None => brief.push_str(&format!(
                "## Plan\n\"{}\" (version {}) couldn't be loaded here: call read_plan.\n\n",
                plan.title, plan.version
            )),
        },
        None => brief.push_str(
            "## Plan\nNo plan from Rena in this chat: work from the task and the conversation, and ask with ask_user \
             if something you can't sensibly assume is missing.\n\n",
        ),
    }

    let on_relay = crate::relay::backend_of(conn, project_id).await.as_deref() == Some(crate::relay::BACKEND_RELAY);
    if on_relay {
        brief.push_str(&relay_section(relay, user_id, project_id).await);
    }
    let packages = relay.packages().await;

    let stack_guide = guides::guide_for_stack(&stack).unwrap_or("sveltekit");
    let mut topics = vec![stack_guide, "preview"];
    if on_relay {
        topics.push("relay");
    }
    for topic in topics {
        if let Some((_, _, content)) = guides::GUIDES.iter().find(|(name, _, _)| *name == topic) {
            let content = if topic == "relay" { with_packages(content, &packages) } else { content.to_string() };
            brief.push_str(&format!("## Guide: {topic} (already read for you)\n{content}\n\n"));
        }
    }
    Some(brief.trim_end().to_string())
}

#[cfg(test)]
mod package_tests {
    use super::*;

    fn guide() -> String {
        guides::GUIDES.iter().find(|g| g.0 == "relay").unwrap().2.to_string()
    }

    #[test]
    fn the_default_packages_leave_the_guide_as_it_is() {
        let said = with_packages(&guide(), &RelayPackages::default());
        assert!(said.contains("\"nomi-relay\": \"^0.1.0\"") && said.contains("from 'nomi-relay'"));
        assert!(said.contains("the command line is `nomi-relay-cli`"));
    }

    #[test]
    fn the_admins_packages_replace_the_guides_name_and_version() {
        let packages = RelayPackages { sdk: "@acme/relay".into(), sdk_version: "^2.1.0".into(), cli: "@acme/relay-cli".into(), cli_version: "^2.0.0".into() };
        let said = with_packages(&guide(), &packages);
        assert!(said.contains("\"@acme/relay\": \"^2.1.0\"") && said.contains("from '@acme/relay'"), "{said}");
        assert!(!said.contains("\"nomi-relay\": \"^0.1.0\"") && !said.contains("from 'nomi-relay'"));
        assert!(said.contains("`@acme/relay-cli` (`\"@acme/relay-cli\": \"^2.0.0\"` in devDependencies"));
    }

    struct StartingRelay;

    #[async_trait::async_trait]
    impl RelayOps for StartingRelay {
        async fn is_available(&self) -> bool {
            true
        }
        async fn backend_starting(&self, _: Uuid) -> bool {
            true
        }
        async fn ensure_backend(&self, _: Uuid, id: Uuid) -> Result<Uuid, String> {
            Ok(id)
        }
        async fn apply_migrations(&self, _: Uuid, _: Uuid, _: RelayEnv, _: &[nomi_agent_core::relay_ops::ProjectFile]) -> Result<String, String> {
            unreachable!()
        }
        async fn describe_backend(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
            panic!("relay_section should not call describe_backend while the dev backend is starting")
        }
        async fn types(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
            unreachable!()
        }
        async fn deploy_functions(
            &self,
            _: Uuid,
            _: Uuid,
            _: RelayEnv,
            _: &[nomi_agent_core::relay_ops::ProjectFile],
            _: &serde_json::Value,
        ) -> Result<String, String> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn relay_section_does_not_call_describe_backend_while_dev_is_starting() {
        let section = relay_section(&StartingRelay, Uuid::new_v4(), Uuid::new_v4()).await;
        assert!(section.contains("The dev backend is being set up in the background right now"), "{section}");
        assert!(!section.contains("Couldn't read the dev backend"), "{section}");
    }
}
