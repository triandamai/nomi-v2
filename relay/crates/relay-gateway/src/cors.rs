//! Which browser origins may call a project's backend. Dev answers Nomi's previews and the
//! builder's own machine; production answers anyone, since row-level security is what protects
//! the data (as with every anon key) and Relay has no per-project origins yet.

use relay_core::Env;

const LOCAL_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];

fn host_of(origin: &str) -> Option<&str> {
    let rest = origin.strip_prefix("https://").or_else(|| origin.strip_prefix("http://"))?;
    rest.split(['/', ':']).next().filter(|h| !h.is_empty())
}

/// `preview_origins` are host suffixes like `.preview.nomi.app`.
pub fn allowed(env: Env, origin: &str, preview_origins: &[String]) -> bool {
    match env {
        Env::Production => true,
        Env::Dev => {
            let Some(host) = host_of(origin) else { return false };
            LOCAL_HOSTS.contains(&host) || preview_origins.iter().any(|suffix| host.ends_with(suffix.as_str()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relay_core::Env;

    const PREVIEWS: &[&str] = &[".preview.nomi.test"];

    fn previews() -> Vec<String> {
        PREVIEWS.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn dev_answers_nomis_previews_and_localhost_only() {
        assert!(allowed(Env::Dev, "https://abc.preview.nomi.test", &previews()));
        assert!(allowed(Env::Dev, "http://localhost:5173", &previews()));
        assert!(allowed(Env::Dev, "http://127.0.0.1:4173", &previews()));
        assert!(!allowed(Env::Dev, "https://evil.test", &previews()));
        assert!(!allowed(Env::Dev, "https://preview.nomi.test.evil.test", &previews()));
        assert!(!allowed(Env::Dev, "https://localhost.evil.test", &previews()));
    }

    #[test]
    fn production_answers_any_origin() {
        assert!(allowed(Env::Production, "https://bakery.example", &previews()));
    }
}
