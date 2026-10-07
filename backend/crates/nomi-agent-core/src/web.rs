//! The crew's view of the internet: `web_search` (Tavily by default; Serper, Brave or a
//! self-hosted SearXNG instead) and `read_web_page`. Both are engine tools any agent gets while
//! they're on in Admin → Tools.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::OnceLock;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;

use crate::tools;

pub const WEB_SEARCH_TOOL_NAME: &str = "web_search";
pub const READ_WEB_PAGE_TOOL_NAME: &str = "read_web_page";

pub fn is_web_tool(name: &str) -> bool {
    name == WEB_SEARCH_TOOL_NAME || name == READ_WEB_PAGE_TOOL_NAME
}

/// Where `web_search` gets its results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchProvider {
    /// Results summarized for AI; 1,000 free searches a month.
    #[default]
    Tavily,
    /// Google results, cheapest per search; snippets only.
    Serper,
    /// Brave's own index.
    Brave,
    /// A self-hosted SearXNG: no per-search cost.
    Searxng,
}

impl SearchProvider {
    pub const ALL: [SearchProvider; 4] = [SearchProvider::Tavily, SearchProvider::Serper, SearchProvider::Brave, SearchProvider::Searxng];

    pub fn as_str(self) -> &'static str {
        match self {
            SearchProvider::Tavily => "tavily",
            SearchProvider::Serper => "serper",
            SearchProvider::Brave => "brave",
            SearchProvider::Searxng => "searxng",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == value)
    }

    /// The environment variable a server can set the key (or, for SearXNG, the address) in
    /// instead of saving it in Admin → Tools.
    pub fn env_var(self) -> &'static str {
        match self {
            SearchProvider::Tavily => "TAVILY_API_KEY",
            SearchProvider::Serper => "SERPER_API_KEY",
            SearchProvider::Brave => "BRAVE_SEARCH_API_KEY",
            SearchProvider::Searxng => "SEARXNG_URL",
        }
    }

    /// SearXNG needs an address instead of a key.
    pub fn needs_key(self) -> bool {
        self != SearchProvider::Searxng
    }

    fn default_base_url(self) -> Option<&'static str> {
        match self {
            SearchProvider::Tavily => Some("https://api.tavily.com"),
            SearchProvider::Serper => Some("https://google.serper.dev"),
            SearchProvider::Brave => Some("https://api.search.brave.com"),
            SearchProvider::Searxng => None,
        }
    }
}

/// `web_search`'s settings as saved (its keys are kept apart, encrypted).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WebSearchSettings {
    pub provider: SearchProvider,
    /// Results per search.
    pub max_results: u8,
    /// The SearXNG address, or another address for the chosen provider (a proxy).
    pub base_url: Option<String>,
}

impl Default for WebSearchSettings {
    fn default() -> Self {
        Self { provider: SearchProvider::Tavily, max_results: 5, base_url: None }
    }
}

pub const MAX_RESULTS_LIMIT: u8 = 10;

/// `read_web_page`'s settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ReadPageSettings {
    /// Most text handed to the agent from one page.
    pub max_chars: usize,
}

impl Default for ReadPageSettings {
    fn default() -> Self {
        Self { max_chars: 12_000 }
    }
}

/// Everything a search needs, ready to use.
#[derive(Debug, Clone)]
pub struct SearchSetup {
    pub provider: SearchProvider,
    pub api_key: Option<String>,
    pub base_url: String,
    pub max_results: u8,
}

/// Where the chosen provider's key comes from, for Admin → Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeySource {
    Saved,
    Env,
    Missing,
}

/// The saved settings and secrets, and whether each provider has its key (or address).
pub async fn search_settings(conn: &mut PgConnection) -> (WebSearchSettings, HashMap<String, String>) {
    let row = tools::load_row(conn, WEB_SEARCH_TOOL_NAME).await.unwrap_or_default();
    let settings: WebSearchSettings = serde_json::from_value(row.config).unwrap_or_default();
    (settings, tools::decrypt_secrets(row.secrets_encrypted.as_deref()))
}

