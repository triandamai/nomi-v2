//! Admin → Configuration: saved values win over the environment, secrets are sealed and never
//! shown back, and bad values are refused before anything is saved.
use std::collections::BTreeMap;

use nomi_settings::config::{load, save, view, Source, ATTACHMENTS, SMTP, STORAGE};
use sqlx::PgPool;
use uuid::Uuid;

const KEY: [u8; 32] = [5u8; 32];

async fn admin(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn changes(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_saved_value_wins_and_the_rest_come_from_the_environment(pool: PgPool) {
    let user = admin(&pool).await;
    // A field nobody saved still answers from the environment (S3_REGION is rarely set in tests,
    // so a made-up variable stands in through the env lookup the section is given).
    let env = |name: &str| (name == "S3_REGION").then(|| "eu-west-1".to_string());

    save(&pool, &KEY, &STORAGE, &changes(&[("bucket", "nomi-files"), ("secret_access_key", "s3cr3t")]), user).await.unwrap();
    let storage = load(&pool, Some(&KEY), &STORAGE).await.unwrap().with_env(env);

    assert_eq!(storage.get("bucket").as_deref(), Some("nomi-files"));
    assert_eq!(storage.get("secret_access_key").as_deref(), Some("s3cr3t"));
    assert_eq!(storage.get("region").as_deref(), Some("eu-west-1"));
    assert_eq!(storage.get("endpoint_url"), None);
    let sealed: Vec<u8> = sqlx::query_scalar("SELECT value_encrypted FROM app_config WHERE field = 'secret_access_key'").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&sealed).contains("s3cr3t"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_view_never_shows_a_secret_and_says_where_each_value_comes_from(pool: PgPool) {
    let user = admin(&pool).await;
    save(&pool, &KEY, &SMTP, &changes(&[("host", "smtp.example.com"), ("password", "hunter2")]), user).await.unwrap();

    let fields = view(&pool, Some(&KEY), &SMTP, |name| (name == "SMTP_PORT").then(|| "587".to_string())).await.unwrap();
    let field = |name: &str| fields.iter().find(|f| f.name == name).unwrap();

    assert_eq!(field("host").value.as_deref(), Some("smtp.example.com"));
    assert_eq!(field("host").source, Source::Saved);
    assert_eq!(field("password").value, None, "secrets never come back");
    assert!(field("password").set);
    assert_eq!(field("password").source, Source::Saved);
    assert_eq!(field("port").value.as_deref(), Some("587"));
    assert_eq!(field("port").source, Source::Environment);
    assert_eq!(field("username").source, Source::None);
    assert!(!serde_json::to_string(&fields).unwrap().contains("hunter2"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_empty_secret_keeps_the_saved_one_and_an_empty_value_clears_it(pool: PgPool) {
    let user = admin(&pool).await;
    save(&pool, &KEY, &SMTP, &changes(&[("host", "smtp.example.com"), ("password", "hunter2")]), user).await.unwrap();

    save(&pool, &KEY, &SMTP, &changes(&[("host", ""), ("password", "")]), user).await.unwrap();

    let smtp = load(&pool, Some(&KEY), &SMTP).await.unwrap().with_env(|_| None);
    assert_eq!(smtp.get("host"), None, "an emptied value goes back to the environment");
    assert_eq!(smtp.get("password").as_deref(), Some("hunter2"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_values_and_unknown_fields_are_refused_and_nothing_is_saved(pool: PgPool) {
    let user = admin(&pool).await;
    for bad in [&[("max_file_mb", "ten")][..], &[("max_files", "0")], &[("max_file_mb", "20"), ("nope", "1")]] {
        assert!(save(&pool, &KEY, &ATTACHMENTS, &changes(bad), user).await.is_err(), "{bad:?}");
    }
    assert!(save(&pool, &KEY, &SMTP, &changes(&[("security", "maybe")]), user).await.is_err());
    let saved: i64 = sqlx::query_scalar("SELECT count(*) FROM app_config").fetch_one(&pool).await.unwrap();
    assert_eq!(saved, 0);

    save(&pool, &KEY, &ATTACHMENTS, &changes(&[("max_file_mb", " 25 ")]), user).await.unwrap();
    let limits = load(&pool, Some(&KEY), &ATTACHMENTS).await.unwrap().with_env(|_| None);
    assert_eq!(limits.number("max_file_mb"), Some(25));
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_the_settings_key_secrets_are_refused_and_reads_skip_them(pool: PgPool) {
    let user = admin(&pool).await;
    save(&pool, &KEY, &SMTP, &changes(&[("host", "smtp.example.com"), ("password", "hunter2")]), user).await.unwrap();

    let smtp = load(&pool, None, &SMTP).await.unwrap().with_env(|_| None);

    assert_eq!(smtp.get("host").as_deref(), Some("smtp.example.com"));
    assert_eq!(smtp.get("password"), None);
}
