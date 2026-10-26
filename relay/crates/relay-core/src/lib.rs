//! What every part of Nomi Relay shares: its configuration, the environments and tiers a project
//! can have, the names of a project's database objects, its API keys, and the encryption of
//! what Relay keeps secret.

pub mod config;
pub mod crypto;
pub mod keys;
pub mod names;

pub use config::Config;
pub use names::{Env, Tier};
