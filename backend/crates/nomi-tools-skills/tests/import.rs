//! Skills from a zip, a markdown file or a link. Link tests run against a local stub standing in
//! for GitHub; they set `NOMI_WEB_ALLOW_PRIVATE=1` (the stub is on 127.0.0.1), except
//! `private_address_refused`, which runs in its own process (see `tests/import_private.rs`).

use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::Router;
use nomi_skills::{SkillError, MAX_SKILL_BYTES};
use nomi_tools_skills::import::{classify, from_link_with_bases, from_link_with_bases_within, from_markdown, from_zip, redirect_allowed, ImportError, LinkKind, MAX_ZIP_ENTRIES};
use zip::write::SimpleFileOptions;

const SKILL: &str = "---\nname: menu\ndescription: Use when planning meals for the week.\n---\n# Menu\nBody";

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        if name.ends_with('/') {
            out.add_directory(*name, SimpleFileOptions::default()).unwrap();
        } else {
            out.start_file(*name, SimpleFileOptions::default()).unwrap();
            out.write_all(bytes).unwrap();
        }
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn markdown_upload() {
    let imported = from_markdown(SKILL.as_bytes()).unwrap();
    assert_eq!(imported.skill.meta.name, "menu");
    assert!(matches!(from_markdown(b"# no frontmatter"), Err(ImportError::Invalid(SkillError::NoFrontmatter))));
    assert!(matches!(from_markdown(&[0xff, 0xfe, 0x00]), Err(ImportError::NotASkill(_))));
}

#[test]
fn zip_root_layout() {
    let imported = from_zip(&zip_of(&[("SKILL.md", SKILL.as_bytes()), ("references/pantry.md", b"rice")])).unwrap();
    assert_eq!(imported.skill.meta.name, "menu");
    assert_eq!(imported.skill.references.len(), 1);
    assert_eq!(imported.skill.references[0].name, "pantry");
    assert!(imported.skipped.is_empty());
}

#[test]
fn zip_one_folder_layout() {
    let imported = from_zip(&zip_of(&[("menu/", b""), ("menu/SKILL.md", SKILL.as_bytes()), ("menu/references/", b""), ("menu/references/pantry.md", b"rice")])).unwrap();
    assert_eq!(imported.skill.references[0].text, "rice");
    assert!(imported.skipped.is_empty(), "folders aren't files to skip: {:?}", imported.skipped);
}

#[test]
fn zip_extra_files_skipped() {
    let imported = from_zip(&zip_of(&[
        ("menu/SKILL.md", SKILL.as_bytes()),
        ("menu/scripts/run.py", b"print()"),
        ("menu/references/deep/x.md", b"x"),
        ("other/notes.md", b"x"),
        ("menu/logo.png", b"png"),
    ]))
    .unwrap();
    let mut skipped = imported.skipped.clone();
    skipped.sort();
    assert_eq!(skipped, vec!["menu/logo.png", "menu/references/deep/x.md", "menu/scripts/run.py", "other/notes.md"]);
}

#[test]
fn zip_two_skill_md_is_not_a_skill() {
    let bytes = zip_of(&[("a/SKILL.md", SKILL.as_bytes()), ("b/SKILL.md", SKILL.as_bytes())]);
    assert!(matches!(from_zip(&bytes), Err(ImportError::NotASkill(_))));
    assert!(matches!(from_zip(&zip_of(&[("a/b/SKILL.md", SKILL.as_bytes())])), Err(ImportError::NotASkill(_))), "two folders deep isn't a skill folder");
    assert!(matches!(from_zip(&zip_of(&[("readme.md", b"x")])), Err(ImportError::NotASkill(_))));
    assert!(matches!(from_zip(b"not a zip"), Err(ImportError::NotASkill(_))));
}

#[test]
fn zip_parent_path_refused() {
    for evil in ["../SKILL.md", "/etc/SKILL.md", "menu/../../x.md", "C:\\SKILL.md"] {
        let bytes = zip_of(&[("SKILL.md", SKILL.as_bytes()), (evil, b"x")]);
        assert!(matches!(from_zip(&bytes), Err(ImportError::NotASkill(_))), "{evil}");
    }
}

#[test]
fn zip_over_512kb_uncompressed_too_large() {
    // Compresses to almost nothing; counted by what is read, not by the header.
    let big = vec![b'a'; 600 * 1024];
    let bytes = zip_of(&[("SKILL.md", SKILL.as_bytes()), ("references/big.md", &big)]);
    assert!(bytes.len() < 64 * 1024);
    assert!(matches!(from_zip(&bytes), Err(ImportError::TooLarge)));
}

#[test]
fn zip_too_many_entries_too_large() {
    let names: Vec<String> = (0..MAX_ZIP_ENTRIES).map(|i| format!("junk{i}.txt")).collect();
    let mut entries: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), b"x".as_slice())).collect();
    entries.push(("SKILL.md", SKILL.as_bytes()));
    assert!(matches!(from_zip(&zip_of(&entries)), Err(ImportError::TooLarge)));
}

