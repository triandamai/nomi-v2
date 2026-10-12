//! Shipyard signs every webhook: `Shipyard-Signature: t=<unix seconds>,v1=<hex>`, where
//! `v1 = HMAC-SHA256(secret, "<t>.<raw body>")`. Checked before the body is parsed.

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// How far the signature's time may be from now.
pub const TOLERANCE_SECS: i64 = 300;

pub fn sign(secret: &str, timestamp: i64, body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC takes any key length");
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

/// Whether `header` is a valid signature of `body` made with `secret` within the tolerance.
pub fn verify(secret: &str, header: &str, body: &[u8], now: i64) -> bool {
    let mut timestamp = None;
    let mut given = None;
    for part in header.split(',') {
        match part.trim().split_once('=') {
            Some(("t", v)) => timestamp = v.parse::<i64>().ok(),
            Some(("v1", v)) => given = hex::decode(v).ok(),
            _ => {}
        }
    }
    let (Some(timestamp), Some(given)) = (timestamp, given) else { return false };
    if (now - timestamp).abs() > TOLERANCE_SECS {
        return false;
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC takes any key length");
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(body);
    // Constant-time comparison.
    mac.verify_slice(&given).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_its_own_signature() {
        let body = br#"{"id":"e1","type":"ping"}"#;
        let header = format!("t=1000,v1={}", sign("whsec_x", 1000, body));
        assert!(verify("whsec_x", &header, body, 1100));
    }

    #[test]
    fn rejects_tampering_wrong_secret_and_stale_times() {
        let body = br#"{"id":"e1"}"#;
        let header = format!("t=1000,v1={}", sign("whsec_x", 1000, body));
        assert!(!verify("whsec_x", &header, br#"{"id":"e2"}"#, 1000));
        assert!(!verify("whsec_y", &header, body, 1000));
        assert!(!verify("whsec_x", &header, body, 1000 + TOLERANCE_SECS + 1));
        assert!(!verify("whsec_x", "garbage", body, 1000));
        assert!(!verify("whsec_x", "t=1000,v1=zz", body, 1000));
    }
}
