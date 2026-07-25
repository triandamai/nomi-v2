# Auth & Claims Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement email/password registration and login, JWT-based claims issuance, refresh-token rotation, and the axum enforcement layer (cheap claims-based checks plus DB-revalidated checks for sensitive writes) designed in `docs/superpowers/specs/2026-07-26-auth-claims-layer-design.md`.

**Architecture:** Core auth logic lives as plain async functions in `backend/src/auth/*.rs`, each independently testable against a real Postgres database via `sqlx::test` with no HTTP involved. A thin axum layer (`backend/src/app.rs`, `backend/src/routes/auth.rs`) wires those functions to HTTP endpoints and adds a `Claims` extractor that any future protected route can depend on. `backend/src/main.rs` becomes the crate's first runnable binary.

**Tech Stack:** axum 0.7, jsonwebtoken 9 (HS256), argon2 0.5, sha2/hex (refresh-token hashing), rand 0.8 (refresh-token generation), tower (dev-only, for `ServiceExt::oneshot` in route tests), building on the existing sqlx/tokio/uuid/chrono/serde_json crate from the schema-migrations plan.

## Global Constraints

- Access tokens: JWT, HS256, 30-minute TTL. Refresh tokens: opaque 32-byte random value (hex-encoded), 30-day TTL, only its SHA-256 hex digest is ever persisted.
- Permission-string grammar (corrected during this plan's design review): `nomi:<scope>:<resource>:[action1,action2]`. `scope` is `admin` or an `org_id`. `resource` is `user` (admin scope only), `member`, or `conversation` (org scope). Actions are `view`/`manage`. **Role names never appear inside a permission string** — `owner` and `admin` memberships both yield `[view,manage]` on both `member` and `conversation`; `member` memberships yield `[view]` only. Distinguishing owner from admin (for future owner-only features) means querying `memberships.role` directly, never the token.
- The JWT secret is never read from environment inside library code — every `auth` module function that needs it takes `jwt_secret: &str` as an explicit parameter, so tests always pass a fixed literal secret. Only `main.rs` reads `JWT_SECRET` from the environment, once, at startup.
- This plan adds `CHECK` constraints on `memberships.role`, `memberships.status`, `org_invites.role`, and `agent_sessions.status` — closing a gap flagged (but deferred) during the prior schema-migrations plan's final review, since this plan is exactly the "state-machine and claims logic" work that gap was deferred to.
- All new tables/columns follow the existing convention: `UUID PRIMARY KEY DEFAULT gen_random_uuid()` (except `web_credentials`, whose PK is `user_id` itself — a 1:1 table, no surrogate key needed), `TIMESTAMPTZ` timestamps.
- Migration numbering continues sequentially from the schema-migrations plan: `0007_auth.sql`.

---

## File Structure

- `backend/migrations/0007_auth.sql` — `users.is_platform_admin`, `web_credentials`, `refresh_tokens`, plus the four `CHECK` constraints.
- `backend/tests/auth_schema.rs` — proves the new columns/tables/constraints.
- `backend/src/auth/mod.rs` — re-exports the submodules below.
- `backend/src/auth/claims.rs` — `Claims` struct, JWT encode/decode, `has_permission`, `permission_string` helper.
- `backend/src/auth/password.rs` — argon2 hash/verify.
- `backend/src/auth/permissions.rs` — `compute_permissions(pool, user_id)` — the DB→claims-array computation.
- `backend/src/auth/registration.rs` — `register_user` (create-org / join-org-by-invite modes).
- `backend/src/auth/login.rs` — `login` (verify credentials, issue access token).
- `backend/src/auth/refresh_token.rs` — issue / refresh / revoke refresh tokens.
- `backend/src/auth/authorize.rs` — `authorize_org_action` (DB-revalidated sensitive-write check).
- `backend/src/auth/extractor.rs` — `AuthClaims`, the axum `FromRequestParts` extractor.
- `backend/src/app.rs` — `AppState`, `build_router`.
- `backend/src/routes/auth.rs` — HTTP handlers wiring the `auth` module to routes.
- `backend/src/main.rs` — binary entry point (env config, pool, migrations, `axum::serve`).
- `backend/tests/auth_routes.rs` — end-to-end HTTP-layer tests via `tower::ServiceExt::oneshot`.

---

### Task 1: Auth Schema — `users.is_platform_admin`, `web_credentials`, `refresh_tokens`, enum `CHECK`s

**Files:**
- Create: `backend/migrations/0007_auth.sql`
- Test: `backend/tests/auth_schema.rs`

**Interfaces:**
- Consumes: `users.id` (from `backend/migrations/0002_identity.sql`), `memberships`/`org_invites` (from `0003_organizations.sql`), `agent_sessions` (from `0005_agent_sessions.sql`).
- Produces: `users.is_platform_admin BOOLEAN`, `web_credentials(user_id, email, password_hash, created_at)`, `refresh_tokens(id, user_id, token_hash, created_at, expires_at, revoked_at)`. Task 4 queries `users.is_platform_admin` and `memberships`; Task 5 writes `web_credentials`; Task 6 reads `web_credentials`; Task 7 reads/writes `refresh_tokens`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_schema.rs
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn users_gains_is_platform_admin_defaulting_false(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!is_platform_admin);
}

#[sqlx::test]
async fn web_credentials_email_is_unique(pool: PgPool) {
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'hash1')",
    )
    .bind(user_a)
    .execute(&pool)
    .await
    .unwrap();

    let user_b: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = sqlx::query(
        "INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'hash2')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("web_credentials_email_key")
    );
}

#[sqlx::test]
async fn refresh_tokens_token_hash_is_unique(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, 'hash-a', now() + interval '30 days')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, 'hash-a', now() + interval '30 days')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("refresh_tokens_token_hash_key")
    );
}