#[test]
fn links_are_classified() {
    assert_eq!(
        classify("https://github.com/acme/skills/tree/main/menu").unwrap(),
        LinkKind::GithubFolder { owner: "acme".into(), repo: "skills".into(), git_ref: "main".into(), path: "menu".into() }
    );
    assert_eq!(
        classify("https://github.com/acme/skills/blob/v2/menu/SKILL.md").unwrap(),
        LinkKind::RawFile { url: "https://raw.githubusercontent.com/acme/skills/v2/menu/SKILL.md".into() }
    );
    assert_eq!(
        classify("https://raw.githubusercontent.com/acme/skills/main/SKILL.md").unwrap(),
        LinkKind::RawFile { url: "https://raw.githubusercontent.com/acme/skills/main/SKILL.md".into() }
    );
    assert_eq!(classify("https://example.com/skills/menu.md").unwrap(), LinkKind::Other { url: "https://example.com/skills/menu.md".into() });
    assert!(matches!(classify("https://example.com/page"), Err(ImportError::NotASkill(_))));
    assert!(matches!(classify("not a url"), Err(ImportError::NotASkill(_))));
}

#[test]
fn http_scheme_refused() {
    assert!(matches!(classify("http://example.com/menu.md"), Err(ImportError::NotASkill(_))));
}

/// A stub of GitHub's contents API and raw host, recording the Authorization header per path.
#[derive(Clone, Default)]
struct Stub {
    seen: Arc<Mutex<Vec<(String, Option<String>)>>>,
    api_status: Option<u16>,
    big_skill: bool,
}

async fn api(State(stub): State<Stub>, Path(rest): Path<String>, headers: HeaderMap) -> (StatusCode, HeaderMap, String) {
    let (status, body) = api_body(&stub, &rest, &headers);
    let mut out = HeaderMap::new();
    if rest == "repos/acme/skills/contents/moved" {
        out.insert("location", "/raw/acme/landing".parse().unwrap());
        return (StatusCode::FOUND, out, String::new());
    }
    (status, out, body)
}

