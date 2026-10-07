//! Emailed sign-in codes: every password sign-in, and every new account, is finished with a
//! 6-digit code sent to the account's email (see `nomi_auth::email_code`). Google sign-in doesn't
//! need one: Google has already checked the person.

use std::sync::Arc;

use nomi_auth::email_code::{Challenge, Purpose, CODE_TTL_SECONDS};
use nomi_i18n::Locale;
use nomi_mail::{Email, Mailer};

#[derive(Clone)]
pub enum EmailCodes {
    /// Password sign-in issues tokens straight away (`AUTH_EMAIL_CODES=off`).
    Off,
    /// Password sign-in, and sign-up, wait for the emailed code.
    Required(Arc<dyn Mailer>),
}

impl EmailCodes {
    /// On unless `AUTH_EMAIL_CODES` is `off`; codes go out through SMTP (`SMTP_*`), or to the
    /// server log when no SMTP server is set.
    pub fn from_env() -> Self {
        let off = std::env::var("AUTH_EMAIL_CODES").is_ok_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "off" | "false" | "0"));
        if off {
            tracing::warn!("AUTH_EMAIL_CODES=off: password sign-in doesn't ask for an emailed code");
            EmailCodes::Off
        } else {
            EmailCodes::Required(nomi_mail::from_env())
        }
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The email that carries `code`, in the person's language.
pub fn code_email(challenge: &Challenge, code: &str, locale: Locale) -> Email {
    let (subject, intro, warning) = match challenge.purpose {
        Purpose::Login => ("sign_in.login_subject", "sign_in.login_intro", "sign_in.login_warning"),
        Purpose::Register => ("sign_in.register_subject", "sign_in.register_intro", "sign_in.register_warning"),
    };
    let subject = locale.tf(subject, &[("code", code)]);
    let intro = locale.t(intro);
    let warning = locale.t(warning);
    let expires = locale.tf("sign_in.expires", &[("minutes", &(CODE_TTL_SECONDS / 60).to_string())]);
    let text = format!("{intro}\n\n    {code}\n\n{expires}\n\n{warning}\n\n— Nomi\n");
    let html = format!(
        "<!doctype html><html><body style=\"margin:0;padding:24px;background:#f4f5f0;font-family:-apple-system,Segoe UI,Roboto,sans-serif;color:#1a1c18\">\
         <div style=\"max-width:440px;margin:0 auto;background:#ffffff;border-radius:24px;padding:32px\">\
         <p style=\"margin:0 0 4px;font-size:22px;font-weight:800;letter-spacing:-0.03em\">nomi</p>\
         <p style=\"margin:20px 0 12px;font-size:16px;line-height:1.5\">{intro}</p>\
         <p style=\"margin:0 0 16px;font-size:36px;font-weight:700;letter-spacing:0.25em;font-family:ui-monospace,Menlo,monospace\">{code}</p>\
         <p style=\"margin:0 0 16px;font-size:14px;color:#44483f\">{expires}</p>\
         <p style=\"margin:0;font-size:13px;line-height:1.5;color:#74796d\">{warning}</p>\
         </div></body></html>",
        intro = escape(&intro),
        code = escape(code),
        expires = escape(&expires),
        warning = escape(&warning),
    );
    Email { to: challenge.email.clone(), subject, text, html: Some(html) }
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
        assert!(email.html.unwrap().contains("042917"));

        let email = code_email(&challenge(Purpose::Register), "111222", Locale::Id);
        assert!(email.subject.starts_with("Konfirmasi"));
        assert!(email.text.contains("111222"));
    }
}
