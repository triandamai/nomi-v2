//! Sending email: SMTP (any provider: Brevo, Resend, Amazon SES, Mailgun, Gmail, your own server)
//! configured from the environment. With no SMTP server set, mail is written to the server log
//! instead, so a development install still works.

pub mod brand;
pub mod layout;

use std::sync::{Arc, Mutex};

use lettre::message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// One email, with a plain-text body and an optional HTML one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
    /// Images the HTML shows as `cid:<content_id>` (the agents' shapes), sent inside the email.
    pub inline_images: Vec<InlineImage>,
}

/// An image carried inside the email (so it shows without loading anything from the web).
#[derive(Debug, Clone, PartialEq)]
pub struct InlineImage {
    pub content_id: String,
    pub png: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("invalid address: {0}")]
    Address(String),
    #[error("couldn't build the email: {0}")]
    Build(String),
    #[error("couldn't send the email: {0}")]
    Send(String),
}

#[async_trait::async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, email: Email) -> Result<(), MailError>;

    /// Whether mail actually leaves the server (false when it's only logged).
    fn delivers(&self) -> bool {
        true
    }
}

/// How the connection to the SMTP server is secured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpSecurity {
    /// Plain connection upgraded with STARTTLS (usually port 587).
    StartTls,
    /// TLS from the start (usually port 465).
    Tls,
    /// No encryption: local relays and test servers only.
    None,
}

impl SmtpSecurity {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "starttls" => Some(Self::StartTls),
            "tls" | "ssl" => Some(Self::Tls),
            "none" | "plain" | "off" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub security: SmtpSecurity,
    /// The sender, e.g. `Nomi <no-reply@example.com>`.
    pub from: String,
}

impl SmtpConfig {
    /// Reads `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_SECURITY` and
    /// `MAIL_FROM`. `None` when `SMTP_HOST` isn't set.
    pub fn from_env() -> Option<Self> {
        let var = |name: &str| std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let host = var("SMTP_HOST")?;
        let port: Option<u16> = var("SMTP_PORT").and_then(|p| p.parse().ok());
        let security = var("SMTP_SECURITY").and_then(|s| SmtpSecurity::parse(&s)).unwrap_or(match port {
            Some(465) => SmtpSecurity::Tls,
            _ => SmtpSecurity::StartTls,
        });
        let port = port.unwrap_or(match security {
            SmtpSecurity::Tls => 465,
            SmtpSecurity::StartTls => 587,
            SmtpSecurity::None => 25,
        });
        let username = var("SMTP_USERNAME");
        let from = var("MAIL_FROM").or_else(|| username.clone().filter(|u| u.contains('@'))).unwrap_or_else(|| format!("Nomi <no-reply@{host}>"));
        Some(Self { host, port, username, password: var("SMTP_PASSWORD"), security, from })
    }
}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpMailer {
    pub fn new(config: &SmtpConfig) -> Result<Self, MailError> {
        let builder = match config.security {
            SmtpSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host),
            SmtpSecurity::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host),
            SmtpSecurity::None => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)),
        }
        .map_err(|e| MailError::Build(e.to_string()))?;
        let mut builder = builder.port(config.port).timeout(Some(std::time::Duration::from_secs(20)));
        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }
        let from = config.from.parse::<Mailbox>().map_err(|_| MailError::Address(config.from.clone()))?;
        Ok(Self { transport: builder.build(), from })
    }
}

/// The message as it goes out: plain text, plus HTML when there is some.
pub fn build_message(from: Mailbox, email: &Email) -> Result<Message, MailError> {
    let to = email.to.parse::<Mailbox>().map_err(|_| MailError::Address(email.to.clone()))?;
    let builder = Message::builder().from(from).to(to).subject(&email.subject);
    let text = SinglePart::builder().header(ContentType::TEXT_PLAIN).body(email.text.clone());
    let message = match &email.html {
        Some(html) => {
            let html = SinglePart::builder().header(ContentType::TEXT_HTML).body(html.clone());
            let alternative = MultiPart::alternative().singlepart(text);
            if email.inline_images.is_empty() {
                builder.multipart(alternative.singlepart(html))
            } else {
                // HTML plus the images it shows, together (multipart/related).
                let png = ContentType::parse("image/png").expect("a valid content type");
                let related = email.inline_images.iter().fold(MultiPart::related().singlepart(html), |related, image| {
                    related.singlepart(Attachment::new_inline(image.content_id.clone()).body(image.png.clone(), png.clone()))
                });
                builder.multipart(alternative.multipart(related))
            }
        }
        None => builder.singlepart(text),
    };
    message.map_err(|e| MailError::Build(e.to_string()))
}