/// Same as `search_settings`, decrypting with `key` (the server's settings key).
pub async fn search_settings_with(conn: &mut PgConnection, key: &[u8; 32]) -> Result<(WebSearchSettings, HashMap<String, String>), sqlx::Error> {
    let row = tools::load_row(conn, WEB_SEARCH_TOOL_NAME).await?;
    let settings: WebSearchSettings = serde_json::from_value(row.config).unwrap_or_default();
    Ok((settings, tools::decrypt_secrets_with(key, row.secrets_encrypted.as_deref())))
}

/// Where `provider`'s key comes from.
pub fn key_source(provider: SearchProvider, settings: &WebSearchSettings, secrets: &HashMap<String, String>) -> KeySource {
    if provider == SearchProvider::Searxng {
        if settings.base_url.as_deref().is_some_and(|u| !u.trim().is_empty()) {
            return KeySource::Saved;
        }
    } else if secrets.get(provider.as_str()).is_some_and(|k| !k.trim().is_empty()) {
        return KeySource::Saved;
    }
    if std::env::var(provider.env_var()).is_ok_and(|v| !v.trim().is_empty()) {
        KeySource::Env
    } else {
        KeySource::Missing
    }
}

/// The search setup when the chosen provider is ready to use, else `None` (the tool isn't
/// offered until it is).
pub async fn search_setup(conn: &mut PgConnection) -> Option<SearchSetup> {
    let (settings, secrets) = search_settings(conn).await;
    setup_from(&settings, &secrets)
}

pub fn setup_from(settings: &WebSearchSettings, secrets: &HashMap<String, String>) -> Option<SearchSetup> {
    let provider = settings.provider;
    let env = std::env::var(provider.env_var()).ok().filter(|v| !v.trim().is_empty());
    let saved_base = settings.base_url.clone().filter(|u| !u.trim().is_empty());
    let (api_key, base_url) = if provider.needs_key() {
        let key = secrets.get(provider.as_str()).cloned().filter(|k| !k.trim().is_empty()).or(env)?;
        (Some(key), saved_base.unwrap_or_else(|| provider.default_base_url().unwrap_or_default().to_string()))
    } else {
        (None, saved_base.or(env)?)
    };
    Some(SearchSetup {
        provider,
        api_key,
        base_url: base_url.trim_end_matches('/').to_string(),
        max_results: settings.max_results.clamp(1, MAX_RESULTS_LIMIT),
    })
}

/// One search result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .user_agent("Mozilla/5.0 (compatible; NomiBot/1.0; +https://nomi.app)")
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                // A page may redirect, but never into the server's own network.
                if attempt.previous().len() >= 5 {
                    attempt.error("too many redirects")
                } else if attempt.url().host_str().is_some_and(is_private_host_literal) {
                    attempt.error("redirect to a private address")
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .expect("failed to build the web tools' HTTP client")
    })
}

/// Searches the web with the configured provider.
pub async fn search(setup: &SearchSetup, query: &str) -> Result<Vec<SearchHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("query must not be empty".to_string());
    }
    let n = setup.max_results as usize;
    let key = setup.api_key.as_deref().unwrap_or_default();
    let base = &setup.base_url;
    let request = match setup.provider {
        SearchProvider::Tavily => http()
            .post(format!("{base}/search"))
            .bearer_auth(key)
            .json(&json!({ "query": query, "max_results": n, "search_depth": "basic", "api_key": key })),
        SearchProvider::Serper => http().post(format!("{base}/search")).header("X-API-KEY", key).json(&json!({ "q": query, "num": n })),
        SearchProvider::Brave => http()
            .get(format!("{base}/res/v1/web/search"))
            .header("X-Subscription-Token", key)
            .header("Accept", "application/json")
            .query(&[("q", query), ("count", &n.to_string())]),
        SearchProvider::Searxng => http().get(format!("{base}/search")).query(&[("q", query), ("format", "json")]),
    };
    let response = request.send().await.map_err(|e| format!("{} couldn't be reached: {e}", setup.provider.as_str()))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("{} returned {status}: {}", setup.provider.as_str(), body.chars().take(300).collect::<String>()));
    }
    let body: Value = response.json().await.map_err(|e| format!("{} sent a reply that couldn't be read: {e}", setup.provider.as_str()))?;
    let mut hits = parse_hits(setup.provider, &body);
    hits.truncate(n);
    Ok(hits)
}

