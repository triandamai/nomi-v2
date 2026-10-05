//! Sign in with Google: create an account, sign in, or add Google to an account that already has
//! a password. Uses the same server OAuth client as the Workspace agent (identity scopes only:
//! signing in never grants Workspace access; that is a separate, per-user connection).

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_workspace::connection::GoogleConfig;

/// What a finished Google sign-in did.
#[derive(Debug, Clone, PartialEq)]
pub enum Finished {
    /// Signed in as `user_id`; `is_new` when this sign-in created the account.
    SignedIn { user_id: Uuid, email: String, is_new: bool },
    /// Google was added to the already signed-in user.
    Linked { user_id: Uuid, email: String },
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SignInError {
    #[error("This Google sign-in has expired. Try again.")]
    Expired,
    #[error("Google didn't accept the sign-in. Try again.")]
    GoogleRejected,
    #[error("Your Google email isn't verified yet. Verify it with Google, then try again.")]
    EmailNotVerified,
    #[error("That Google account already signs in to a different Nomi account.")]
    LinkedElsewhere,
    #[error("That invite code isn't valid any more.")]
    InvalidInvite,
    #[error("{0}")]
    Other(String),
}

impl From<sqlx::Error> for SignInError {
    fn from(e: sqlx::Error) -> Self {
        SignInError::Other(e.to_string())
    }
}

pub enum Purpose {
    /// Not signed in yet. `invite_code` joins that organization when the account is new.
    SignIn { invite_code: Option<String> },
    /// Add Google to this signed-in user.
    Link { user_id: Uuid },
}

/// The Google page to send the person to.
pub async fn start(pool: &PgPool, config: &GoogleConfig, purpose: Purpose) -> Result<String, SignInError> {
    let state = {
        use rand::RngCore;
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    };
    let (kind, user_id, invite_code) = match purpose {
        Purpose::SignIn { invite_code } => ("sign_in", None, invite_code.filter(|c| !c.trim().is_empty())),
        Purpose::Link { user_id } => ("link", Some(user_id), None),
    };
    sqlx::query("DELETE FROM google_sign_in_states WHERE created_at < now() - interval '10 minutes'").execute(pool).await?;
    sqlx::query("INSERT INTO google_sign_in_states (state, purpose, user_id, invite_code) VALUES ($1, $2, $3, $4)")
        .bind(&state)
        .bind(kind)
        .bind(user_id)
        .bind(&invite_code)
        .execute(pool)
        .await?;

    let mut url = reqwest::Url::parse(&config.auth_url).map_err(|e| SignInError::Other(e.to_string()))?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.signin_redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "openid email profile")
        .append_pair("prompt", "select_account")
        .append_pair("state", &state);
    Ok(url.to_string())
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserInfo {
    sub: String,
    email: String,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    given_name: Option<String>,
}

/// Finishes a Google sign-in: checks the state, asks Google who this is, then signs in, creates
/// the account, or links Google, as the state's purpose says.
pub async fn finish(pool: &PgPool, http: &reqwest::Client, config: &GoogleConfig, code: &str, state: &str) -> Result<Finished, SignInError> {
    let pending: Option<(String, Option<Uuid>, Option<String>)> = sqlx::query_as(
        "DELETE FROM google_sign_in_states WHERE state = $1 AND created_at > now() - interval '10 minutes' \
         RETURNING purpose, user_id, invite_code",
    )
    .bind(state)
    .fetch_optional(pool)
    .await?;
    let (purpose, link_user, invite_code) = pending.ok_or(SignInError::Expired)?;

    let response = http
        .post(&config.token_url)
        .form(&[
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.signin_redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| SignInError::Other(e.to_string()))?;
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "google sign-in: code exchange failed");
        return Err(SignInError::GoogleRejected);
    }
    let tokens: TokenResponse = response.json().await.map_err(|_| SignInError::GoogleRejected)?;
    let info: UserInfo = http
        .get(&config.userinfo_url)
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|_| SignInError::GoogleRejected)?
        .json()
        .await
        .map_err(|_| SignInError::GoogleRejected)?;
    let email = info.email.trim().to_lowercase();

    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM google_identities WHERE google_sub = $1")
        .bind(&info.sub)
        .fetch_optional(pool)
        .await?;

    if purpose == "link" {
        let user_id = link_user.ok_or(SignInError::Expired)?;
        if owner.is_some_and(|o| o != user_id) {
            return Err(SignInError::LinkedElsewhere);
        }
        save_identity(pool, user_id, &info.sub, &email).await?;
        return Ok(Finished::Linked { user_id, email });
    }

    // A returning Google user.
    if let Some(user_id) = owner {
        sqlx::query("UPDATE google_identities SET email = $2 WHERE user_id = $1").bind(user_id).bind(&email).execute(pool).await?;
        return Ok(Finished::SignedIn { user_id, email, is_new: false });
    }

    if !info.email_verified {
        return Err(SignInError::EmailNotVerified);
    }

    // Someone who signed up with a password using this same (verified) address: Google now signs
    // in to that account too.
    let by_email: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE lower(email) = $1")
        .bind(&email)
        .fetch_optional(pool)
        .await?;
    if let Some(user_id) = by_email {
        save_identity(pool, user_id, &info.sub, &email).await?;
        return Ok(Finished::SignedIn { user_id, email, is_new: false });
    }

    let first_name = info
        .given_name
        .clone()
        .or_else(|| info.name.as_deref().and_then(|n| n.split_whitespace().next()).map(str::to_string))
        .unwrap_or_else(|| email.split('@').next().unwrap_or("My").to_string());
    let user_id = create_account(pool, &email, &info.sub, info.name.as_deref(), &first_name, invite_code.as_deref()).await?;
    Ok(Finished::SignedIn { user_id, email, is_new: true })
}

