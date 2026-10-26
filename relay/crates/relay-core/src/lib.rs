//! What every part of Nomi Relay shares: its configuration, the environments and tiers a project
//! can have, the names of a project's database objects, its API keys, and the encryption of
//! what Relay keeps secret.

pub mod backend;
pub mod config;
pub mod crypto;
pub mod keys;
pub mod names;
pub mod settings;

pub use config::Config;
pub use names::{Env, Tier};