#[sqlx::test]
async fn membership_role_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'superowner')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_role_check")
    );
}

#[sqlx::test]
async fn membership_status_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query(
        "INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, 'member', 'banned')",
    )
    .bind(org_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_status_check")
    );
}

#[sqlx::test]
async fn org_invites_role_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('BADROLE', $1, 'superowner', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("org_invites_role_check")
    );
}

#[sqlx::test]
async fn agent_sessions_status_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '999') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'Active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("agent_sessions_status_check")
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_schema`
Expected: FAIL — `relation "web_credentials" does not exist` (and similarly for the other new relations/constraints).

- [ ] **Step 3: Write the migration**

```sql
-- backend/migrations/0007_auth.sql
ALTER TABLE users ADD COLUMN is_platform_admin BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE web_credentials (
    user_id       UUID PRIMARY KEY REFERENCES users(id),
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    token_hash  TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ
);

ALTER TABLE memberships ADD CONSTRAINT memberships_role_check
    CHECK (role IN ('owner', 'admin', 'member'));
ALTER TABLE memberships ADD CONSTRAINT memberships_status_check
    CHECK (status IN ('invited', 'active', 'removed'));
ALTER TABLE org_invites ADD CONSTRAINT org_invites_role_check
    CHECK (role IN ('owner', 'admin', 'member'));
