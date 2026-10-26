//! Signing in with an outside provider. Google, Apple and any company login are OpenID Connect:
//! Relay reads the issuer's discovery document and checks the ID token against its keys. GitHub
//! isn't, so it has its own few calls.

use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::{secret_name, AuthConfig, Secrets, APPLE, GITHUB, GOOGLE, OIDC_PREFIX};
use crate::AuthError;

pub const GOOGLE_ISSUER: &str = "https://accounts.google.com";
pub const APPLE_ISSUER: &str = "https://appleid.apple.com";
const GITHUB_API: &str = "https://api.github.com";
const GITHUB_WEB: &str = "https://github.com";
const OIDC_SCOPES: &str = "openid email profile";
const APPLE_SCOPES: &str = "name email";
const GITHUB_SCOPES: &str = "read:user user:email";
/// Apple's client secret is made fresh for each exchange and lives this long.
const APPLE_SECRET_SECONDS: u64 = 300;
const DISCOVERY_PATH: &str = "/.well-known/openid-configuration";
/// GitHub refuses API calls without a User-Agent.
const USER_AGENT: &str = "nomi-relay";

/// Where to send someone for this sign-in, and how to find the flow when they come back.
pub struct Start {
    pub state: String,
    pub nonce: String,
    pub redirect_uri: String,
    /// Scopes the app asked for, instead of the provider's defaults.
    pub scopes: Option<String>,
}

/// Who the provider says signed in.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
}

pub enum ClientSecret {
    Plain(String),
    /// Signed from the `.p8` for each exchange.
    Apple { team_id: String, key_id: String, p8: String },
}

pub struct OidcProvider {
    /// `google`, `apple` or `oidc:<id>`.
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub secret: ClientSecret,
    pub scopes: String,
    /// Apple sends the person back with a form POST.
    pub form_post: bool,
}

pub struct GithubProvider {
    pub client_id: String,
    pub client_secret: String,
    pub api_base: String,
    pub web_base: String,
}

pub enum Provider {
    Oidc(OidcProvider),
    Github(GithubProvider),
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn not_ready(name: &str, what: &str) -> AuthError {
    AuthError::bad("PROVIDER_NOT_READY", format!("Signing in with {name} isn't ready: {what}."))
}

fn refused(message: impl Into<String>) -> AuthError {
    AuthError::unauthorized("PROVIDER_REFUSED", message)
}

fn bad_token(why: impl std::fmt::Display) -> AuthError {
    AuthError::unauthorized("BAD_ID_TOKEN", format!("The provider's sign-in couldn't be checked: {why}."))
}

/// Apple's client secret: an ES256 JWT from the Services ID's `.p8` key, good for five minutes.
pub fn apple_client_secret(team_id: &str, key_id: &str, client_id: &str, p8: &str, issued_at: u64) -> Result<String, AuthError> {
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id.to_string());
    let claims = json!({ "iss": team_id, "iat": issued_at, "exp": issued_at + APPLE_SECRET_SECONDS, "aud": APPLE_ISSUER, "sub": client_id });
    let key = EncodingKey::from_ec_pem(p8.as_bytes()).map_err(|_| not_ready("Apple", "its key (NOMI_OAUTH_APPLE_KEY) isn't a .p8 key"))?;
    encode(&header, &claims, &key).map_err(|e| AuthError::Internal(e.to_string()))
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

async fn discover(http: &reqwest::Client, issuer: &str) -> Result<Discovery, AuthError> {
    let issuer = issuer.trim_end_matches('/');
    let found: Discovery = http
        .get(format!("{issuer}{DISCOVERY_PATH}"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| AuthError::Unavailable { code: "PROVIDER_UNAVAILABLE", message: format!("The sign-in provider couldn't be reached: {e}") })?
        .json()
        .await
        .map_err(|e| AuthError::Unavailable { code: "PROVIDER_UNAVAILABLE", message: format!("The provider's discovery document isn't readable: {e}") })?;
    if found.issuer.trim_end_matches('/') != issuer {
        return Err(AuthError::Unavailable { code: "PROVIDER_UNAVAILABLE", message: "The provider's discovery document names another issuer.".into() });
    }
    Ok(found)
}

fn with_query(base: &str, pairs: &[(&str, &str)]) -> Result<String, AuthError> {
    let mut url = reqwest::Url::parse(base).map_err(|e| AuthError::Unavailable { code: "PROVIDER_UNAVAILABLE", message: format!("The provider's address isn't usable: {e}") })?;
    url.query_pairs_mut().extend_pairs(pairs);
    Ok(url.to_string())
}

/// `true`, or Apple's `"true"`.
fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "true",
        _ => false,
    }
}

