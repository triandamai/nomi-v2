//! Which project and environment a request is for, from its hostname:
//! `<slug>.<domain>` is production, `<slug>-dev.<domain>` is dev.

use relay_core::Env;

const DEV_SUFFIX: &str = "-dev";

pub fn route(host: &str, domain: &str) -> Option<(String, Env)> {
    let host = host.split(':').next()?.to_ascii_lowercase();
    let label = host.strip_suffix(&format!(".{}", domain.to_ascii_lowercase()))?;
    if label.is_empty() || label.contains('.') {
        return None;
    }
    match label.strip_suffix(DEV_SUFFIX) {
        Some(slug) if !slug.is_empty() => Some((slug.to_string(), Env::Dev)),
        _ => Some((label.to_string(), Env::Production)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use relay_core::Env;

    #[test]
    fn a_projects_host_names_it_and_its_environment() {
        assert_eq!(route("bakery-6dfd42.api.relay.test", "api.relay.test"), Some(("bakery-6dfd42".to_string(), Env::Production)));
        assert_eq!(route("bakery-6dfd42-dev.api.relay.test", "api.relay.test"), Some(("bakery-6dfd42".to_string(), Env::Dev)));
        assert_eq!(route("BAKERY-6dfd42.api.relay.test:443", "api.relay.test"), Some(("bakery-6dfd42".to_string(), Env::Production)));
        assert_eq!(route("api.relay.test", "api.relay.test"), None);
        assert_eq!(route("a.b.api.relay.test", "api.relay.test"), None);
        assert_eq!(route("bakery-6dfd42.evil.test", "api.relay.test"), None);
    }
}
