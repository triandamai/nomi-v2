//! The backend's account-wide pages, called with the session: the person and their crew, money,
//! reminders, memory, connections, usage and projects. Online calls with typed models; nothing here
//! is stored locally (the apps never kept these offline).
//!
//! Shapes follow `backend/crates/nomi-server/src/routes` as they are. Timestamps stay as the
//! backend's RFC 3339 strings and days as `YYYY-MM-DD`, as the screens already read them.

pub(crate) mod account;
pub(crate) mod attachments;
pub(crate) mod home;
pub(crate) mod money;
pub(crate) mod projects;
pub(crate) mod reminders;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::NomiError;
use crate::network::auth::Auth;
use crate::network::http;

/// Authorized JSON calls under `/api`, as the services make them.
#[derive(Clone)]
pub(crate) struct Services {
    auth: Arc<Auth>,
}

impl Services {
    pub fn new(auth: Arc<Auth>) -> Self {
        Self { auth }
    }

    /// `GET /api/<segments>?<query>` as `T`.
    async fn get<T: DeserializeOwned>(
        &self,
        segments: &[&str],
        query: &[(&str, &str)],
    ) -> Result<T, NomiError> {
        let mut url = self.auth.api_url(segments);
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        http::json(response).await
    }

    /// `GET /api/<segments>`'s raw body (files, images).
    async fn get_bytes(&self, segments: &[&str]) -> Result<Vec<u8>, NomiError> {
        let url = self.auth.api_url(segments);
        let response = self.auth.send(|client| client.get(url.clone())).await?;
        http::bytes(response).await
    }

    /// A request with an optional JSON body whose answer is only its status.
    async fn call(
        &self,
        method: Method,
        segments: &[&str],
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<(), NomiError> {
        let response = self.request(method, segments, body).await?;
        http::success(response).await.map(drop)
    }

    /// A request with an optional JSON body, answered with `T`.
    async fn call_for<T: DeserializeOwned>(
        &self,
        method: Method,
        segments: &[&str],
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<T, NomiError> {
        let response = self.request(method, segments, body).await?;
        http::json(response).await
    }

    async fn request(
        &self,
        method: Method,
        segments: &[&str],
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<reqwest::Response, NomiError> {
        let url = self.auth.api_url(segments);
        self.auth
            .send(|client| {
                let request = client.request(method.clone(), url.clone());
                match body {
                    Some(body) => request.json(body),
                    None => request,
                }
            })
            .await
    }
}

/// No body, for [`Services::call`].
const NO_BODY: Option<&()> = None;
