//! Share links to a reply (`routes/shares.rs`), as the web's share dialog makes them by default:
//! amounts and contact details masked, open to anyone with the link, for a week.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::Services;
use crate::error::NomiError;

const SCOPE_REPLY: &str = "reply";
/// The web's default expiry (1, 7 or 30 days are allowed).
const SHARE_DAYS: i64 = 7;

#[derive(Serialize)]
struct ShareRequest<'a> {
    scope: &'a str,
    message_id: &'a str,
    mask: bool,
    anyone: bool,
    expires_in_days: i64,
}

/// A share link: the page lives at `<web app>/shared/<token>`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ShareLink {
    pub id: String,
    pub token: String,
}

impl Services {
    /// A new link to one reply.
    pub async fn share_reply(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<ShareLink, NomiError> {
        let body = ShareRequest {
            scope: SCOPE_REPLY,
            message_id,
            mask: true,
            anyone: true,
            expires_in_days: SHARE_DAYS,
        };
        self.call_for(
            Method::POST,
            &["sessions", session_id, "shares"],
            Some(&body),
        )
        .await
    }
}
