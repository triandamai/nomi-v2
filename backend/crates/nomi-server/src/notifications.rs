//! What Nomi tells a person outside a chat: plan and quota changes, promos and account notices.
//! Each lands in their in-app inbox (routes::notifications) and, unless they turned that kind of
//! email off, is also emailed in Nomi's branded layout.

use std::sync::{Arc, OnceLock};

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_i18n::Locale;
use nomi_mail::layout::{BrandedEmail, Sender};
use nomi_mail::Mailer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Account,
    Subscription,
    Quota,
    Promo,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Account => "account",
            Kind::Subscription => "subscription",
            Kind::Quota => "quota",
            Kind::Promo => "promo",
        }
    }

    /// The preference that lets this kind be emailed.
    fn email_preference(self) -> &'static str {
        match self {
            Kind::Promo => "email_promos",
            _ => "email_account",
        }
    }
}

/// One notification, in the person's language already.
#[derive(Debug, Clone)]
pub struct Notice {
    pub kind: Kind,
    pub title: String,
    pub body: String,
    /// Where it leads in the app, e.g. `/billing`.
    pub link: Option<String>,
}

static MAILER: OnceLock<Arc<dyn Mailer>> = OnceLock::new();

/// The mailer notifications go out through, set once at startup (SMTP, or the log).
pub fn init_mailer(mailer: Arc<dyn Mailer>) {
    let _ = MAILER.set(mailer);
}

fn mailer() -> Arc<dyn Mailer> {
    MAILER.get_or_init(|| Arc::new(nomi_mail::LogMailer)).clone()
}

/// The address and language to email `user_id` at, if they want this kind of email.
async fn email_target(pool: &PgPool, user_id: Uuid, kind: Kind) -> Option<(String, Locale)> {
    let query = format!(
        "SELECT COALESCE(w.email, g.email), COALESCE(p.language, 'en'), COALESCE(p.{}, true) \
         FROM users u \
         LEFT JOIN web_credentials w ON w.user_id = u.id \
         LEFT JOIN google_identities g ON g.user_id = u.id \
         LEFT JOIN user_preferences p ON p.user_id = u.id \
         WHERE u.id = $1",
        kind.email_preference()
    );
    let row: Option<(Option<String>, String, bool)> = sqlx::query_as(&query).bind(user_id).fetch_optional(pool).await.ok().flatten();
    match row {
        Some((Some(email), language, true)) => Some((email, Locale::from_code_or_default(Some(&language)))),
        _ => None,
    }
}

fn email_for(to: String, notice: &Notice, locale: Locale) -> nomi_mail::Email {
    let mut notes = Vec::new();
    if let Some(link) = &notice.link {
        notes.push(locale.tf("notify.open_in_app", &[("where", link)]));
    }
    let reason = if notice.kind == Kind::Promo { "notify.reason_promo" } else { "notify.reason_account" };
    BrandedEmail {
        sender: Sender::nomi(),
        to,
        subject: notice.title.clone(),
        preheader: notice.body.chars().take(120).collect(),
        title: notice.title.clone(),
        paragraphs: notice.body.split("\n\n").map(str::to_string).collect(),
        highlight: None,
        notes,
        crew_label: locale.t("email.crew_label"),
        footer: format!("{} {}", locale.t(reason), locale.t("email.tagline")),
    }
    .build()
}

/// Puts `notice` in `user_id`'s inbox, and emails it in the background when they allow it.
pub async fn send(pool: &PgPool, user_id: Uuid, notice: Notice) -> Result<Uuid, sqlx::Error> {
    send_with(pool, user_id, notice, None).await
}

async fn send_with(pool: &PgPool, user_id: Uuid, notice: Notice, broadcast_id: Option<Uuid>) -> Result<Uuid, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO notifications (user_id, kind, title, body, link, broadcast_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(user_id)
    .bind(notice.kind.as_str())
    .bind(&notice.title)
    .bind(&notice.body)
    .bind(&notice.link)
    .bind(broadcast_id)
    .fetch_one(pool)
    .await?;

    let pool = pool.clone();
    tokio::spawn(async move {
        let Some((to, locale)) = email_target(&pool, user_id, notice.kind).await else { return };
        match mailer().send(email_for(to, &notice, locale)).await {
            Ok(()) => {
                let _ = sqlx::query("UPDATE notifications SET emailed_at = now() WHERE id = $1").bind(id).execute(&pool).await;
            }
            Err(e) => tracing::warn!(error = %e, notification_id = %id, "failed to email a notification"),
        }
    });
    Ok(id)
}

/// Who a broadcast goes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audience {
    Everyone,
    /// People currently on this plan.
    Plan(Uuid),
}

impl Audience {
    pub fn as_str(&self) -> String {
        match self {
            Audience::Everyone => "all".to_string(),
            Audience::Plan(id) => id.to_string(),
        }
    }
}

/// Sends the same notice to everyone in `audience`; returns the broadcast's id and how many it
/// reached. Emails go out in the background, to those who allow them.
pub async fn broadcast(pool: &PgPool, notice: Notice, audience: Audience, sent_by: Uuid) -> Result<(Uuid, i64), sqlx::Error> {
    let recipients: Vec<Uuid> = match &audience {
        Audience::Everyone => sqlx::query_scalar("SELECT id FROM users").fetch_all(pool).await?,
        Audience::Plan(plan_id) => {
            sqlx::query_scalar("SELECT user_id FROM user_subscriptions WHERE plan_id = $1").bind(plan_id).fetch_all(pool).await?
        }
    };
    let broadcast_id: Uuid = sqlx::query_scalar(
        "INSERT INTO notification_broadcasts (title, body, link, audience, emailed, recipients, sent_by) \
         VALUES ($1, $2, $3, $4, true, $5, $6) RETURNING id",
    )
    .bind(&notice.title)
    .bind(&notice.body)
    .bind(&notice.link)
    .bind(audience.as_str())
    .bind(recipients.len() as i32)
    .bind(sent_by)
    .fetch_one(pool)
    .await?;
    for user_id in &recipients {
        send_with(pool, *user_id, notice.clone(), Some(broadcast_id)).await?;
    }
    Ok((broadcast_id, recipients.len() as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notice_email_carries_its_link_and_why_it_was_sent() {
        let notice = Notice {
            kind: Kind::Subscription,
            title: "You're on Pro now".into(),
            body: "Your plan changed to Pro.\n\nYou can use 10,000,000 tokens a month.".into(),
            link: Some("/billing".into()),
        };
        let email = email_for("ana@example.com".into(), &notice, Locale::En);
        let html = email.html.unwrap();
        assert_eq!(email.subject, "You're on Pro now");
        assert!(html.contains("10,000,000 tokens"));
        assert!(html.contains("/billing"));
        assert!(email.text.contains("Your plan changed to Pro."));
    }
}
