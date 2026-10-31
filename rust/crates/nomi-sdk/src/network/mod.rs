//! Talking to the Nomi backend: one pooled HTTP client, the signed-in session, and the realtime
//! socket. Endpoints and wire shapes follow the backend as it is (`backend/crates/nomi-server`).

pub(crate) mod auth;
mod http;
pub(crate) mod realtime;
#[cfg(test)]
mod tests;
mod tls;

use std::sync::Arc;

use tokio::runtime::Handle;
use url::Url;

use crate::error::NomiError;
use crate::event::EventBus;
use auth::{Auth, CredentialStore};
use realtime::{Realtime, RealtimeSettings, REALTIME_PATH};

pub(crate) struct Network {
    pub auth: Arc<Auth>,
    pub realtime: Realtime,
}

impl Network {
    pub fn new(
        runtime: &Handle,
        base_url: Url,
        credentials: Box<dyn CredentialStore>,
        bus: Arc<EventBus>,
        settings: RealtimeSettings,
    ) -> Result<Self, NomiError> {
        let tls = tls::client_config()?;
        let http = http::Http::new(base_url, tls.clone())?;
        let socket_url = http.websocket_url(REALTIME_PATH);
        let auth = Arc::new(Auth::new(http, credentials, bus.clone()));
        let realtime = Realtime::spawn(runtime, socket_url, tls, auth.clone(), bus, settings);
        Ok(Self { auth, realtime })
    }
}