fn api_body(stub: &Stub, rest: &str, headers: &HeaderMap) -> (StatusCode, String) {
    stub.seen.lock().unwrap().push((format!("api/{rest}"), headers.get("authorization").map(|v| v.to_str().unwrap().to_string())));
    if let Some(code) = stub.api_status {
        return (StatusCode::from_u16(code).unwrap(), "{}".into());
    }
    match rest {
        "repos/acme/skills/contents/huge" => (StatusCode::OK, format!("[{}]", " ".repeat(300 * 1024))),
        "repos/acme/skills/contents/menu" => (StatusCode::OK, r#"[{"name":"SKILL.md","type":"file"},{"name":"references","type":"dir"},{"name":"run.py","type":"file"}]"#.into()),
        "repos/acme/skills/contents/menu/references" => (StatusCode::OK, r#"[{"name":"pantry.md","type":"file"},{"name":"img.png","type":"file"}]"#.into()),
        _ => (StatusCode::NOT_FOUND, "{}".into()),
    }
}

async fn raw(State(stub): State<Stub>, Path(rest): Path<String>, headers: HeaderMap) -> (StatusCode, String) {
    stub.seen.lock().unwrap().push((format!("raw/{rest}"), headers.get("authorization").map(|v| v.to_str().unwrap().to_string())));
    if rest == "acme/skills/main/slow/SKILL.md" {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    }
    match rest.as_str() {
        "acme/skills/main/menu/SKILL.md" | "acme/skills/v2/menu/SKILL.md" if stub.big_skill => (StatusCode::OK, "x".repeat(MAX_SKILL_BYTES * 2)),
        "acme/skills/main/menu/SKILL.md" | "acme/skills/v2/menu/SKILL.md" => (StatusCode::OK, SKILL.into()),
        "acme/skills/main/menu/references/pantry.md" => (StatusCode::OK, "rice".into()),
        "acme/skills/main/slow/SKILL.md" => (StatusCode::OK, SKILL.into()),
        _ => (StatusCode::NOT_FOUND, String::new()),
    }
}

async fn serve(stub: Stub) -> (String, String) {
    std::env::set_var("NOMI_WEB_ALLOW_PRIVATE", "1");
    let app = Router::new().route("/api/*rest", get(api)).route("/raw/*rest", get(raw)).with_state(stub);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}/api"), format!("http://{addr}/raw"))
}

#[tokio::test]
async fn github_folder_import() {
    let stub = Stub::default();
    let (api_base, raw_base) = serve(stub.clone()).await;
    let imported = from_link_with_bases("https://github.com/acme/skills/tree/main/menu", None, &api_base, &raw_base).await.unwrap();
    assert_eq!(imported.skill.meta.name, "menu");
    assert_eq!(imported.skill.references.len(), 1);
    assert_eq!(imported.skill.references[0].text, "rice");
    let mut skipped = imported.skipped.clone();
    skipped.sort();
    assert_eq!(skipped, vec!["menu/references/img.png", "menu/run.py"]);
    assert_eq!(stub.seen.lock().unwrap().len(), 4, "two listings, SKILL.md and one reference");
}

#[tokio::test]
async fn github_blob_import() {
    let (api_base, raw_base) = serve(Stub::default()).await;
    let imported = from_link_with_bases("https://github.com/acme/skills/blob/v2/menu/SKILL.md", None, &api_base, &raw_base).await.unwrap();
    assert_eq!(imported.skill.meta.name, "menu");
}

#[tokio::test]
async fn raw_md_import() {
    let (api_base, raw_base) = serve(Stub::default()).await;
    let imported = from_link_with_bases("https://raw.githubusercontent.com/acme/skills/main/menu/SKILL.md", None, &api_base, &raw_base).await.unwrap();
    assert!(imported.skill.text.contains("# Menu"));
    let missing = from_link_with_bases("https://raw.githubusercontent.com/acme/skills/main/gone/SKILL.md", None, &api_base, &raw_base).await;
    assert!(matches!(missing, Err(ImportError::FetchFailed(_))));
}

#[tokio::test]
async fn github_403_is_rate_limited() {
    for code in [403, 429] {
        let (api_base, raw_base) = serve(Stub { api_status: Some(code), ..Stub::default() }).await;
        let result = from_link_with_bases("https://github.com/acme/skills/tree/main/menu", None, &api_base, &raw_base).await;
        assert!(matches!(result, Err(ImportError::RateLimited)), "{code}");
    }
}

#[tokio::test]
async fn token_sent_only_to_api() {
    let stub = Stub::default();
    let (api_base, raw_base) = serve(stub.clone()).await;
    from_link_with_bases("https://github.com/acme/skills/tree/main/menu", Some("secret"), &api_base, &raw_base).await.unwrap();
    for (path, auth) in stub.seen.lock().unwrap().iter() {
        if path.starts_with("api/") {
            assert_eq!(auth.as_deref(), Some("Bearer secret"), "{path}");
        } else {
            assert_eq!(auth, &None, "{path}");
        }
    }
}

#[tokio::test]
async fn oversized_file_too_large() {
    let (api_base, raw_base) = serve(Stub { big_skill: true, ..Stub::default() }).await;
    let result = from_link_with_bases("https://github.com/acme/skills/blob/v2/menu/SKILL.md", None, &api_base, &raw_base).await;
    assert!(matches!(result, Err(ImportError::TooLarge)));
}

#[tokio::test]
async fn a_huge_listing_is_too_large_while_streaming() {
    let (api_base, raw_base) = serve(Stub::default()).await;
    let result = from_link_with_bases("https://github.com/acme/skills/tree/main/huge", None, &api_base, &raw_base).await;
    assert!(matches!(result, Err(ImportError::TooLarge)), "{result:?}");
}

#[tokio::test]
async fn a_redirect_drops_the_token() {
    let stub = Stub::default();
    let (api_base, raw_base) = serve(stub.clone()).await;
    let _ = from_link_with_bases("https://github.com/acme/skills/tree/main/moved", Some("secret"), &api_base, &raw_base).await;
    let seen = stub.seen.lock().unwrap();
    let landing = seen.iter().find(|(path, _)| path == "raw/acme/landing").expect("the redirect was followed");
    assert_eq!(landing.1, None);
}

#[tokio::test]
async fn a_whole_import_has_a_deadline() {
    let (api_base, raw_base) = serve(Stub::default()).await;
    let started = std::time::Instant::now();
    let result = from_link_with_bases_within(
        "https://raw.githubusercontent.com/acme/skills/main/slow/SKILL.md",
        None,
        &api_base,
        &raw_base,
        std::time::Duration::from_millis(300),
    )
    .await;
    assert!(matches!(result, Err(ImportError::FetchFailed(_))), "{result:?}");
    assert!(started.elapsed() < std::time::Duration::from_millis(1200));
}

#[test]
fn a_redirect_never_drops_to_http() {
    let url = |u: &str| reqwest::Url::parse(u).unwrap();
    assert!(redirect_allowed(&url("https://a.example/x"), &url("https://b.example/y")));
    assert!(!redirect_allowed(&url("https://a.example/x"), &url("http://b.example/y")));
    assert!(!redirect_allowed(&url("https://a.example/x"), &url("ftp://b.example/y")));
    assert!(redirect_allowed(&url("http://127.0.0.1/x"), &url("http://127.0.0.1/y")), "the tests' plain-http stub");
}