impl Provider {
    /// The provider `name` as the project set it up; refused when it's off or missing its secret.
    pub fn resolve(config: &AuthConfig, secrets: &Secrets, name: &str) -> Result<Provider, AuthError> {
        if !config.allows(name) {
            return Err(AuthError::bad("METHOD_OFF", format!("Signing in with {name} is switched off for this app.")));
        }
        let secret = || secrets.get(&secret_name(name)).map(str::to_string).ok_or_else(|| not_ready(name, &format!("set the project secret {}", secret_name(name))));
        let oidc = |issuer: &str, client_id: &str, secret: ClientSecret, scopes: &str, form_post: bool| {
            Provider::Oidc(OidcProvider { name: name.to_string(), issuer: issuer.to_string(), client_id: client_id.to_string(), secret, scopes: scopes.to_string(), form_post })
        };
        match name {
            GOOGLE => {
                let client = config.google.as_ref().ok_or_else(|| not_ready(name, "add its client_id to backend.json"))?;
                Ok(oidc(GOOGLE_ISSUER, &client.client_id, ClientSecret::Plain(secret()?), OIDC_SCOPES, false))
            }
            APPLE => {
                let apple = config.apple.as_ref().ok_or_else(|| not_ready(name, "add its client_id, team_id and key_id to backend.json"))?;
                let key = ClientSecret::Apple { team_id: apple.team_id.clone(), key_id: apple.key_id.clone(), p8: secret()? };
                Ok(oidc(APPLE_ISSUER, &apple.client_id, key, APPLE_SCOPES, true))
            }
            GITHUB => {
                let github = config.github.as_ref().ok_or_else(|| not_ready(name, "add its client_id to backend.json"))?;
                Ok(Provider::Github(GithubProvider {
                    client_id: github.client_id.clone(),
                    client_secret: secret()?,
                    api_base: github.api_base.clone().unwrap_or_else(|| GITHUB_API.to_string()),
                    web_base: github.web_base.clone().unwrap_or_else(|| GITHUB_WEB.to_string()),
                }))
            }
            _ => {
                let id = name.strip_prefix(OIDC_PREFIX).ok_or_else(|| AuthError::bad("METHOD_OFF", format!("{name} isn't a sign-in method Relay knows.")))?;
                let found = config.oidc.iter().find(|o| o.id == id).ok_or_else(|| not_ready(name, "add it under oidc in backend.json"))?;
                Ok(oidc(&found.issuer, &found.client_id, ClientSecret::Plain(secret()?), OIDC_SCOPES, false))
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Provider::Oidc(p) => &p.name,
            Provider::Github(_) => GITHUB,
        }
    }

    /// Whether the provider comes back with a form POST.
    pub fn form_post(&self) -> bool {
        matches!(self, Provider::Oidc(p) if p.form_post)
    }

    pub async fn authorize_url(&self, http: &reqwest::Client, start: &Start) -> Result<String, AuthError> {
        match self {
            Provider::Oidc(p) => {
                let found = discover(http, &p.issuer).await?;
                let scopes = start.scopes.as_deref().unwrap_or(&p.scopes);
                let mut pairs = vec![
                    ("response_type", "code"),
                    ("client_id", p.client_id.as_str()),
                    ("redirect_uri", start.redirect_uri.as_str()),
                    ("scope", scopes),
                    ("state", start.state.as_str()),
                    ("nonce", start.nonce.as_str()),
                ];
                if p.form_post {
                    pairs.push(("response_mode", "form_post"));
                }
                with_query(&found.authorization_endpoint, &pairs)
            }
            Provider::Github(g) => {
                let scopes = start.scopes.as_deref().unwrap_or(GITHUB_SCOPES);
                with_query(
                    &format!("{}/login/oauth/authorize", g.web_base.trim_end_matches('/')),
                    &[("client_id", g.client_id.as_str()), ("redirect_uri", start.redirect_uri.as_str()), ("scope", scopes), ("state", start.state.as_str()), ("allow_signup", "true")],
                )
            }
        }
    }

    /// Trades the provider's `code` for who signed in.
    pub async fn exchange(&self, http: &reqwest::Client, code: &str, redirect_uri: &str, nonce: &str) -> Result<Profile, AuthError> {
        match self {
            Provider::Oidc(p) => exchange_oidc(http, p, code, redirect_uri, nonce).await,
            Provider::Github(g) => exchange_github(http, g, code, redirect_uri).await,
        }
    }
}

async fn exchange_oidc(http: &reqwest::Client, p: &OidcProvider, code: &str, redirect_uri: &str, nonce: &str) -> Result<Profile, AuthError> {
    let found = discover(http, &p.issuer).await?;
    let client_secret = match &p.secret {
        ClientSecret::Plain(secret) => secret.clone(),
        ClientSecret::Apple { team_id, key_id, p8 } => apple_client_secret(team_id, key_id, &p.client_id, p8, now())?,
    };
    let form = [("grant_type", "authorization_code"), ("code", code), ("redirect_uri", redirect_uri), ("client_id", p.client_id.as_str()), ("client_secret", client_secret.as_str())];
    let response = http.post(&found.token_endpoint).header("accept", "application/json").form(&form).send().await.map_err(|e| refused(format!("The provider couldn't be reached: {e}")))?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(refused(format!("The provider refused the sign-in: {}", body.chars().take(200).collect::<String>())));
    }
    let tokens: Value = response.json().await.map_err(|e| refused(format!("The provider's answer isn't readable: {e}")))?;
    let id_token = tokens.get("id_token").and_then(Value::as_str).ok_or_else(|| refused("The provider didn't send an ID token."))?;
    let claims = check_id_token(http, &found, p, id_token, nonce).await?;
    let sub = claims.get("sub").and_then(Value::as_str).ok_or_else(|| bad_token("it has no subject"))?;
    Ok(Profile {
        provider_user_id: sub.to_string(),
        email: claims.get("email").and_then(Value::as_str).map(|e| e.trim().to_lowercase()),
        email_verified: truthy(claims.get("email_verified")),
    })
}