#[async_trait::async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, email: Email) -> Result<(), MailError> {
        let message = build_message(self.from.clone(), &email)?;
        self.transport.send(message).await.map_err(|e| MailError::Send(e.to_string()))?;
        Ok(())
    }
}

/// Writes mail to the server log instead of sending it (no SMTP server configured).
pub struct LogMailer;

#[async_trait::async_trait]
impl Mailer for LogMailer {
    async fn send(&self, email: Email) -> Result<(), MailError> {
        tracing::warn!(to = %email.to, subject = %email.subject, body = %email.text, "no SMTP server configured (SMTP_HOST); email written to the log instead of sent");
        Ok(())
    }

    fn delivers(&self) -> bool {
        false
    }
}

/// Keeps every email in memory, for tests.
#[derive(Default)]
pub struct MemoryMailer {
    sent: Mutex<Vec<Email>>,
}

impl MemoryMailer {
    pub fn sent(&self) -> Vec<Email> {
        self.sent.lock().unwrap().clone()
    }

    pub fn last_to(&self, to: &str) -> Option<Email> {
        self.sent.lock().unwrap().iter().rev().find(|e| e.to == to).cloned()
    }
}

#[async_trait::async_trait]
impl Mailer for MemoryMailer {
    async fn send(&self, email: Email) -> Result<(), MailError> {
        self.sent.lock().unwrap().push(email);
        Ok(())
    }
}

/// The mailer the environment asks for: SMTP when `SMTP_HOST` is set, else the log.
pub fn from_env() -> Arc<dyn Mailer> {
    match SmtpConfig::from_env() {
        Some(config) => match SmtpMailer::new(&config) {
            Ok(mailer) => {
                tracing::info!(host = %config.host, port = config.port, "email: sending through SMTP");
                Arc::new(mailer)
            }
            Err(e) => panic!("SMTP settings are invalid: {e}"),
        },
        None => {
            tracing::warn!("email: SMTP_HOST isn't set, so emails (including sign-in codes) are written to the server log, not sent");
            Arc::new(LogMailer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_follows_the_port_unless_set() {
        assert_eq!(SmtpSecurity::parse("STARTTLS"), Some(SmtpSecurity::StartTls));
        assert_eq!(SmtpSecurity::parse("ssl"), Some(SmtpSecurity::Tls));
        assert_eq!(SmtpSecurity::parse("bogus"), None);
    }

    #[test]
    fn a_message_carries_text_and_html() {
        let email = Email { to: "ana@example.com".into(), subject: "Your code".into(), text: "123456".into(), html: Some("<b>123456</b>".into()), ..Default::default() };
        let message = build_message("Nomi <no-reply@example.com>".parse().unwrap(), &email).unwrap();
        let raw = String::from_utf8(message.formatted()).unwrap();
        assert!(raw.contains("Subject: Your code"));
        assert!(raw.contains("multipart/alternative"));
        assert!(raw.contains("123456"));
    }

    #[test]
    fn inline_images_travel_with_the_html() {
        let email = Email {
            to: "ana@example.com".into(),
            subject: "Hi".into(),
            text: "hi".into(),
            html: Some(r#"<img src="cid:nomi">"#.into()),
            inline_images: vec![InlineImage { content_id: "nomi".into(), png: brand::shape_png("cookie9", "glow", true, 48) }],
        };
        let raw = String::from_utf8(build_message("a@b.co".parse().unwrap(), &email).unwrap().formatted()).unwrap();
        assert!(raw.contains("multipart/alternative") && raw.contains("multipart/related"));
        assert!(raw.contains("Content-ID: <nomi>"));
        assert!(raw.contains("Content-Disposition: inline"));
        assert!(raw.contains("image/png"));
    }

    #[test]
    fn a_bad_address_is_refused() {
        let email = Email { to: "not an address".into(), subject: "x".into(), text: "x".into(), ..Default::default() };
        assert!(matches!(build_message("a@b.co".parse().unwrap(), &email), Err(MailError::Address(_))));
    }
}
