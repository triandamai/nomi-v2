//! Nomi Relay's server: the control plane Nomi calls and the gateway apps call, picked by
//! hostname.

pub mod app;
mod auth_routes;
pub mod envelope;
pub mod gateway;
pub mod realtime;
pub mod storage_routes;
