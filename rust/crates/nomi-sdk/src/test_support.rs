//! Helpers shared by the crate's tests.

use std::sync::{Arc, Mutex};

use base64::Engine as _;

use crate::network::auth::{CredentialStore, Credentials};

/// A [`CredentialStore`] in memory; clones share the same slot so tests can look inside.
#[derive(Clone, Default)]
pub(crate) struct MemoryCredentials(Arc<Mutex<Option<Credentials>>>);

impl MemoryCredentials {
    pub fn holding(credentials: Credentials) -> Self {
        Self(Arc::new(Mutex::new(Some(credentials))))
    }

    pub fn get(&self) -> Option<Credentials> {
        self.0.lock().unwrap().clone()
    }

    pub fn boxed(&self) -> Box<dyn CredentialStore> {
        Box::new(self.clone())
    }
}

impl CredentialStore for MemoryCredentials {
    fn load(&self) -> Option<Credentials> {
        self.get()
    }

    fn save(&self, credentials: Credentials) {
        *self.0.lock().unwrap() = Some(credentials);
    }

    fn clear(&self) {
        *self.0.lock().unwrap() = None;
    }
}

/// An unsigned JWT-shaped token for `user_id`, tagged so tests can tell tokens apart.
pub(crate) fn token(user_id: &str, tag: &str) -> String {
    let encode = |json: &str| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json);
    format!(
        "{}.{}.{tag}",
        encode(r#"{"alg":"none"}"#),
        encode(&format!(r#"{{"sub":"{user_id}","tag":"{tag}"}}"#))
    )
}

pub(crate) fn credentials(user_id: &str, tag: &str) -> Credentials {
    Credentials {
        access_token: token(user_id, tag),
        refresh_token: format!("refresh-{tag}"),
    }
}
