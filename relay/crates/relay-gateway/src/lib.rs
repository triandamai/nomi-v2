//! Relay's gateway: what apps call. It finds the project by hostname, checks the key or the
//! user's token, and passes the request on to PostgREST as the project's own database role.

pub mod auth;
pub mod cors;
pub mod host;