/// Each provider's results, as one shape.
pub fn parse_hits(provider: SearchProvider, body: &Value) -> Vec<SearchHit> {
    let (list, url_key, snippet_key) = match provider {
        SearchProvider::Tavily => (body.get("results"), "url", "content"),
        SearchProvider::Serper => (body.get("organic"), "link", "snippet"),
        SearchProvider::Brave => (body.get("web").and_then(|w| w.get("results")), "url", "description"),
        SearchProvider::Searxng => (body.get("results"), "url", "content"),
    };
    let text = |v: Option<&Value>| v.and_then(|v| v.as_str()).map(|s| strip_tags(s).trim().to_string()).unwrap_or_default();
    list.and_then(|l| l.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let url = text(item.get(url_key));
                    (!url.is_empty()).then(|| SearchHit { title: text(item.get("title")), url, snippet: text(item.get(snippet_key)) })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The results as the agent reads them: numbered, each with its link to cite.
pub fn format_hits(query: &str, hits: &[SearchHit]) -> String {
    if hits.is_empty() {
        return format!("No results for \"{query}\".");
    }
    let mut out = format!("Results for \"{query}\" (cite the links you use):\n");
    for (i, hit) in hits.iter().enumerate() {
        out.push_str(&format!("\n{}. {}\n   {}\n", i + 1, if hit.title.is_empty() { &hit.url } else { &hit.title }, hit.url));
        if !hit.snippet.is_empty() {
            out.push_str(&format!("   {}\n", hit.snippet.chars().take(500).collect::<String>()));
        }
    }
    out
}

/// `web_search` as the agent sees it.
pub fn web_search_tool_definition() -> nomi_llm::ToolDefinition {
    nomi_llm::ToolDefinition {
        name: WEB_SEARCH_TOOL_NAME.to_string(),
        description: "Search the internet for current or factual information you don't already know \
                      (news, prices, schedules, facts to check). Returns titles, links and snippets; \
                      use read_web_page on a link for the full text. Cite the links you use."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"query": {"type": "string", "description": "What to search for, as you'd type it into a search engine"}},
            "required": ["query"]
        }),
    }
}

/// `read_web_page` as the agent sees it.
pub fn read_web_page_tool_definition() -> nomi_llm::ToolDefinition {
    nomi_llm::ToolDefinition {
        name: READ_WEB_PAGE_TOOL_NAME.to_string(),
        description: "Read a public web page as plain text, e.g. a link from web_search or one the user shared.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"url": {"type": "string", "description": "The page's full http(s) address"}},
            "required": ["url"]
        }),
    }
}

/// Runs a web tool call. `None` when `name` isn't a web tool.
pub async fn execute(conn: &mut PgConnection, name: &str, input: &Value) -> Option<Result<String, String>> {
    match name {
        WEB_SEARCH_TOOL_NAME => {
            let query = input.get("query").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            if query.is_empty() {
                return Some(Err("query is required".to_string()));
            }
            let Some(setup) = search_setup(conn).await else {
                return Some(Err("web search isn't set up yet (an admin needs to add a search key in Admin → Tools)".to_string()));
            };
            Some(search(&setup, &query).await.map(|hits| format_hits(&query, &hits)))
        }
        READ_WEB_PAGE_TOOL_NAME => {
            let url = input.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if url.trim().is_empty() {
                return Some(Err("url is required".to_string()));
            }
            let row = tools::load_row(conn, READ_WEB_PAGE_TOOL_NAME).await.unwrap_or_default();
            let settings: ReadPageSettings = serde_json::from_value(row.config).unwrap_or_default();
            Some(read_page(&url, settings.max_chars).await)
        }
        _ => None,
    }
}

/// Most bytes downloaded from one page.
const MAX_PAGE_BYTES: usize = 2 * 1024 * 1024;

