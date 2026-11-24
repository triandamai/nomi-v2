//! Its own test binary, so `NOMI_WEB_ALLOW_PRIVATE` (set by the stub tests in `import.rs`) is
//! never on here.

use nomi_tools_skills::import::{from_link_with_bases, ImportError};

#[tokio::test]
async fn private_address_refused() {
    std::env::remove_var("NOMI_WEB_ALLOW_PRIVATE");
    let result = from_link_with_bases("https://raw.githubusercontent.com/acme/skills/main/SKILL.md", None, "http://127.0.0.1:9/api", "http://127.0.0.1:9/raw").await;
    assert!(matches!(result, Err(ImportError::PrivateAddress)), "{result:?}");
    let result = from_link_with_bases("https://localhost/menu.md", None, "http://127.0.0.1:9/api", "http://127.0.0.1:9/raw").await;
    assert!(matches!(result, Err(ImportError::PrivateAddress)), "{result:?}");
}
