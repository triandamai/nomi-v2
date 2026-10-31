//! The shared HTTP client and how its failures become [`NomiError`]s.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use rustls::ClientConfig;
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::NomiError;

/// How long establishing a connection may take.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a whole request may take, body included.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Idle connections kept per host for reuse.
const POOL_IDLE_PER_HOST: usize = 4;
/// How much of an error body ends up in a `NomiError::Http` reason.
const MAX_ERROR_BODY: usize = 200;
const USER_AGENT: &str = concat!("nomi-sdk/", env!("CARGO_PKG_VERSION"));

/// One pooled client for every REST call, addressing paths under the backend's base URL.
#[derive(Clone)]
pub(crate) struct Http {
    client: Client,
    base: Url,
}

impl Http {
    pub fn new(base: Url, tls: Arc<ClientConfig>) -> Result<Self, NomiError> {
        let client = Client::builder()
            .use_preconfigured_tls((*tls).clone())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .pool_max_idle_per_host(POOL_IDLE_PER_HOST)
            .user_agent(USER_AGENT)
            // Never follow a redirect off HTTPS, or anywhere a bearer token could leak.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| NomiError::Internal {
                reason: format!("HTTP client setup: {error}"),
            })?;
        Ok(Self { client, base })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// `path` (starting with `/`) under the base URL, keeping any path prefix the base has.
    pub fn url(&self, path: &str) -> Url {
        let mut url = self.base.clone();
        let joined = format!("{}{path}", self.base.path().trim_end_matches('/'));
        url.set_path(&joined);
        url
    }

    /// The realtime endpoint: the base URL with `ws`/`wss` for `http`/`https`.
    pub fn websocket_url(&self, path: &str) -> Url {
        let mut url = self.url(path);
        let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
        url.set_scheme(scheme)
            .expect("http(s) and ws(s) are interchangeable");
        url
    }
}

pub(crate) fn transport_error(error: reqwest::Error) -> NomiError {
    let reason = if error.is_timeout() {
        "the request timed out".to_owned()
    } else {
        error.to_string()
    };
    NomiError::Network { reason }
}

/// The body as `T` for a 2xx response, otherwise `NomiError::Http` with the start of the body.
pub(crate) async fn json<T: DeserializeOwned>(response: Response) -> Result<T, NomiError> {
    let response = success(response).await?;
    response.json().await.map_err(|error| {
        if error.is_decode() {
            NomiError::InvalidResponse {
                reason: error.to_string(),
            }
        } else {
            transport_error(error)
        }
    })
}

/// The body as `T` whatever the status (for answers that explain a refusal in their body).
pub(crate) async fn json_any<T: DeserializeOwned>(response: Response) -> Result<T, NomiError> {
    response.json().await.map_err(|error| {
        if error.is_decode() {
            NomiError::InvalidResponse {
                reason: error.to_string(),
            }
        } else {
            transport_error(error)
        }
    })
}

/// The raw body of a 2xx response, otherwise `NomiError::Http`.
pub(crate) async fn bytes(response: Response) -> Result<Vec<u8>, NomiError> {
    let response = success(response).await?;
    let body = response.bytes().await.map_err(transport_error)?;
    Ok(body.to_vec())
}

/// The response itself when it is 2xx, otherwise `NomiError::Http`.
pub(crate) async fn success(response: Response) -> Result<Response, NomiError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    Err(status_error(status, response).await)
}

pub(crate) async fn status_error(status: StatusCode, response: Response) -> NomiError {
    let mut reason = response.text().await.unwrap_or_default();
    if reason.len() > MAX_ERROR_BODY {
        let cut = (0..=MAX_ERROR_BODY)
            .rev()
            .find(|&index| reason.is_char_boundary(index))
            .unwrap_or(0);
        reason.truncate(cut);
    }
    NomiError::Http {
        status: status.as_u16(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn http(base: &str) -> Http {
        Http::new(
            Url::parse(base).unwrap(),
            super::super::tls::client_config().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn joins_paths_under_the_base_url() {
        assert_eq!(
            http("https://api.example.com").url("/api/profile").as_str(),
            "https://api.example.com/api/profile"
        );
        assert_eq!(
            http("https://api.example.com/")
                .url("/api/profile")
                .as_str(),
            "https://api.example.com/api/profile"
        );
        assert_eq!(
            http("https://example.com/nomi/").url("/api/x").as_str(),
            "https://example.com/nomi/api/x"
        );
    }

    #[test]
    fn swaps_the_scheme_for_the_socket() {
        assert_eq!(
            http("https://api.example.com")
                .websocket_url("/ws")
                .as_str(),
            "wss://api.example.com/ws"
        );
        assert_eq!(
            http("http://localhost:8080").websocket_url("/ws").as_str(),
            "ws://localhost:8080/ws"
        );
    }
}
