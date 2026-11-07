//! Emailed sign-in codes: every password sign-in, and every new account, is finished with a
//! 6-digit code sent to the account's email (see `nomi_auth::email_code`). Google sign-in doesn't
//! need one: Google has already checked the person.

use std::sync::Arc;

use nomi_auth::email_code::{Challenge, Purpose, CODE_TTL_SECONDS};
use nomi_auth::password_reset::RESET_TTL_SECONDS;
use nomi_i18n::Locale;
use nomi_mail::layout::{Action, BrandedEmail, Sender};
use nomi_mail::{Email, Mailer};

#[derive(Clone)]
pub enum EmailCodes {
    /// Password sign-in issues tokens straight away (`AUTH_EMAIL_CODES=off`).
    Off,
    /// Password sign-in, and sign-up, wait for the emailed code.
    Required(Arc<dyn Mailer>),
}

impl EmailCodes {
    /// On unless `AUTH_EMAIL_CODES` is `off`; codes go out through the SMTP server in Admin →
    /// Configuration (or `SMTP_*`), or to the server log when there's none.
    pub fn from_env() -> Self {
        let off = std::env::var("AUTH_EMAIL_CODES").is_ok_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "off" | "false" | "0"));
        if off {
            tracing::warn!("AUTH_EMAIL_CODES=off: password sign-in doesn't ask for an emailed code");
            EmailCodes::Off
        } else {
            EmailCodes::Required(nomi_mail::configured())
        }
    }
}

/// The email that carries `code`, in the person's language, signed by Nomi.
pub fn code_email(challenge: &Challenge, code: &str, locale: Locale) -> Email {
    let (subject, title, intro, warning, reason) = match challenge.purpose {
        Purpose::Login => ("sign_in.login_subject", "sign_in.login_title", "sign_in.login_intro", "sign_in.login_warning", "sign_in.login_reason"),
        Purpose::Register => (
            "sign_in.register_subject",
            "sign_in.register_title",
            "sign_in.register_intro",
            "sign_in.register_warning",
            "sign_in.register_reason",
        ),
    };
    let minutes = (CODE_TTL_SECONDS / 60).to_string();
    BrandedEmail {
        sender: Sender::nomi(),
        to: challenge.email.clone(),
        subject: locale.tf(subject, &[("code", code)]),
        preheader: locale.tf("sign_in.preheader", &[("minutes", &minutes)]),
        title: locale.t(title),
        paragraphs: vec![locale.t(intro)],
        highlight: Some(code.to_string()),
        action: None,
        notes: vec![locale.tf("sign_in.expires", &[("minutes", &minutes)]), locale.t(warning)],
        crew_label: locale.t("email.crew_label"),
        footer: format!("{} {}", locale.t(reason), locale.t("email.tagline")),
    }
    .build()
}

/// The email with the link that lets someone choose a new password (`url` opens the reset page).
pub fn reset_email(to: &str, url: String, locale: Locale) -> Email {
    let minutes = (RESET_TTL_SECONDS / 60).to_string();
    BrandedEmail {
        sender: Sender::nomi(),
        to: to.to_string(),
        subject: locale.t("password_reset.subject"),
        preheader: locale.tf("password_reset.preheader", &[("minutes", &minutes)]),
        title: locale.t("password_reset.title"),
        paragraphs: vec![locale.t("password_reset.intro")],
        highlight: None,
        action: Some(Action { label: locale.t("password_reset.button"), url }),
        notes: vec![locale.tf("password_reset.expires", &[("minutes", &minutes)]), locale.t("password_reset.warning")],
        crew_label: locale.t("email.crew_label"),
        footer: format!("{} {}", locale.t("password_reset.reason"), locale.t("email.tagline")),
    }
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn challenge(purpose: Purpose) -> Challenge {
        Challenge {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            purpose,
            email: "ana@example.com".into(),
            expires_at: Utc::now(),
            resend_at: Utc::now(),
            sends_left: 4,
        }
    }

    #[test]
    fn the_email_carries_the_code_in_the_persons_language() {
        let email = code_email(&challenge(Purpose::Login), "042917", Locale::En);
        assert_eq!(email.to, "ana@example.com");
        assert!(email.subject.contains("042917"));
        assert!(email.text.contains("042917") && email.text.contains("10 minutes"));
        let html = email.html.unwrap();
        assert!(html.contains("042917") && html.contains("cid:sender"), "signed with Nomi's shape");
        assert!(!email.inline_images.is_empty());

        let email = code_email(&challenge(Purpose::Register), "111222", Locale::Id);
        assert!(email.subject.starts_with("Konfirmasi"));
        assert!(email.text.contains("111222"));
    }

    #[test]
    fn the_reset_email_links_to_the_reset_page() {
        let email = reset_email("ana@example.com", "https://nomi.example.com/reset-password?token=abc".into(), Locale::En);
        assert_eq!(email.to, "ana@example.com");
        assert!(email.text.contains("https://nomi.example.com/reset-password?token=abc") && email.text.contains("60 minutes"));
        let email = reset_email("ana@example.com", "https://x.io/r?token=abc".into(), Locale::Id);
        assert!(email.subject.starts_with("Atur ulang"));
    }
}
