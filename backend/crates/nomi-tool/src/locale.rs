//! The language each person picked (Account → Language), for everything the backend writes to
//! them. See `nomi_i18n` for the texts themselves.

use sqlx::PgConnection;
use uuid::Uuid;

pub use nomi_i18n::Locale;

/// The person's language, or English when they haven't picked one (or it can't be read).
pub async fn user_locale(conn: &mut PgConnection, user_id: Uuid) -> Locale {
    let code: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(conn)
        .await
        .ok()
        .flatten();
    Locale::from_code_or_default(code.as_deref())
}
