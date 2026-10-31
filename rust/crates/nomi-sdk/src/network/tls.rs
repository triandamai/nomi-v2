//! One rustls configuration for HTTP and the realtime socket.

use std::sync::Arc;

use rustls::ClientConfig;

use crate::error::NomiError;

const HTTP1_ALPN: &[u8] = b"http/1.1";

/// rustls on ring with Mozilla's roots (webpki-roots), the same on every platform.
/// ALPN offers HTTP/1.1 only: the realtime upgrade needs it, and REST calls are small.
pub(crate) fn client_config() -> Result<Arc<ClientConfig>, NomiError> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let mut config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|error| NomiError::Internal {
                reason: format!("TLS setup: {error}"),
            })?
            .with_root_certificates(roots)
            .with_no_client_auth();
    config.alpn_protocols = vec![HTTP1_ALPN.to_vec()];
    Ok(Arc::new(config))
}