async fn save_identity(pool: &PgPool, user_id: Uuid, sub: &str, email: &str) -> Result<(), SignInError> {
    sqlx::query(
        "INSERT INTO google_identities (user_id, google_sub, email) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id) DO UPDATE SET google_sub = EXCLUDED.google_sub, email = EXCLUDED.email",
    )
    .bind(user_id)
    .bind(sub)
    .bind(email)
    .execute(pool)
    .await?;
    Ok(())
}

/// A new account from a Google sign-in: the user, their email record (with a password nobody
/// knows, so password sign-in stays off until they set one), their own space or the invite's
/// organization, and their name.
async fn create_account(
    pool: &PgPool,
    email: &str,
    sub: &str,
    name: Option<&str>,
    first_name: &str,
    invite_code: Option<&str>,
) -> Result<Uuid, SignInError> {
    let unusable_password = {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        nomi_auth::password::hash_password(&hex::encode(bytes)).map_err(|e| SignInError::Other(e.to_string()))?
    };

    let mut tx = pool.begin().await?;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&mut *tx).await?;
    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash, password_login) VALUES ($1, $2, $3, false)")
        .bind(user_id)
        .bind(email)
        .bind(&unusable_password)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO google_identities (user_id, google_sub, email) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(sub)
        .bind(email)
        .execute(&mut *tx)
        .await?;

    match invite_code {
        Some(code) => {
            let invite: Option<(Uuid, String)> =
                sqlx::query_as("SELECT org_id, role FROM org_invites WHERE code = $1 AND used_at IS NULL AND expires_at > now()")
                    .bind(code)
                    .fetch_optional(&mut *tx)
                    .await?;
            let (org_id, role) = invite.ok_or(SignInError::InvalidInvite)?;
            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)").bind(org_id).bind(user_id).bind(&role).execute(&mut *tx).await?;
            sqlx::query("UPDATE org_invites SET used_at = now() WHERE code = $1").bind(code).execute(&mut *tx).await?;
        }
        None => {
            let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ($1, false) RETURNING id")
                .bind(format!("{first_name}'s space"))
                .fetch_one(&mut *tx)
                .await?;
            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')").bind(org_id).bind(user_id).execute(&mut *tx).await?;
        }
    }

    if let Some(name) = name.map(str::trim).filter(|n| !n.is_empty()) {
        sqlx::query("INSERT INTO user_profiles (user_id, display_name) VALUES ($1, $2) ON CONFLICT (user_id) DO NOTHING")
            .bind(user_id)
            .bind(name)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(user_id)
}

/// The user's sign-in methods, for Account settings.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct SignInMethods {
    pub password: bool,
    pub google_email: Option<String>,
}

pub async fn methods(pool: &PgPool, user_id: Uuid) -> Result<SignInMethods, sqlx::Error> {
    let password: bool = sqlx::query_scalar("SELECT password_login FROM web_credentials WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false);
    let google_email: Option<String> =
        sqlx::query_scalar("SELECT email FROM google_identities WHERE user_id = $1").bind(user_id).fetch_optional(pool).await?;
    Ok(SignInMethods { password, google_email })
}

/// Removes Google sign-in, unless it's the only way this user can sign in.
pub async fn unlink(pool: &PgPool, user_id: Uuid) -> Result<bool, SignInError> {
    let current = methods(pool, user_id).await?;
    if !current.password {
        return Ok(false);
    }
    sqlx::query("DELETE FROM google_identities WHERE user_id = $1").bind(user_id).execute(pool).await?;
    Ok(true)
}
