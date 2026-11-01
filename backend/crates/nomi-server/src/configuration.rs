//! What the server and the worker take from Admin → Configuration at start. Most settings are
//! read on use (attachments, Google, email, the agent loop); file storage is set up once, so a
//! change to it applies the next time Nomi starts.

use nomi_settings::config;

/// S3 as saved in Admin → Configuration, field by field over the environment's `S3_*` and
/// `AWS_*`; `None` keeps files on disk.
pub async fn storage() -> Option<nomi_storage::S3Config> {
    let storage = config::current(&config::STORAGE).await;
    nomi_storage::build_from(|field| storage.get(field)).await
}

/// The domain every published app is addressed under (Admin → Configuration → Domains), with no
/// environment fallback; `None` leaves the address to Shipyard's own apps domain.
pub async fn apps_base_domain() -> Option<String> {
    config::current(&config::DOMAINS).await.get("apps_base_domain")
}