/// Reads a web page as plain text (its title first), at most `max_chars` characters.
pub async fn read_page(url: &str, max_chars: usize) -> Result<String, String> {
    let parsed = reqwest::Url::parse(url.trim()).map_err(|_| "that isn't a valid web address".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("only http and https pages can be read".to_string());
    }
    let host = parsed.host_str().ok_or("that web address has no host")?.to_string();
    ensure_public_host(&host, parsed.port_or_known_default().unwrap_or(443)).await?;

    let response = http().get(parsed).send().await.map_err(|e| format!("couldn't open the page: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("the page answered {status}"));
    }
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_ascii_lowercase();
    let is_html = content_type.contains("html") || content_type.is_empty();
    if !is_html && !content_type.starts_with("text/") && !content_type.contains("json") && !content_type.contains("xml") {
        return Err(format!("that link isn't a web page ({content_type})"));
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("the page stopped loading: {e}"))?;
        bytes.extend_from_slice(&chunk);
        if bytes.len() >= MAX_PAGE_BYTES {
            bytes.truncate(MAX_PAGE_BYTES);
            break;
        }
    }
    let raw = String::from_utf8_lossy(&bytes);
    let (title, text) = if is_html { html_to_text(&raw) } else { (None, raw.trim().to_string()) };
    let mut out = String::new();
    if let Some(title) = title {
        out.push_str(&format!("# {title}\n\n"));
    }
    let total = text.chars().count();
    out.push_str(&text.chars().take(max_chars).collect::<String>());
    if total > max_chars {
        out.push_str(&format!("\n\n[…the page goes on; {} more characters not shown]", total - max_chars));
    }
    Ok(out)
}

/// Pages on the server's own network (localhost, private ranges, cloud metadata) are off-limits:
/// an agent must never be steered into reading internal services. `NOMI_WEB_ALLOW_PRIVATE=1`
/// lifts this for local development and tests.
async fn ensure_public_host(host: &str, port: u16) -> Result<(), String> {
    if std::env::var("NOMI_WEB_ALLOW_PRIVATE").is_ok_and(|v| v == "1") {
        return Ok(());
    }
    if is_private_host_literal(host) {
        return Err("pages on private or local addresses can't be read".to_string());
    }
    let addrs = tokio::net::lookup_host((host.trim_matches(['[', ']']), port)).await.map_err(|_| "that site couldn't be found".to_string())?;
    let mut any = false;
    for addr in addrs {
        any = true;
        if !is_public_ip(addr.ip()) {
            return Err("pages on private or local addresses can't be read".to_string());
        }
    }
    if any {
        Ok(())
    } else {
        Err("that site couldn't be found".to_string())
    }
}

fn is_private_host_literal(host: &str) -> bool {
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".internal") || host.ends_with(".local") {
        return true;
    }
    host.parse::<IpAddr>().is_ok_and(|ip| !is_public_ip(ip))
}

/// Whether an address is on the public internet.
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_documentation()
                || a == 0
                || (a == 100 && (64..128).contains(&b)) // carrier-grade NAT
                || (a == 198 && (18..20).contains(&b)))
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            !(v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() || (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80)
        }
    }
}

fn strip_tags(s: &str) -> String {
    static TAG: OnceLock<regex::Regex> = OnceLock::new();
    decode_entities(&TAG.get_or_init(|| regex::Regex::new(r"(?s)<[^>]*>").unwrap()).replace_all(s, ""))
}

