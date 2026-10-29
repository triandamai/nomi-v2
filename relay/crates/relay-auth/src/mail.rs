//! Sending email: through Relay's shared sender or a project's own SMTP server.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use relay_core::settings::Smtp;

/// How long Relay waits on an SMTP server.
const SMTP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, PartialEq)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[async_trait]
pub trait Outbox: Send + Sync {
    async fn send(&self, smtp: &Smtp, email: Email) -> Result<(), String>;
    /// Whether the server answers and takes the credentials, without sending anything.
    async fn check(&self, smtp: &Smtp) -> Result<(), String>;
}

fn transport(smtp: &Smtp) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = match smtp.security.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host).map_err(|e| e.to_string())?,
        "none" => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.host),
        _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host).map_err(|e| e.to_string())?,
    };
    let mut builder = builder.port(smtp.port).timeout(Some(SMTP_TIMEOUT));
    if !smtp.user.is_empty() {
        builder = builder.credentials(Credentials::new(smtp.user.clone(), smtp.password.clone()));
    }
    Ok(builder.build())
}

/// The real thing, over SMTP.
pub struct SmtpOutbox;

#[async_trait]
impl Outbox for SmtpOutbox {
    async fn send(&self, smtp: &Smtp, email: Email) -> Result<(), String> {
        let from: Mailbox = smtp.from.parse().map_err(|_| format!("{} isn't a sender address", smtp.from))?;
        let to: Mailbox = email.to.parse().map_err(|_| format!("{} isn't an address", email.to))?;
        let message = Message::builder().from(from).to(to).subject(email.subject).body(email.text).map_err(|e| e.to_string())?;
        transport(smtp)?.send(message).await.map(|_| ()).map_err(|e| e.to_string())
    }

    async fn check(&self, smtp: &Smtp) -> Result<(), String> {
        smtp.from.parse::<Mailbox>().map_err(|_| format!("{} isn't a sender address", smtp.from))?;
        match transport(smtp)?.test_connection().await {
            Ok(true) => Ok(()),
            Ok(false) => Err("the server didn't answer".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Keeps what it was asked to send instead of sending it (tests, and a Relay with mail off).
#[derive(Default)]
pub struct RecordingOutbox {
    sent: Mutex<Vec<(Smtp, Email)>>,
}

impl RecordingOutbox {
    pub fn sent(&self) -> Vec<(Smtp, Email)> {
        self.sent.lock().expect("outbox lock").clone()
    }
}

#[async_trait]
impl Outbox for RecordingOutbox {
    async fn send(&self, smtp: &Smtp, email: Email) -> Result<(), String> {
        self.sent.lock().expect("outbox lock").push((smtp.clone(), email));
        Ok(())
    }

    async fn check(&self, _smtp: &Smtp) -> Result<(), String> {
        Ok(())
    }
}
