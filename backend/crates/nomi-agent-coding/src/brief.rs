//! Koda's brief: what he'd otherwise spend his first rounds looking up (read_plan, list_files,
//! read_guide), put in front of him once at the start of a build.

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::plans::{latest_plan_by, open_checklist_items};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};

use crate::guides;

/// More files than this are summarised as a count: list_files shows them all.
const MAX_LISTED_FILES: usize = 150;
const MAX_CHECK_OUTPUT: usize = 3000;

/// The project named by the newest "Project <uuid>:" in what the person (or the agent handing
/// the task over) said, else the newest project in this chat — `None` when there's neither, or
/// the person can't open it.
async fn project_for(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, messages: &[LlmMessage]) -> Option<Uuid> {
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

    let stack_guide = guides::guide_for_stack(&stack).unwrap_or("sveltekit");
    for topic in [stack_guide, "preview"] {
        if let Some((_, _, content)) = guides::GUIDES.iter().find(|(name, _, _)| *name == topic) {
            brief.push_str(&format!("## Guide: {topic} (already read for you)\n{content}\n\n"));
        }
    }
    Some(brief.trim_end().to_string())
}