fn decode_entities(s: &str) -> String {
    static NUMERIC: OnceLock<regex::Regex> = OnceLock::new();
    let s = NUMERIC.get_or_init(|| regex::Regex::new(r"&#(x[0-9a-fA-F]+|[0-9]+);").unwrap()).replace_all(s, |c: &regex::Captures| {
        let code = &c[1];
        let n = if let Some(hex) = code.strip_prefix('x') { u32::from_str_radix(hex, 16).ok() } else { code.parse().ok() };
        n.and_then(char::from_u32).map(String::from).unwrap_or_default()
    });
    s.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// A page's title and its readable text: scripts, styles and navigation chrome dropped, block
/// elements kept as line breaks.
pub fn html_to_text(html: &str) -> (Option<String>, String) {
    static TITLE: OnceLock<regex::Regex> = OnceLock::new();
    static DROP: OnceLock<regex::Regex> = OnceLock::new();
    static BREAK: OnceLock<regex::Regex> = OnceLock::new();
    static BLANKS: OnceLock<regex::Regex> = OnceLock::new();
    static SPACES: OnceLock<regex::Regex> = OnceLock::new();
    let title = TITLE
        .get_or_init(|| regex::Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap())
        .captures(html)
        .map(|c| decode_entities(c[1].trim()))
        .filter(|t| !t.is_empty());
    let body = DROP
        .get_or_init(|| {
            // One alternative per element: the regex crate has no backreferences.
            let elements = ["script", "style", "noscript", "svg", "head", "nav", "footer", "iframe", "template"];
            let pattern = elements.iter().map(|e| format!(r"<{e}\b.*?</{e}\s*>")).chain([r"<!--.*?-->".to_string()]).collect::<Vec<_>>().join("|");
            regex::Regex::new(&format!("(?is){pattern}")).unwrap()
        })
        .replace_all(html, " ");
    let body = BREAK
        .get_or_init(|| regex::Regex::new(r"(?i)<(br|/p|/div|/li|/h[1-6]|/tr|/section|/article|/blockquote|/pre|hr)\b[^>]*>").unwrap())
        .replace_all(&body, "\n");
    let text = strip_tags(&body);
    let text = SPACES.get_or_init(|| regex::Regex::new(r"[ \t\r\u{a0}]+").unwrap()).replace_all(&text, " ");
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let text = BLANKS.get_or_init(|| regex::Regex::new(r"\n{3,}").unwrap()).replace_all(&lines.join("\n"), "\n\n").trim().to_string();
    (title, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_provider_reads_as_the_same_results() {
        let tavily = json!({"results": [{"title": "Bali", "url": "https://a.example", "content": "Island"}]});
        let serper = json!({"organic": [{"title": "Bali", "link": "https://a.example", "snippet": "Island"}]});
        let brave = json!({"web": {"results": [{"title": "Bali", "url": "https://a.example", "description": "<strong>Island</strong>"}]}});
        let searxng = json!({"results": [{"title": "Bali", "url": "https://a.example", "content": "Island"}]});
        let expected = vec![SearchHit { title: "Bali".into(), url: "https://a.example".into(), snippet: "Island".into() }];
        assert_eq!(parse_hits(SearchProvider::Tavily, &tavily), expected);
        assert_eq!(parse_hits(SearchProvider::Serper, &serper), expected);
        assert_eq!(parse_hits(SearchProvider::Brave, &brave), expected);
        assert_eq!(parse_hits(SearchProvider::Searxng, &searxng), expected);
    }

    #[test]
    fn a_page_reads_as_its_title_and_text() {
        let html = "<html><head><title>Ubud &amp; more</title><style>p{}</style></head><body><nav>Menu</nav>\
                    <h1>Ubud</h1><p>Rice&nbsp;terraces<br>and temples.</p><script>track()</script><p>Day&#39;s plan</p></body></html>";
        let (title, text) = html_to_text(html);
        assert_eq!(title.as_deref(), Some("Ubud & more"));
        assert_eq!(text, "Ubud\nRice terraces\nand temples.\nDay's plan");
    }

    #[test]
    fn private_and_local_addresses_are_not_public() {
        for ip in ["127.0.0.1", "10.0.0.5", "192.168.1.1", "172.16.0.1", "169.254.169.254", "100.64.0.1", "::1", "fd00::1", "fe80::1", "::ffff:10.0.0.1"] {
            assert!(!is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            assert!(is_public_ip(ip.parse().unwrap()), "{ip}");
        }
        assert!(is_private_host_literal("localhost") && is_private_host_literal("metadata.google.internal") && !is_private_host_literal("example.com"));
    }

    #[test]
    fn the_chosen_provider_is_ready_only_with_its_key_or_address() {
        let settings = WebSearchSettings::default();
        let mut secrets = HashMap::new();
        if std::env::var("TAVILY_API_KEY").is_err() {
            assert!(setup_from(&settings, &secrets).is_none());
        }
        secrets.insert("tavily".to_string(), "tvly-123".to_string());
        let setup = setup_from(&settings, &secrets).unwrap();
        assert_eq!((setup.provider, setup.api_key.as_deref(), setup.base_url.as_str()), (SearchProvider::Tavily, Some("tvly-123"), "https://api.tavily.com"));

        let searxng = WebSearchSettings { provider: SearchProvider::Searxng, base_url: Some("http://search.local/".into()), ..Default::default() };
        let setup = setup_from(&searxng, &HashMap::new()).unwrap();
        assert_eq!((setup.api_key, setup.base_url.as_str()), (None, "http://search.local"));
    }
}
