//! The draft of a preview Sora is still building. A whole preview rarely fits in one reply, so
//! `save_preview` can take its files over several calls: until the last one, they wait here, per
//! chat and direction, where the person never sees them. The last call merges the draft into a
//! version (`merge`) and the draft is cleared.

use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::design_artifact::ArtifactFile;

/// The files of the next version: the last version's, then the draft's, then those of the call
/// that finishes it, each replacing a file of the same path in place (new paths go at the end),
/// and last the paths the call removes.
pub fn merge(previous: Vec<ArtifactFile>, draft: Vec<ArtifactFile>, sent: Vec<ArtifactFile>, remove: &[String]) -> Vec<ArtifactFile> {
    let mut files: Vec<ArtifactFile> = previous;
    for file in draft.into_iter().chain(sent) {
        match files.iter_mut().find(|f| f.path == file.path) {
            Some(existing) => *existing = file,
            None => files.push(file),
        }
    }
    files.retain(|f| !remove.contains(&f.path));
    files
}

/// Adds `files` to the draft of `direction_id` in this chat; a file already there is replaced.
pub async fn stage(conn: &mut PgConnection, session_id: Uuid, direction_id: &str, files: &[ArtifactFile]) -> Result<(), sqlx::Error> {
    // A piece goes into the draft whole or not at all.
    let mut tx = conn.begin().await?;
    for file in files {
        sqlx::query(
            "INSERT INTO design_preview_drafts (session_id, direction_id, path, content) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (session_id, direction_id, path) DO UPDATE SET content = EXCLUDED.content, updated_at = now()",
        )
        .bind(session_id)
        .bind(direction_id)
        .bind(&file.path)
        .bind(&file.content)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

/// The draft of `direction_id` in this chat, in the order its files first arrived.
pub async fn draft(conn: &mut PgConnection, session_id: Uuid, direction_id: &str) -> Result<Vec<ArtifactFile>, sqlx::Error> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT path, content FROM design_preview_drafts WHERE session_id = $1 AND direction_id = $2 ORDER BY created_at, path")
            .bind(session_id)
            .bind(direction_id)
            .fetch_all(conn)
            .await?;
    Ok(rows.into_iter().map(|(path, content)| ArtifactFile { path, content }).collect())
}

/// Drops `paths` from the draft of `direction_id` in this chat once they became a version; a file
/// staged after the draft was read stays for the next one.
pub async fn clear(conn: &mut PgConnection, session_id: Uuid, direction_id: &str, paths: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM design_preview_drafts WHERE session_id = $1 AND direction_id = $2 AND path = ANY($3)")
        .bind(session_id)
        .bind(direction_id)
        .bind(paths)
        .execute(conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_artifact::ArtifactFile;

    fn file(path: &str, content: &str) -> ArtifactFile {
        ArtifactFile { path: path.into(), content: content.into() }
    }

    #[test]
    fn later_layers_replace_earlier_ones_and_remove_goes_last() {
        let previous = vec![file("index.html", "v1"), file("styles.css", "a")];
        let merged = merge(previous, vec![file("about.html", "d")], vec![file("index.html", "v2")], &["styles.css".to_string()]);
        assert_eq!(merged, vec![file("index.html", "v2"), file("about.html", "d")]);
    }

    #[test]
    fn a_path_sent_and_removed_is_removed() {
        let merged = merge(vec![file("index.html", "v1")], vec![], vec![file("about.html", "d")], &["about.html".to_string()]);
        assert_eq!(merged, vec![file("index.html", "v1")]);
    }
}