async fn check_id_token(http: &reqwest::Client, found: &Discovery, p: &OidcProvider, id_token: &str, nonce: &str) -> Result<Value, AuthError> {
    let header = decode_header(id_token).map_err(bad_token)?;
    if !matches!(header.alg, Algorithm::RS256 | Algorithm::ES256) {
        return Err(bad_token("it isn't signed with the provider's key"));
    }
    let keys: JwkSet = http.get(&found.jwks_uri).send().await.map_err(bad_token)?.json().await.map_err(bad_token)?;
    let jwk = match &header.kid {
        Some(kid) => keys.find(kid),
        None => keys.keys.first(),
    }
    .ok_or_else(|| bad_token("its key isn't one the provider publishes"))?;
    let key = DecodingKey::from_jwk(jwk).map_err(bad_token)?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[&p.client_id]);
    validation.set_issuer(&[found.issuer.as_str(), found.issuer.trim_end_matches('/')]);
    let claims = decode::<Value>(id_token, &key, &validation).map_err(bad_token)?.claims;
    if claims.get("nonce").and_then(Value::as_str) != Some(nonce) {
        return Err(bad_token("it was made for another sign-in"));
    }
    Ok(claims)
}

#[derive(Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

async fn exchange_github(http: &reqwest::Client, g: &GithubProvider, code: &str, redirect_uri: &str) -> Result<Profile, AuthError> {
    let form = [("client_id", g.client_id.as_str()), ("client_secret", g.client_secret.as_str()), ("code", code), ("redirect_uri", redirect_uri)];
    let tokens: Value = http
        .post(format!("{}/login/oauth/access_token", g.web_base.trim_end_matches('/')))
        .header("accept", "application/json")
        .form(&form)
        .send()
        .await
        .map_err(|e| refused(format!("GitHub couldn't be reached: {e}")))?
        .json()
        .await
        .map_err(|e| refused(format!("GitHub's answer isn't readable: {e}")))?;
    let token = tokens.get("access_token").and_then(Value::as_str).ok_or_else(|| {
        refused(format!("GitHub refused the sign-in: {}", tokens.get("error_description").and_then(Value::as_str).unwrap_or("no access token")))
    })?;
    let api = g.api_base.trim_end_matches('/');
    let get = |path: &str| http.get(format!("{api}{path}")).bearer_auth(token).header("accept", "application/vnd.github+json").header("user-agent", USER_AGENT).send();
    let user: Value = get("/user").await.and_then(|r| r.error_for_status()).map_err(|e| refused(e.to_string()))?.json().await.map_err(|e| refused(e.to_string()))?;
    let id = user.get("id").and_then(Value::as_i64).ok_or_else(|| refused("GitHub didn't say who signed in."))?;
    let emails: Vec<GithubEmail> = match get("/user/emails").await.and_then(|r| r.error_for_status()) {
        Ok(response) => response.json().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let chosen = emails.iter().find(|e| e.primary && e.verified).or_else(|| emails.iter().find(|e| e.verified));
    Ok(Profile { provider_user_id: id.to_string(), email: chosen.map(|e| e.email.trim().to_lowercase()), email_verified: chosen.is_some() })
}
