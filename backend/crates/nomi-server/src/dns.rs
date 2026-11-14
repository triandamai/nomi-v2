//! DNS looks for a project's own domains: what a hostname's A, CNAME and TXT records say right
//! now. The system's resolver (hickory, from `/etc/resolv.conf`) answers by default; tests
//! install their own answers with `install`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::TokioResolver;

/// How long one look may take before it counts as finding nothing.
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordKind {
    A,
    Cname,
    Txt,
}

impl RecordKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::Cname => "CNAME",
            Self::Txt => "TXT",
        }
    }

    fn record_type(self) -> RecordType {
        match self {
            Self::A => RecordType::A,
            Self::Cname => RecordType::CNAME,
            Self::Txt => RecordType::TXT,
        }
    }
}

/// Answers DNS questions: the values of `name`'s records of `kind` (names without the trailing
/// dot), empty when there are none or the look failed.
#[async_trait::async_trait]
pub trait DnsLookup: Send + Sync {
    async fn lookup(&self, name: &str, kind: RecordKind) -> Vec<String>;
}

/// The system's resolver.
struct SystemDns {
    resolver: Option<TokioResolver>,
}

#[async_trait::async_trait]
impl DnsLookup for SystemDns {
    async fn lookup(&self, name: &str, kind: RecordKind) -> Vec<String> {
        let Some(resolver) = &self.resolver else {
            return Vec::new();
        };
        let wanted = kind.record_type();
        match tokio::time::timeout(LOOKUP_TIMEOUT, resolver.lookup(name, wanted)).await {
            Ok(Ok(found)) => found
                .answers()
                .iter()
                .filter(|r| r.record_type() == wanted)
                .map(|r| r.data.to_string().trim_end_matches('.').to_string())
                .collect(),
            Ok(Err(e)) => {
                tracing::debug!(error = %e, name, kind = kind.as_str(), "dns look found nothing");
                Vec::new()
            }
            Err(_) => Vec::new(),
        }
    }
}

/// Fixed answers, for tests: `set` what a name says, and the look reads it.
#[derive(Default)]
pub struct StaticDns {
    answers: Mutex<HashMap<(String, RecordKind), Vec<String>>>,
}

impl StaticDns {
    pub fn set(&self, name: &str, kind: RecordKind, values: &[&str]) {
        let mut answers = self.answers.lock().unwrap_or_else(|p| p.into_inner());
        answers.insert(
            (name.to_lowercase(), kind),
            values.iter().map(|v| v.to_string()).collect(),
        );
    }
}

#[async_trait::async_trait]
impl DnsLookup for StaticDns {
    async fn lookup(&self, name: &str, kind: RecordKind) -> Vec<String> {
        let answers = self.answers.lock().unwrap_or_else(|p| p.into_inner());
        answers
            .get(&(name.to_lowercase(), kind))
            .cloned()
            .unwrap_or_default()
    }
}

static RESOLVER: OnceLock<Arc<dyn DnsLookup>> = OnceLock::new();

/// Puts `resolver` in place of the system's, once, before the first look (tests).
pub fn install(resolver: Arc<dyn DnsLookup>) -> bool {
    RESOLVER.set(resolver).is_ok()
}

/// The resolver in use: the installed one, else the system's.
pub fn resolver() -> Arc<dyn DnsLookup> {
    RESOLVER
        .get_or_init(|| {
            let resolver = TokioResolver::builder_tokio()
                .and_then(|b| b.build())
                .map_err(|e| tracing::warn!(error = %e, "no system DNS resolver"))
                .ok();
            Arc::new(SystemDns { resolver })
        })
        .clone()
}
