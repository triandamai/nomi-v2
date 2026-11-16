//! Publishing a design preview as a static site, through whichever provider the server has:
//! optional and replaceable. With none, a preview is still saved; it just has no address.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::design_artifact::ArtifactFile;

/// What the default provider says.
pub const NOT_SET_UP: &str = "publishing previews isn't set up";
/// The only kind of address a preview is given out at.
const SECURE_SCHEME: &str = "https://";

/// One preview version to publish.
#[derive(Debug, Clone, Copy)]
pub struct PublishRequest<'a> {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub version: i32,
    pub title: &'a str,
    pub files: &'a [ArtifactFile],
}

/// Where a version was published, and by which provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Published {
    pub provider: String,
    pub url: String,
}

#[async_trait]
pub trait StaticPublisher: Send + Sync {
    /// Publishes the files and returns their address. The error is words for the agent.
    async fn publish(&self, request: &PublishRequest<'_>) -> Result<Published, String>;
}

pub type SharedStaticPublisher = Arc<dyn StaticPublisher>;

/// No provider: nothing is published.
pub struct NoStaticPublisher;

#[async_trait]
impl StaticPublisher for NoStaticPublisher {
    async fn publish(&self, _request: &PublishRequest<'_>) -> Result<Published, String> {
        Err(NOT_SET_UP.to_string())
    }
}

/// Publishes through `publisher`, refusing an address that isn't https: a provider's answer is
/// checked before anyone is sent to it.
pub async fn publish(publisher: &dyn StaticPublisher, request: &PublishRequest<'_>) -> Result<Published, String> {
    let published = publisher.publish(request).await?;
    let host = published.url.strip_prefix(SECURE_SCHEME).unwrap_or_default();
    if host.is_empty() || host.starts_with('/') || published.url.chars().any(char::is_whitespace) {
        return Err(format!("{} gave an address that can't be used", published.provider));
    }
    Ok(published)
}

/// Records where a version of the chat's previews was published. False when there is no such
/// version.
pub async fn record(conn: &mut PgConnection, session_id: Uuid, version: i32, published: &Published) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "INSERT INTO design_artifact_publications (artifact_id, provider, url) \
         SELECT id, $3, $4 FROM design_artifacts WHERE session_id = $1 AND version = $2",
    )
    .bind(session_id)
    .bind(version)
    .bind(&published.provider)
    .bind(&published.url)
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Where a version was published last.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid, version: i32) -> Result<Option<Published>, sqlx::Error> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT p.provider, p.url FROM design_artifact_publications p JOIN design_artifacts a ON a.id = p.artifact_id \
         WHERE a.session_id = $1 AND a.version = $2 ORDER BY p.created_at DESC, p.id DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(version)
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|(provider, url)| Published { provider, url }))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Answers(Result<Published, String>);

    #[async_trait]
    impl StaticPublisher for Answers {
        async fn publish(&self, _request: &PublishRequest<'_>) -> Result<Published, String> {
            self.0.clone()
        }
    }

    fn request() -> PublishRequest<'static> {
        PublishRequest { user_id: Uuid::nil(), session_id: Uuid::nil(), version: 1, title: "Editorial", files: &[] }
    }

    fn at(url: &str) -> Answers {
        Answers(Ok(Published { provider: "fake".into(), url: url.into() }))
    }

    #[tokio::test]
    async fn without_a_provider_nothing_is_published() {
        assert_eq!(publish(&NoStaticPublisher, &request()).await.unwrap_err(), NOT_SET_UP);
    }

    #[tokio::test]
    async fn a_providers_address_is_given_out_only_when_it_is_https() {
        assert_eq!(publish(&at("https://previews.example/a"), &request()).await.unwrap().url, "https://previews.example/a");
        for bad in ["http://previews.example/a", "javascript:alert(1)", "https://", "https:///a", "https://previews.example/a b", ""] {
            assert_eq!(publish(&at(bad), &request()).await.unwrap_err(), "fake gave an address that can't be used", "{bad}");
        }
    }

    #[tokio::test]
    async fn a_providers_failure_is_passed_on_in_its_words() {
        assert_eq!(publish(&Answers(Err("the service is down".into())), &request()).await.unwrap_err(), "the service is down");
    }
}