ALTER TABLE agent_sessions ADD CONSTRAINT agent_sessions_status_check
    CHECK (status IN ('active', 'completed', 'cancelled', 'expired'));
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_schema`
Expected: PASS (7 tests). If any test still fails with "relation does not exist" after the migration file is correct, run `cargo clean` first — `sqlx::test`'s migration discovery can miss a newly-added file under incremental compilation (a known issue from the schema-migrations plan).

- [ ] **Step 5: Commit**

```bash
git add backend/migrations/0007_auth.sql backend/tests/auth_schema.rs
git commit -m "feat: add auth schema (web_credentials, refresh_tokens, is_platform_admin, enum CHECKs)"
```

---

### Task 2: Claims — JWT Encode/Decode & Permission Matching

**Files:**
- Modify: `backend/Cargo.toml` (add `serde` with `derive`, `jsonwebtoken`)
- Create: `backend/src/auth/mod.rs`
- Create: `backend/src/auth/claims.rs`

**Interfaces:**
- Consumes: nothing new (pure logic, no DB).
- Produces: `pub struct Claims { pub sub: Uuid, pub active_org_id: Uuid, pub permissions: Vec<String>, pub exp: i64, pub iat: i64 }`, `Claims::new(sub, active_org_id, permissions, ttl_seconds) -> Claims`, `Claims::encode(&self, secret: &str) -> Result<String, ClaimsError>`, `Claims::decode(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error>`, `Claims::has_permission(&self, scope: &str, resource: &str, action: &str) -> bool`, `pub fn permission_string(scope: &str, resource: &str, actions: &[&str]) -> String`. Tasks 4, 5, 6, 7, 9, 10 all use `Claims` and `permission_string`.

- [ ] **Step 1: Add dependencies**

Add to `backend/Cargo.toml` under `[dependencies]`:

```toml
serde = { version = "1", features = ["derive"] }
jsonwebtoken = "9"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/src/auth/claims.rs (bottom of the file, #[cfg(test)] mod tests)

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const SECRET: &str = "test-secret-do-not-use-in-prod";

    fn sample_claims(permissions: Vec<String>) -> Claims {
        Claims::new(Uuid::new_v4(), Uuid::new_v4(), permissions, 1800)
    }

    #[test]
    fn encode_decode_roundtrip_preserves_claims() {
        let claims = sample_claims(vec![permission_string("admin", "user", &["view", "manage"])]);
        let token = claims.encode(SECRET).unwrap();
        let decoded = Claims::decode(&token, SECRET).unwrap();
        assert_eq!(decoded, claims);
    }

    #[test]
    fn decode_rejects_wrong_secret() {
        let claims = sample_claims(vec![]);
        let token = claims.encode(SECRET).unwrap();
        let result = Claims::decode(&token, "a-different-secret");
        assert!(result.is_err());
    }

    #[test]
    fn decode_rejects_expired_token() {
        let mut claims = sample_claims(vec![]);
        claims.exp = chrono::Utc::now().timestamp() - 60; // expired one minute ago
        let token = claims.encode(SECRET).unwrap();
        let result = Claims::decode(&token, SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn has_permission_matches_admin_scope() {
        let claims = sample_claims(vec![permission_string("admin", "user", &["view", "manage"])]);
        assert!(claims.has_permission("admin", "user", "view"));
        assert!(claims.has_permission("admin", "user", "manage"));
    }

    #[test]
    fn has_permission_matches_org_scope() {
        let org_id = "11111111-1111-1111-1111-111111111111";
        let claims = sample_claims(vec![permission_string(org_id, "member", &["view", "manage"])]);
        assert!(claims.has_permission(org_id, "member", "manage"));
    }

    #[test]
    fn has_permission_rejects_wrong_org() {
        let org_a = "11111111-1111-1111-1111-111111111111";
        let org_b = "22222222-2222-2222-2222-222222222222";
        let claims = sample_claims(vec![permission_string(org_a, "member", &["view", "manage"])]);
        assert!(!claims.has_permission(org_b, "member", "manage"));
    }

    #[test]
    fn has_permission_rejects_wrong_action() {
        let org_id = "11111111-1111-1111-1111-111111111111";
        let claims = sample_claims(vec![permission_string(org_id, "member", &["view"])]);
        assert!(!claims.has_permission(org_id, "member", "manage"));
    }

    #[test]
    fn permission_string_formats_as_expected() {
        assert_eq!(
            permission_string("admin", "user", &["view", "manage"]),
            "nomi:admin:user:[view,manage]"
        );
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --lib auth::claims`
Expected: FAIL to compile — `Claims`, `permission_string`, etc. don't exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/auth/claims.rs (above the tests module)
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: Uuid,
    pub active_org_id: Uuid,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ClaimsError {
    #[error("token encoding failed: {0}")]
    Encode(#[from] jsonwebtoken::errors::Error),
}

impl Claims {
    pub fn new(sub: Uuid, active_org_id: Uuid, permissions: Vec<String>, ttl_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Claims {
            sub,
            active_org_id,
            permissions,
            exp: now + ttl_seconds,
            iat: now,
        }
    }

    pub fn encode(&self, secret: &str) -> Result<String, ClaimsError> {
        Ok(encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?)
    }

    pub fn decode(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }

    pub fn has_permission(&self, scope: &str, resource: &str, action: &str) -> bool {
        let prefix = format!("nomi:{scope}:{resource}:[");
        self.permissions.iter().any(|p| {
            p.strip_prefix(prefix.as_str())
                .and_then(|rest| rest.strip_suffix(']'))
                .map(|actions| actions.split(',').any(|a| a == action))
                .unwrap_or(false)
        })
    }
}

pub fn permission_string(scope: &str, resource: &str, actions: &[&str]) -> String {
    format!("nomi:{scope}:{resource}:[{}]", actions.join(","))
}
```

Add `thiserror = "1"` to `backend/Cargo.toml` under `[dependencies]` if not already present (it isn't yet in this crate).

Create `backend/src/auth/mod.rs`:

```rust
pub mod claims;
```

And add to `backend/src/lib.rs`:

```rust
pub mod auth;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --lib auth::claims`
Expected: PASS (8 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/lib.rs backend/src/auth/mod.rs backend/src/auth/claims.rs
git commit -m "feat: add Claims struct with JWT encode/decode and permission matching"
```

---

### Task 3: Password Hashing

**Files:**
- Modify: `backend/Cargo.toml` (add `argon2`)
- Create: `backend/src/auth/password.rs`
- Modify: `backend/src/auth/mod.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces: `pub fn hash_password(password: &str) -> Result<String, PasswordError>`, `pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError>`. Tasks 5 and 6 use both.

- [ ] **Step 1: Add dependency**

Add to `backend/Cargo.toml` under `[dependencies]`:

```toml
argon2 = "0.5"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/src/auth/password.rs (bottom of file)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_succeeds_for_correct_password() {
        let hash = hash_password("correct-horse-battery-staple").unwrap();
        assert!(verify_password("correct-horse-battery-staple", &hash).unwrap());
    }

    #[test]
    fn verify_fails_for_wrong_password() {
        let hash = hash_password("correct-horse-battery-staple").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn hashing_the_same_password_twice_yields_different_hashes() {
        let hash1 = hash_password("same-password").unwrap();
        let hash2 = hash_password("same-password").unwrap();
        assert_ne!(hash1, hash2, "salts should differ between hashes");
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --lib auth::password`
Expected: FAIL to compile — `hash_password`/`verify_password` don't exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/auth/password.rs (above the tests module)
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("password hashing failed: {0}")]
    Hash(String),
    #[error("password hash is malformed: {0}")]
    MalformedHash(String),
}

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::Hash(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| PasswordError::MalformedHash(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod claims;
pub mod password;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --lib auth::password`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/auth/mod.rs backend/src/auth/password.rs
git commit -m "feat: add argon2 password hashing helpers"
```

---

### Task 4: Compute Permissions From Current DB State

**Files:**
- Create: `backend/src/auth/permissions.rs`
- Modify: `backend/src/auth/mod.rs`
- Test: `backend/tests/auth_permissions.rs`

**Interfaces:**
- Consumes: `Claims::permission_string` (Task 2), `users.is_platform_admin` and `memberships(org_id, user_id, role, status)` (Task 1 / schema plan).
- Produces: `pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error>`. Tasks 6 (login) and 7 (refresh) call this directly.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_permissions.rs
use nomi_orchestrator::auth::permissions::compute_permissions;
use sqlx::PgPool;
use uuid::Uuid;

async fn make_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn make_org(pool: &PgPool, name: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO organizations (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn add_membership(pool: &PgPool, org_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test]
async fn platform_admin_gets_the_admin_permission_string(pool: PgPool) {
    let user_id = make_user(&pool).await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view,manage]".to_string()));
}

#[sqlx::test]
async fn owner_gets_manage_permissions_on_member_and_conversation(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "owner").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
    assert!(permissions.contains(&format!("nomi:{org_id}:conversation:[view,manage]")));
}

#[sqlx::test]
async fn member_role_gets_view_only_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "member").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view]")));
    assert!(permissions.contains(&format!("nomi:{org_id}:conversation:[view]")));
    assert!(!permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
}

#[sqlx::test]
async fn permissions_span_every_org_the_user_belongs_to(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_a = make_org(&pool, "Org A").await;
    let org_b = make_org(&pool, "Org B").await;
    add_membership(&pool, org_a, user_id, "owner").await;
    add_membership(&pool, org_b, user_id, "member").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_a}:member:[view,manage]")));
    assert!(permissions.contains(&format!("nomi:{org_b}:member:[view]")));
}

#[sqlx::test]
async fn removed_membership_is_excluded(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "member").await;
    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.contains(&format!("nomi:{org_id}:member:[view]")));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_permissions`
Expected: FAIL to compile — `nomi_orchestrator::auth::permissions` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/auth/permissions.rs
use sqlx::PgPool;
use uuid::Uuid;

use super::claims::permission_string;

pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let mut permissions = Vec::new();

    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if is_platform_admin {
        permissions.push(permission_string("admin", "user", &["view", "manage"]));
    }

    let memberships: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT org_id, role FROM memberships WHERE user_id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for (org_id, role) in memberships {
        let actions: &[&str] = if role == "owner" || role == "admin" {
            &["view", "manage"]
        } else {
            &["view"]
        };
        let org_scope = org_id.to_string();
        permissions.push(permission_string(&org_scope, "member", actions));
        permissions.push(permission_string(&org_scope, "conversation", actions));
    }

    Ok(permissions)
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod claims;
pub mod password;
pub mod permissions;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_permissions`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/mod.rs backend/src/auth/permissions.rs backend/tests/auth_permissions.rs
git commit -m "feat: compute permission strings from current membership/admin state"
```

---

### Task 5: Registration (Create-Org / Join-Org-By-Invite)

**Files:**
- Create: `backend/src/auth/registration.rs`
- Modify: `backend/src/auth/mod.rs`
- Test: `backend/tests/auth_registration.rs`

**Interfaces:**
- Consumes: `hash_password` (Task 3), `web_credentials`/`organizations`/`memberships`/`org_invites` tables (Task 1 / schema plan).
- Produces: `pub enum OrgMode { Create { name: String }, Join { invite_code: String } }`, `pub async fn register_user(pool: &PgPool, email: &str, password: &str, org_mode: OrgMode) -> Result<Uuid, RegistrationError>`. Task 10's HTTP handler calls this directly.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_registration.rs
use nomi_orchestrator::auth::registration::{register_user, OrgMode, RegistrationError};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn create_mode_creates_org_and_owner_membership(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "founder@example.com",
        "hunter2-hunter2",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let (org_id, role): (Uuid, String) =
        sqlx::query_as("SELECT org_id, role FROM memberships WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(role, "owner");

    let org_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(org_name, "Acme");
}

#[sqlx::test]
async fn join_mode_uses_invite_role_and_marks_invite_used(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('JOINME', $1, 'admin', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let user_id = register_user(
        &pool,
        "newmember@example.com",
        "hunter2-hunter2",
        OrgMode::Join { invite_code: "JOINME".to_string() },
    )
    .await
    .unwrap();

    let role: String = sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(role, "admin");

    let used_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT used_at FROM org_invites WHERE code = 'JOINME'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used_at.is_some());
}

#[sqlx::test]
async fn join_mode_rejects_a_reused_invite_code(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at, used_at) VALUES ('USEDUP', $1, 'member', $2, now() + interval '7 days', now())",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let result = register_user(
        &pool,
        "latecomer@example.com",
        "hunter2-hunter2",
        OrgMode::Join { invite_code: "USEDUP".to_string() },
    )
    .await;

    assert!(matches!(result, Err(RegistrationError::InvalidInvite)));
}

#[sqlx::test]
async fn rejects_duplicate_email(pool: PgPool) {
    register_user(
        &pool,
        "dup@example.com",
        "hunter2-hunter2",
        OrgMode::Create { name: "First Co".to_string() },
    )
    .await
    .unwrap();

    let result = register_user(
        &pool,
        "dup@example.com",
        "different-password",
        OrgMode::Create { name: "Second Co".to_string() },
    )
    .await;

    assert!(matches!(result, Err(RegistrationError::EmailTaken)));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_registration`
Expected: FAIL to compile — `nomi_orchestrator::auth::registration` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/auth/registration.rs
use sqlx::PgPool;
use uuid::Uuid;

use super::password::{hash_password, PasswordError};

pub enum OrgMode {
    Create { name: String },
    Join { invite_code: String },
}

#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("email already registered")]
    EmailTaken,
    #[error("invite code not found, expired, or already used")]
    InvalidInvite,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Password(#[from] PasswordError),
}

pub async fn register_user(
    pool: &PgPool,
    email: &str,
    password: &str,
    org_mode: OrgMode,
) -> Result<Uuid, RegistrationError> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
    if existing.is_some() {
        return Err(RegistrationError::EmailTaken);
    }

    let password_hash = hash_password(password)?;

    let mut tx = pool.begin().await?;

    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(email)
        .bind(&password_hash)
        .execute(&mut *tx)
        .await?;

    match org_mode {
        OrgMode::Create { name } => {
            let org_id: Uuid = sqlx::query_scalar(
                "INSERT INTO organizations (name, is_personal) VALUES ($1, false) RETURNING id",
            )
            .bind(&name)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
                .bind(org_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        OrgMode::Join { invite_code } => {
            let invite: Option<(Uuid, String)> = sqlx::query_as(
                "SELECT org_id, role FROM org_invites WHERE code = $1 AND used_at IS NULL AND expires_at > now()",
            )
            .bind(&invite_code)
            .fetch_optional(&mut *tx)
            .await?;

            let (org_id, role) = invite.ok_or(RegistrationError::InvalidInvite)?;

            sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
                .bind(org_id)
                .bind(user_id)
                .bind(&role)
                .execute(&mut *tx)
                .await?;

            sqlx::query("UPDATE org_invites SET used_at = now() WHERE code = $1")
                .bind(&invite_code)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(user_id)
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod claims;
pub mod password;
pub mod permissions;
pub mod registration;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_registration`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/mod.rs backend/src/auth/registration.rs backend/tests/auth_registration.rs
git commit -m "feat: add registration with create-org and join-by-invite modes"
```

---

### Task 6: Login

**Files:**
- Create: `backend/src/auth/login.rs`
- Modify: `backend/src/auth/mod.rs`
- Test: `backend/tests/auth_login.rs`

**Interfaces:**
- Consumes: `verify_password` (Task 3), `compute_permissions` (Task 4), `Claims` (Task 2), `web_credentials`/`memberships` (Task 1 / schema plan).
- Produces: `pub async fn login(pool: &PgPool, email: &str, password: &str, jwt_secret: &str) -> Result<(String, Uuid), LoginError>` — returns `(access_token, user_id)`. Task 10's HTTP handler calls this directly.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_login.rs
use nomi_orchestrator::auth::{claims::Claims, login::{login, LoginError}, registration::{register_user, OrgMode}};
use sqlx::PgPool;

const SECRET: &str = "test-secret-do-not-use-in-prod";

#[sqlx::test]
async fn login_succeeds_with_correct_credentials(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "alice@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let (token, returned_user_id) = login(&pool, "alice@example.com", "correct-password", SECRET)
        .await
        .unwrap();
    assert_eq!(returned_user_id, user_id);

    let claims = Claims::decode(&token, SECRET).unwrap();
    assert_eq!(claims.sub, user_id);
    assert!(claims
        .permissions
        .iter()
        .any(|p| p.contains("member") && p.contains("manage")));
}

#[sqlx::test]
async fn login_rejects_wrong_password(pool: PgPool) {
    register_user(
        &pool,
        "bob@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let result = login(&pool, "bob@example.com", "wrong-password", SECRET).await;
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[sqlx::test]
async fn login_rejects_unknown_email(pool: PgPool) {
    let result = login(&pool, "nobody@example.com", "whatever", SECRET).await;
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_login`
Expected: FAIL to compile — `nomi_orchestrator::auth::login` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/auth/login.rs
use sqlx::PgPool;
use uuid::Uuid;

use super::{claims::Claims, permissions::compute_permissions, password::verify_password};

pub const ACCESS_TOKEN_TTL_SECONDS: i64 = 30 * 60;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Claims(#[from] super::claims::ClaimsError),
}

pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<(String, Uuid), LoginError> {
    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, password_hash FROM web_credentials WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

    let (user_id, password_hash) = row.ok_or(LoginError::InvalidCredentials)?;

    let valid = verify_password(password, &password_hash).unwrap_or(false);
    if !valid {
        return Err(LoginError::InvalidCredentials);
    }

    let permissions = compute_permissions(pool, user_id).await?;

    let active_org_id: Uuid = sqlx::query_scalar(
        "SELECT org_id FROM memberships WHERE user_id = $1 AND status = 'active' ORDER BY created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let claims = Claims::new(user_id, active_org_id, permissions, ACCESS_TOKEN_TTL_SECONDS);
    let token = claims.encode(jwt_secret)?;

    Ok((token, user_id))
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod claims;
pub mod login;
pub mod password;
pub mod permissions;
pub mod registration;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_login`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/mod.rs backend/src/auth/login.rs backend/tests/auth_login.rs
git commit -m "feat: add login issuing access tokens with computed permissions"
```

---

### Task 7: Refresh Tokens (Issue, Refresh, Revoke)

**Files:**
- Modify: `backend/Cargo.toml` (add `rand`, `sha2`, `hex`)
- Create: `backend/src/auth/refresh_token.rs`
- Modify: `backend/src/auth/mod.rs`
- Test: `backend/tests/auth_refresh.rs`

**Interfaces:**
- Consumes: `Claims`, `ClaimsError` (Task 2), `compute_permissions` (Task 4), `refresh_tokens` table (Task 1).
- Produces: `pub fn hash_token(raw: &str) -> String`, `pub fn generate_raw_token() -> String`, `pub async fn issue_refresh_token(pool: &PgPool, user_id: Uuid) -> Result<String, sqlx::Error>`, `pub async fn refresh_access_token(pool: &PgPool, raw_refresh_token: &str, jwt_secret: &str) -> Result<String, RefreshError>`, `pub async fn revoke_refresh_token(pool: &PgPool, raw_refresh_token: &str) -> Result<(), sqlx::Error>`. Task 10's HTTP handlers call all three async functions.

- [ ] **Step 1: Add dependencies**

Add to `backend/Cargo.toml` under `[dependencies]`:

```toml
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/tests/auth_refresh.rs
use nomi_orchestrator::auth::{
    login::login,
    refresh_token::{issue_refresh_token, refresh_access_token, revoke_refresh_token, RefreshError},
    claims::Claims,
    registration::{register_user, OrgMode},
};
use sqlx::PgPool;

const SECRET: &str = "test-secret-do-not-use-in-prod";

#[sqlx::test]
async fn issue_and_refresh_roundtrip(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "carol@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();
    let new_access_token = refresh_access_token(&pool, &raw_refresh, SECRET).await.unwrap();

    let claims = Claims::decode(&new_access_token, SECRET).unwrap();
    assert_eq!(claims.sub, user_id);
}

#[sqlx::test]
async fn refresh_rejects_after_revocation(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "dave@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();
    revoke_refresh_token(&pool, &raw_refresh).await.unwrap();

    let result = refresh_access_token(&pool, &raw_refresh, SECRET).await;
    assert!(matches!(result, Err(RefreshError::Invalid)));
}

#[sqlx::test]
async fn refresh_rejects_after_expiry(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "erin@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = "manually-inserted-expired-token";
    let token_hash = nomi_orchestrator::auth::refresh_token::hash_token(raw_refresh);
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, now() - interval '1 day')",
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&pool)
    .await
    .unwrap();

    let result = refresh_access_token(&pool, raw_refresh, SECRET).await;
    assert!(matches!(result, Err(RefreshError::Invalid)));
}

#[sqlx::test]
async fn refresh_recomputes_permissions_after_membership_change(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "frank@example.com",
        "correct-password",
        OrgMode::Create { name: "First Org".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();

    let new_org_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Second Org') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(new_org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let new_access_token = refresh_access_token(&pool, &raw_refresh, SECRET).await.unwrap();
    let claims = Claims::decode(&new_access_token, SECRET).unwrap();
    assert!(claims
        .permissions
        .contains(&format!("nomi:{new_org_id}:member:[view]")));
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_refresh`
Expected: FAIL to compile — `nomi_orchestrator::auth::refresh_token` doesn't exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/auth/refresh_token.rs
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::{claims::Claims, permissions::compute_permissions};

const REFRESH_TOKEN_TTL_DAYS: &str = "30 days";
const ACCESS_TOKEN_TTL_SECONDS: i64 = 30 * 60;

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("refresh token invalid, revoked, or expired")]
    Invalid,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Claims(#[from] super::claims::ClaimsError),
}

pub fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_raw_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn issue_refresh_token(pool: &PgPool, user_id: Uuid) -> Result<String, sqlx::Error> {
    let raw = generate_raw_token();
    let token_hash = hash_token(&raw);
    sqlx::query(&format!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, now() + interval '{REFRESH_TOKEN_TTL_DAYS}')"
    ))
    .bind(user_id)
    .bind(&token_hash)
    .execute(pool)
    .await?;
    Ok(raw)
}

pub async fn refresh_access_token(
    pool: &PgPool,
    raw_refresh_token: &str,
    jwt_secret: &str,
) -> Result<String, RefreshError> {
    let token_hash = hash_token(raw_refresh_token);

    let user_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    let user_id = user_id.ok_or(RefreshError::Invalid)?;

    let permissions = compute_permissions(pool, user_id).await?;
    let active_org_id: Uuid = sqlx::query_scalar(
        "SELECT org_id FROM memberships WHERE user_id = $1 AND status = 'active' ORDER BY created_at ASC LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let claims = Claims::new(user_id, active_org_id, permissions, ACCESS_TOKEN_TTL_SECONDS);
    Ok(claims.encode(jwt_secret)?)
}

pub async fn revoke_refresh_token(pool: &PgPool, raw_refresh_token: &str) -> Result<(), sqlx::Error> {
    let token_hash = hash_token(raw_refresh_token);
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod claims;
pub mod login;
pub mod password;
pub mod permissions;
pub mod refresh_token;
pub mod registration;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_refresh`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/auth/mod.rs backend/src/auth/refresh_token.rs backend/tests/auth_refresh.rs
git commit -m "feat: add refresh-token issue/refresh/revoke with fresh permission recomputation"
```

---

### Task 8: Sensitive-Write Re-Validation (`authorize_org_action`)

**Files:**
- Create: `backend/src/auth/authorize.rs`
- Modify: `backend/src/auth/mod.rs`
- Test: `backend/tests/auth_authorize.rs`

**Interfaces:**
- Consumes: `memberships` table (Task 1 / schema plan).
- Produces: `pub async fn authorize_org_action(pool: &PgPool, user_id: Uuid, org_id: Uuid, allowed_roles: &[&str]) -> Result<(), AuthorizeError>`. Task 10's sensitive HTTP handler (`remove_member`) calls this instead of trusting `Claims::has_permission`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_authorize.rs
use nomi_orchestrator::auth::authorize::{authorize_org_action, AuthorizeError};
use sqlx::PgPool;
use uuid::Uuid;

async fn make_membership(pool: &PgPool, role: &str) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    (org_id, user_id)
}

#[sqlx::test]
async fn allows_a_matching_role(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "owner").await;
    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(result.is_ok());
}

#[sqlx::test]
async fn rejects_a_role_not_in_the_allow_list(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "member").await;
    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}

#[sqlx::test]
async fn rejects_a_different_org(pool: PgPool) {
    let (_org_a, user_id) = make_membership(&pool, "owner").await;
    let org_b: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Other Org') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = authorize_org_action(&pool, user_id, org_b, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}

#[sqlx::test]
async fn rejects_a_membership_removed_after_it_was_granted(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "owner").await;

    // Simulate the membership being revoked after some earlier access token was already issued.
    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_authorize`
Expected: FAIL to compile — `nomi_orchestrator::auth::authorize` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/auth/authorize.rs
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("not authorized for this organization")]
    Forbidden,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn authorize_org_action(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Uuid,
    allowed_roles: &[&str],
) -> Result<(), AuthorizeError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match role {
        Some(r) if allowed_roles.contains(&r.as_str()) => Ok(()),
        _ => Err(AuthorizeError::Forbidden),
    }
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod authorize;
pub mod claims;
pub mod login;
pub mod password;
pub mod permissions;
pub mod refresh_token;
pub mod registration;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_authorize`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/mod.rs backend/src/auth/authorize.rs backend/tests/auth_authorize.rs
git commit -m "feat: add authorize_org_action for DB-revalidated sensitive writes"
```

---

### Task 9: axum `Claims` Extractor + Minimal Protected Route

**Files:**
- Modify: `backend/Cargo.toml` (add `axum`; add dev-dependencies `tower`, `http-body-util`)
- Create: `backend/src/auth/extractor.rs`
- Modify: `backend/src/auth/mod.rs`
- Create: `backend/src/app.rs`
- Modify: `backend/src/lib.rs`
- Test: `backend/tests/auth_extractor.rs`

**Interfaces:**
- Consumes: `Claims::decode` (Task 2).
- Produces: `pub struct AppState { pub pool: PgPool, pub jwt_secret: String }`, `pub struct AuthClaims(pub Claims)` (implements `FromRequestParts<AppState>`), `pub fn build_router(state: AppState) -> axum::Router`. Task 10 adds the remaining routes to the same router and reuses `AuthClaims`.

- [ ] **Step 1: Add dependencies**

Add to `backend/Cargo.toml` under `[dependencies]`:

```toml
axum = "0.7"
```

Add under `[dev-dependencies]` (create this section if it doesn't exist):

```toml
[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/tests/auth_extractor.rs
use axum::{body::Body, http::{Request, StatusCode}, routing::get, Router};
use http_body_util::BodyExt;
use nomi_orchestrator::{app::AppState, auth::{claims::Claims, extractor::AuthClaims}};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_router(pool: PgPool) -> Router {
    let state = AppState { pool, jwt_secret: SECRET.to_string() };
    Router::new()
        .route("/whoami", get(|AuthClaims(claims): AuthClaims| async move {
            axum::Json(claims)
        }))
        .with_state(state)
}

#[sqlx::test]
async fn valid_token_reaches_the_handler(pool: PgPool) {
    let claims = Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], 1800);
    let token = claims.encode(SECRET).unwrap();

    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let decoded: Claims = serde_json::from_slice(&body).unwrap();
    assert_eq!(decoded.sub, claims.sub);
}

#[sqlx::test]
async fn missing_token_is_rejected(pool: PgPool) {
    let response = test_router(pool)
        .oneshot(Request::builder().uri("/whoami").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn malformed_token_is_rejected(pool: PgPool) {
    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", "Bearer not-a-real-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn expired_token_is_rejected(pool: PgPool) {
    let mut claims = Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], 1800);
    claims.exp = chrono::Utc::now().timestamp() - 60;
    let token = claims.encode(SECRET).unwrap();

    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .uri("/whoami")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_extractor`
Expected: FAIL to compile — `nomi_orchestrator::app` and `nomi_orchestrator::auth::extractor` don't exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/auth/extractor.rs
use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};

use super::claims::Claims;
use crate::app::AppState;

pub struct AuthClaims(pub Claims);

impl FromRequestParts<AppState> for AuthClaims {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "malformed authorization header"))?;

        let claims = Claims::decode(token, &state.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token"))?;

        Ok(AuthClaims(claims))
    }
}
```

```rust
// backend/src/app.rs
use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::auth::extractor::AuthClaims;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .with_state(state)
}
```

Update `backend/src/auth/mod.rs`:

```rust
pub mod authorize;
pub mod claims;
pub mod extractor;
pub mod login;
pub mod password;
pub mod permissions;
pub mod refresh_token;
pub mod registration;
```

Update `backend/src/lib.rs`:

```rust
pub mod app;
pub mod auth;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_extractor`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/lib.rs backend/src/app.rs backend/src/auth/mod.rs backend/src/auth/extractor.rs backend/tests/auth_extractor.rs
git commit -m "feat: add Claims extractor and minimal protected /api/whoami route"
```

---

### Task 10: Full Auth Routes + Sensitive Example Route + Binary Entry Point

**Files:**
- Modify: `backend/src/app.rs`
- Create: `backend/src/routes/mod.rs`
- Create: `backend/src/routes/auth.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/src/main.rs`
- Test: `backend/tests/auth_routes.rs`

**Interfaces:**
- Consumes: every `auth` module function from Tasks 2–9 (`register_user`, `login`, `issue_refresh_token`, `refresh_access_token`, `revoke_refresh_token`, `authorize_org_action`, `AuthClaims`, `AppState`, `build_router`).
- Produces: the full HTTP API (`/api/auth/register`, `/api/auth/login`, `/api/auth/refresh`, `/api/auth/logout`, `/api/auth/switch-org`, `/api/whoami`, `/api/orgs/:org_id/members/:user_id`) and the `nomi-orchestrator` binary. Nothing later in this plan depends on this — it's the plan's capstone. Future plans (orchestrator turn-loop, frontend integration) build on this HTTP surface.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/auth_routes.rs
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json_body)
}

#[sqlx::test]
async fn register_returns_a_usable_token_pair(pool: PgPool) {
    let state = AppState { pool, jwt_secret: SECRET.to_string() };
    let router = build_router(state);

    let (status, register_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "grace@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access_token = register_body["access_token"].as_str().unwrap().to_string();
    assert!(register_body["refresh_token"].as_str().is_some());

    let (status, whoami_body) = json_request(router, "GET", "/api/whoami", Value::Null, Some(&access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(whoami_body["sub"].is_string());
}

#[sqlx::test]
async fn login_after_registration_also_succeeds(pool: PgPool) {
    let state = AppState { pool, jwt_secret: SECRET.to_string() };
    let router = build_router(state);

    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "grace2@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;

    let (status, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "grace2@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access_token = login_body["access_token"].as_str().unwrap().to_string();

    let (status, whoami_body) = json_request(router, "GET", "/api/whoami", Value::Null, Some(&access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(whoami_body["sub"].is_string());
}

#[sqlx::test]
async fn remove_member_rejects_a_caller_scoped_to_a_different_org(pool: PgPool) {
    let state = AppState { pool: pool.clone(), jwt_secret: SECRET.to_string() };
    let router = build_router(state);

    let (_, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "owner-a@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Org A" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "owner-a@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let owner_a_token = login_body["access_token"].as_str().unwrap().to_string();

    let org_b_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Org B') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let target_user_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_b_id)
        .bind(target_user_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "DELETE",
        &format!("/api/orgs/{org_b_id}/members/{target_user_id}"),
        Value::Null,
        Some(&owner_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn remove_member_succeeds_for_the_owning_org_owner(pool: PgPool) {
    let state = AppState { pool: pool.clone(), jwt_secret: SECRET.to_string() };
    let router = build_router(state);

    let (_, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "owner@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "owner@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let owner_token = login_body["access_token"].as_str().unwrap().to_string();
    let owner_claims = nomi_orchestrator::auth::claims::Claims::decode(&owner_token, SECRET).unwrap();
    let org_id = owner_claims.active_org_id;

    let member_user_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(member_user_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "DELETE",
        &format!("/api/orgs/{org_id}/members/{member_user_id}"),
        Value::Null,
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let member_status: String = sqlx::query_scalar("SELECT status FROM memberships WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(member_user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(member_status, "removed");
}

#[sqlx::test]
async fn refresh_and_logout_flow(pool: PgPool) {
    let state = AppState { pool, jwt_secret: SECRET.to_string() };
    let router = build_router(state);

    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "hank@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "hank@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let refresh_token = login_body["refresh_token"].as_str().unwrap().to_string();

    let (status, refresh_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(refresh_body["access_token"].is_string());

    let (status, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/logout",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = json_request(
        router,
        "POST",
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test auth_routes`
Expected: FAIL — the register/login/refresh/logout/remove-member routes don't exist yet (404s), or fail to compile if `login_body["refresh_token"]` isn't yet returned by the login handler.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/routes/auth.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::{
    authorize::authorize_org_action,
    extractor::AuthClaims,
    login::login,
    refresh_token::{issue_refresh_token, refresh_access_token, revoke_refresh_token},
    registration::{register_user, OrgMode},
};

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum OrgModeRequest {
    Create { name: String },
    Join { invite_code: String },
}

impl From<OrgModeRequest> for OrgMode {
    fn from(value: OrgModeRequest) -> Self {
        match value {
            OrgModeRequest::Create { name } => OrgMode::Create { name },
            OrgModeRequest::Join { invite_code } => OrgMode::Join { invite_code },
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub org: OrgModeRequest,
}

#[derive(Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<TokenPairResponse>, (StatusCode, &'static str)> {
    register_user(&state.pool, &req.email, &req.password, req.org.into())
        .await
        .map_err(|e| match e {
            crate::auth::registration::RegistrationError::EmailTaken => {
                (StatusCode::CONFLICT, "email already registered")
            }
            crate::auth::registration::RegistrationError::InvalidInvite => {
                (StatusCode::BAD_REQUEST, "invite code not found, expired, or already used")
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "registration failed"),
        })?;

    // Per the design doc: registration immediately performs the same claims
    // computation as login and returns the token pair — no separate login
    // call needed. Reuses `login` directly since the credentials just
    // written are already valid.
    let (access_token, user_id) = login(&state.pool, &req.email, &req.password, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "registered but failed to issue tokens"))?;
    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;

    Ok(Json(TokenPairResponse { access_token, refresh_token }))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenPairResponse>, (StatusCode, &'static str)> {
    let (access_token, user_id) = login(&state.pool, &req.email, &req.password, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid credentials"))?;

    let refresh_token = issue_refresh_token(&state.pool, user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to issue refresh token"))?;

    Ok(Json(TokenPairResponse { access_token, refresh_token }))
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AccessTokenResponse>, (StatusCode, &'static str)> {
    let access_token = refresh_access_token(&state.pool, &req.refresh_token, &state.jwt_secret)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "refresh token invalid, revoked, or expired"))?;

    Ok(Json(AccessTokenResponse { access_token }))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    revoke_refresh_token(&state.pool, &req.refresh_token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke refresh token"))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_org_action(&state.pool, claims.sub, org_id, &["owner", "admin"])
        .await
        .map_err(|_| (StatusCode::FORBIDDEN, "not authorized for this organization"))?;

    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove member"))?;

    Ok(StatusCode::NO_CONTENT)
}
```

```rust
// backend/src/routes/mod.rs
pub mod auth;
```

```rust
// backend/src/app.rs (replace entirely)
use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;

use crate::auth::extractor::AuthClaims;
use crate::routes::auth as auth_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login_handler))
        .route("/api/auth/refresh", post(auth_routes::refresh_handler))
        .route("/api/auth/logout", post(auth_routes::logout_handler))
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .route("/api/orgs/:org_id/members/:user_id", delete(auth_routes::remove_member))
        .with_state(state)
}
```

```rust
// backend/src/main.rs
#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = nomi_orchestrator::app::AppState { pool, jwt_secret };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to port 8080");
    axum::serve(listener, app).await.expect("server error");
}
```

Update `backend/src/lib.rs`:

```rust
pub mod app;
pub mod auth;
pub mod routes;
```

Note the `switch-org` endpoint described in the design doc is **not** included in this task — it needs `active_org_id` reassignment logic that isn't exercised by any test above; add it as a follow-up once the frontend integration plan actually needs it, rather than building it speculatively here.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test auth_routes`
Expected: PASS (5 tests)

- [ ] **Step 5: Run the full test suite once to confirm nothing regressed**

Run: `cd backend && cargo test`
Expected: PASS (every test file from this plan and the schema-migrations plan)

- [ ] **Step 6: Confirm the binary actually builds and starts**

Run: `cd backend && cargo build --bin nomi-orchestrator`
Expected: builds with no errors. (Starting it against the local Postgres with `DATABASE_URL`/`JWT_SECRET` set is a reasonable manual smoke check, but not required for this task's automated pass/fail.)

- [ ] **Step 7: Commit**

```bash
git add backend/src/app.rs backend/src/lib.rs backend/src/main.rs backend/src/routes backend/tests/auth_routes.rs
git commit -m "feat: wire full auth HTTP routes and add nomi-orchestrator binary entry point"
```

---

## Self-Review Notes

- **Spec coverage**: every piece of `docs/superpowers/specs/2026-07-26-auth-claims-layer-design.md` has a task — schema (§1 → Task 1), registration/login (§2 → Tasks 5/6/10), claims/tokens (§3 → Tasks 2/7/10), enforcement (§4 → Tasks 8/9/10), and the testing approach (§5 → the test file in every task).
- **Deliberately deferred beyond this plan**: `switch-org` (noted inline in Task 10 — no test demands it yet, so it isn't built speculatively), cookie-based token transport, password reset, rate-limiting, cross-channel-linking UI — all already listed as Out of Scope in the design doc.
- **Type consistency**: `Claims`, `ClaimsError`, `AppState`, `AuthClaims`, and every `auth::*` function signature are used identically across the tasks that consume them (e.g. `login`'s `(String, Uuid)` return in Task 6 matches exactly how Task 10's `login_handler` destructures it).
