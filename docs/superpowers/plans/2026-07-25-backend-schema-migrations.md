# Backend Schema & Migrations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the Postgres+pgvector schema (identity, organizations/RBAC, sessions/messages, sub-agent lifecycle, memory/audit) that every later backend and frontend task depends on, with each table's key invariants proven by an integration test against a real database.

**Architecture:** A single Rust crate (`nomi-orchestrator`) with no application logic yet — this plan only produces `migrations/*.sql` files plus `tests/*.rs` integration tests that run those migrations against an ephemeral Postgres database per test (via `sqlx::test`) and assert the constraints from the four approved design docs actually hold (uniqueness, foreign keys, the partial-unique-index sub-agent invariant, vector column shape).

**Tech Stack:** Rust, `sqlx` 0.7 (Postgres driver + migration runner + `sqlx::test`), Postgres 16 with the `pgvector` extension, Docker Compose for local Postgres, `sqlx-cli` for applying migrations outside of tests.

## Global Constraints

- Postgres image: `pgvector/pgvector:pg16` (bundles the `vector` extension; `pgcrypto` for `gen_random_uuid()` is enabled explicitly in migration 0001).
- Migrations are plain (non-reversible) `.sql` files, sequentially numbered (`0001_...`, `0002_...`, ...) rather than timestamp-prefixed, so the ordering in this plan is exact and reviewable.
- All primary keys are `UUID DEFAULT gen_random_uuid()`; all timestamps are `TIMESTAMPTZ`.
- `memory_items.embedding` is `VECTOR(1536)`, matching OpenAI's `text-embedding-3-small` — this was flagged as an open decision in `plans/initial.md` §5; 1536 is the concrete default for this plan and is the plan's own decision, not something the spec pinned down. Changing embedding models later means a follow-up migration (drop/recreate the column and its index), not a schema design flaw.
- Every table/column name below is taken verbatim from the four approved specs (`plans/initial.md`, `2026-07-22-sub-agent-lifecycle-design.md`, `2026-07-22-cross-channel-identity-design.md`, `2026-07-25-multi-tenant-rbac-design.md`) where they were specified there, and is a plan-level concrete decision (documented inline) where a spec only described a table narratively without exact DDL (`messages`, `memory_items`, `agent_events`).
- `sqlx::test` provisions one throwaway database per test by connecting to `DATABASE_URL` and running every migration in `./migrations` fresh — no manual test-database setup is needed.

---

## File Structure

- `Cargo.toml` — crate manifest, `sqlx`/`tokio`/`uuid`/`chrono` dependencies.
- `.cargo/config.toml` — sets `DATABASE_URL` for `cargo test`/`cargo build` against the local Docker Postgres, so `sqlx::test` and any future `sqlx::query!` compile-time checks work without manual env exports.
- `docker-compose.yml` — local Postgres 16 + pgvector.
- `.gitignore` — add `/target`.
- `migrations/0001_extensions.sql` — `pgcrypto`, `vector`.
- `migrations/0002_identity.sql` — `users`, `channel_identities`, `link_codes` (cross-channel identity design).
- `migrations/0003_organizations.sql` — `organizations`, `memberships`, `org_invites` (multi-tenant RBAC design).
- `migrations/0004_sessions.sql` — `sessions`, `messages`, `session_participants` (cross-channel identity design + RBAC design's `org_id`/`session_participants` amendments, folded into final form).
- `migrations/0005_agent_sessions.sql` — `agent_sessions` with the per-speaker partial unique index (sub-agent lifecycle design, as amended by the cross-channel identity design).
- `migrations/0006_memory_and_events.sql` — `memory_items`, `agent_events` (parent plan's "brain," given concrete DDL here for the first time).
- `src/lib.rs` — empty crate root (just enough for `cargo test` to have something to compile); real orchestrator code arrives in a later plan.
- `tests/extensions.rs`, `tests/identity.rs`, `tests/organizations.rs`, `tests/sessions.rs`, `tests/agent_sessions.rs`, `tests/memory_and_events.rs` — one integration-test file per migration, each a separate test binary.

---

### Task 1: Project Scaffold & Extensions Migration

**Files:**
- Create: `Cargo.toml`
- Create: `.cargo/config.toml`
- Create: `docker-compose.yml`
- Create: `src/lib.rs`
- Create: `migrations/0001_extensions.sql`
- Test: `tests/extensions.rs`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: a running local Postgres reachable at `postgres://nomi:nomi@localhost:5432/nomi`, with `pgcrypto` and `vector` extensions enabled — every later task's tests depend on this connection and on `gen_random_uuid()`/`VECTOR` being available.

- [ ] **Step 1: Check the working directory is clean before scaffolding**

Run: `ls -la` and `git status`
Expected: no `Cargo.toml`, `migrations/`, or `docker-compose.yml` already present (this repo is docs-only so far).

- [ ] **Step 2: Write `docker-compose.yml`**

```yaml
services:
  postgres:
    image: pgvector/pgvector:pg16
    environment:
      POSTGRES_USER: nomi
      POSTGRES_PASSWORD: nomi
      POSTGRES_DB: nomi
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

- [ ] **Step 3: Start Postgres and confirm it's healthy**

Run: `docker compose up -d` then `docker compose exec postgres pg_isready -U nomi`
Expected: `accepting connections`

- [ ] **Step 4: Write `Cargo.toml`**

```toml
[package]
name = "nomi-orchestrator"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate", "macros"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 5: Write `src/lib.rs`**

```rust
// Schema-only crate for now; orchestrator logic arrives in a later plan.
```

- [ ] **Step 6: Write `.cargo/config.toml` so `cargo test` can reach Postgres without manual env exports**

```toml
[env]
DATABASE_URL = "postgres://nomi:nomi@localhost:5432/nomi"
```

- [ ] **Step 7: Add `/target` to `.gitignore`**

Append this line to the existing `.gitignore` (which currently only has `.superpowers/`):

```
/target
```

- [ ] **Step 8: Write the failing test**

```rust
// tests/extensions.rs
use sqlx::PgPool;

#[sqlx::test]
async fn required_extensions_are_enabled(pool: PgPool) {
    let exts: Vec<String> = sqlx::query_scalar(
        "SELECT extname FROM pg_extension WHERE extname IN ('pgcrypto', 'vector') ORDER BY extname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(exts, vec!["pgcrypto".to_string(), "vector".to_string()]);
}
```

- [ ] **Step 9: Run the test to verify it fails**

Run: `cargo test --test extensions`
Expected: FAIL — the assertion fails because `migrations/` doesn't exist yet, so neither extension is enabled (query returns an empty vec).

- [ ] **Step 10: Write the migration**

```sql
-- migrations/0001_extensions.sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS vector;
```

- [ ] **Step 11: Run the test to verify it passes**

Run: `cargo test --test extensions`
Expected: PASS

- [ ] **Step 12: Commit**

```bash
git add Cargo.toml .cargo/config.toml docker-compose.yml src/lib.rs .gitignore migrations/0001_extensions.sql tests/extensions.rs Cargo.lock
git commit -m "chore: scaffold nomi-orchestrator crate, enable pgcrypto/vector extensions"
```

---

### Task 2: Identity Schema — `users`, `channel_identities`, `link_codes`

**Files:**
- Create: `migrations/0002_identity.sql`
- Test: `tests/identity.rs`

**Interfaces:**
- Consumes: `pgcrypto`'s `gen_random_uuid()` (Task 1).
- Produces: `users(id)`, `channel_identities(id, user_id, channel, channel_user_id)` with `UNIQUE (channel, channel_user_id)`, `link_codes(code, user_id, expires_at, used_at)`. Tasks 3–6 all reference `users.id`; Tasks 4–6 reference `channel_identities.id`.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/identity.rs
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn channel_identity_is_unique_per_channel_and_channel_user_id(pool: PgPool) {
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '12345')",
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
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '12345')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap_err();

    let db_err = err.as_database_error().unwrap();
    assert_eq!(
        db_err.constraint(),
        Some("channel_identities_channel_channel_user_id_key")
    );
}

#[sqlx::test]
async fn link_code_is_single_use_via_used_at(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO link_codes (code, user_id, expires_at) VALUES ('ABC234', $1, now() + interval '10 minutes')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Marking it used is a normal update, not a constraint — this proves the column round-trips.
    sqlx::query("UPDATE link_codes SET used_at = now() WHERE code = 'ABC234'")
        .execute(&pool)
        .await
        .unwrap();

    let used_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT used_at FROM link_codes WHERE code = 'ABC234'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used_at.is_some());

    // A second link_codes row can't reuse the same primary key.
    let dup_err = sqlx::query(
        "INSERT INTO link_codes (code, user_id, expires_at) VALUES ('ABC234', $1, now() + interval '10 minutes')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(
        dup_err.as_database_error().unwrap().constraint(),
        Some("link_codes_pkey")
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test identity`
Expected: FAIL — `relation "users" does not exist`

- [ ] **Step 3: Write the migration**

```sql
-- migrations/0002_identity.sql
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE channel_identities (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id),
    channel           TEXT NOT NULL,
    channel_user_id   TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (channel, channel_user_id)
);

CREATE TABLE link_codes (
    code        TEXT PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ
);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test identity`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add migrations/0002_identity.sql tests/identity.rs
git commit -m "feat: add users, channel_identities, link_codes schema"
```

---

### Task 3: Organizations & RBAC Schema — `organizations`, `memberships`, `org_invites`

**Files:**
- Create: `migrations/0003_organizations.sql`
- Test: `tests/organizations.rs`

**Interfaces:**
- Consumes: `users.id` (Task 2).
- Produces: `organizations(id, name, is_personal)`, `memberships(id, org_id, user_id, role, status)` with `UNIQUE (org_id, user_id)`, `org_invites(code, org_id, role, expires_at, used_at)`. Task 4 references `organizations.id` for `sessions.org_id`.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/organizations.rs
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn membership_is_unique_per_org_and_user(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_org_id_user_id_key")
    );
}

#[sqlx::test]
async fn personal_org_defaults_to_false_unless_set(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Real Team') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let is_personal: bool =
        sqlx::query_scalar("SELECT is_personal FROM organizations WHERE id = $1")
            .bind(org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!is_personal);

    let personal_org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Dinda (personal)', true) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let is_personal: bool =
        sqlx::query_scalar("SELECT is_personal FROM organizations WHERE id = $1")
            .bind(personal_org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(is_personal);
}

#[sqlx::test]
async fn org_invite_code_is_unique_and_single_use(pool: PgPool) {
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
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('XYZ789', $1, 'member', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let dup_err = sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('XYZ789', $1, 'member', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        dup_err.as_database_error().unwrap().constraint(),
        Some("org_invites_pkey")
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test organizations`
Expected: FAIL — `relation "organizations" does not exist`

- [ ] **Step 3: Write the migration**

```sql
-- migrations/0003_organizations.sql
CREATE TABLE organizations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    is_personal BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE memberships (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    role        TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'active',
    invited_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, user_id)
);

CREATE TABLE org_invites (
    code        TEXT PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id),
    role        TEXT NOT NULL,
    invited_by  UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ
);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test organizations`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add migrations/0003_organizations.sql tests/organizations.rs
git commit -m "feat: add organizations, memberships, org_invites schema"
```

---

### Task 4: Sessions Schema — `sessions`, `messages`, `session_participants`

**Files:**
- Create: `migrations/0004_sessions.sql`
- Test: `tests/sessions.rs`

**Interfaces:**
- Consumes: `organizations.id` (Task 3), `users.id`/`channel_identities.id` (Task 2).
- Produces: `sessions(id, org_id, channel, chat_type, chat_id)` with `UNIQUE (channel, chat_id)`, `messages(id, session_id, sender_channel_identity_id, content)`, `session_participants(session_id, user_id, invited_by, added_at)`. Task 5 references `sessions.id`; Task 6 references `sessions.id`.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/sessions.rs
use sqlx::PgPool;
use uuid::Uuid;

async fn make_org(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn make_user_with_channel_identity(pool: &PgPool, channel_user_id: &str) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(user_id)
    .bind(channel_user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, identity_id)
}

#[sqlx::test]
async fn session_requires_an_org(pool: PgPool) {
    let err = sqlx::query(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES (gen_random_uuid(), 'telegram', 'chat-1')",
    )
    .execute(&pool)
    .await
    .unwrap_err();

    // gen_random_uuid() here is a random, non-existent org — this must fail the FK, not just NOT NULL.
    assert!(err.as_database_error().unwrap().message().contains("violates foreign key constraint"));
}

#[sqlx::test]
async fn session_is_unique_per_channel_and_chat_id(pool: PgPool) {
    let org_id = make_org(&pool).await;
    sqlx::query("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1')")
        .bind(org_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1')")
        .bind(org_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("sessions_channel_chat_id_key")
    );
}

#[sqlx::test]
async fn message_sender_is_nullable_for_assistant_replies(pool: PgPool) {
    let org_id = make_org(&pool).await;
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (_user_id, identity_id) = make_user_with_channel_identity(&pool, "111").await;

    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, 'hi')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, 'hello back')")
        .bind(session_id)
        .execute(&pool)
        .await
        .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[sqlx::test]
async fn session_participant_cannot_be_added_twice(pool: PgPool) {
    let org_id = make_org(&pool).await;
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, 'web', 'group', 'group-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (user_id, _identity_id) = make_user_with_channel_identity(&pool, "222").await;

    sqlx::query("INSERT INTO session_participants (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO session_participants (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("session_participants_pkey")
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test sessions`
Expected: FAIL — `relation "sessions" does not exist`

- [ ] **Step 3: Write the migration**

```sql
-- migrations/0004_sessions.sql
CREATE TABLE sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id),
    channel     TEXT NOT NULL,
    chat_type   TEXT NOT NULL DEFAULT 'dm',
    chat_id     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (channel, chat_id)
);

CREATE TABLE messages (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID REFERENCES channel_identities(id),
    content                     TEXT NOT NULL,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE session_participants (
    session_id  UUID NOT NULL REFERENCES sessions(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    invited_by  UUID REFERENCES users(id),
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, user_id)
);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test sessions`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add migrations/0004_sessions.sql tests/sessions.rs
git commit -m "feat: add sessions, messages, session_participants schema"
```

---

### Task 5: Agent Lifecycle Schema — `agent_sessions`

**Files:**
- Create: `migrations/0005_agent_sessions.sql`
- Test: `tests/agent_sessions.rs`

**Interfaces:**
- Consumes: `sessions.id` (Task 4), `channel_identities.id` (Task 2).
- Produces: `agent_sessions(id, session_id, sender_channel_identity_id, agent_type, status, state, started_at, last_activity_at, ended_at)` with the partial unique index `agent_sessions_one_active_per_speaker` on `(session_id, sender_channel_identity_id) WHERE status = 'active'`. Task 6's `agent_events.agent_session_id` references this table.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/agent_sessions.rs
use sqlx::PgPool;
use uuid::Uuid;

async fn make_session_and_speaker(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '333') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, identity_id)
}

#[sqlx::test]
async fn only_one_active_agent_session_per_speaker(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("agent_sessions_one_active_per_speaker")
    );
}

#[sqlx::test]
async fn a_completed_and_a_new_active_agent_session_can_coexist(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'booking', 'completed', now())",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    // A new active row is fine — the partial index only guards 'active' rows.
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[sqlx::test]
async fn agent_session_state_defaults_to_empty_json_object(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let state: serde_json::Value =
        sqlx::query_scalar("SELECT state FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(state, serde_json::json!({}));
}
```

- [ ] **Step 2: Add `serde_json` as a dependency (needed by the state-default test)**

Add to `Cargo.toml` under `[dependencies]`:

```toml
serde_json = "1"
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --test agent_sessions`
Expected: FAIL — `relation "agent_sessions" does not exist`

- [ ] **Step 4: Write the migration**

```sql
-- migrations/0005_agent_sessions.sql
CREATE TABLE agent_sessions (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID NOT NULL REFERENCES sessions(id),
    sender_channel_identity_id  UUID NOT NULL REFERENCES channel_identities(id),
    agent_type                  TEXT NOT NULL,
    status                      TEXT NOT NULL,
    state                       JSONB NOT NULL DEFAULT '{}',
    started_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_activity_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at                    TIMESTAMPTZ
);

CREATE UNIQUE INDEX agent_sessions_one_active_per_speaker
    ON agent_sessions (session_id, sender_channel_identity_id) WHERE status = 'active';
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --test agent_sessions`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock migrations/0005_agent_sessions.sql tests/agent_sessions.rs
git commit -m "feat: add agent_sessions schema with per-speaker active-agent invariant"
```

---

### Task 6: Memory & Audit Schema — `memory_items`, `agent_events`

**Files:**
- Create: `migrations/0006_memory_and_events.sql`
- Test: `tests/memory_and_events.rs`

**Interfaces:**
- Consumes: `users.id` (Task 2), `sessions.id` (Task 4), `channel_identities.id` (Task 2), `agent_sessions.id` (Task 5), the `vector` extension (Task 1).
- Produces: `memory_items(id, user_id, content, embedding, weight)`, `agent_events(id, session_id, sender_channel_identity_id, agent_session_id, agent_type, event_type, payload, created_at)`. Nothing later in this plan depends on these, but the orchestrator's turn-loop plan (next plan in the sequence) will.

- [ ] **Step 1: Write the failing tests**

```rust
// tests/memory_and_events.rs
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn memory_item_accepts_a_1536_dimension_embedding_and_default_weight(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let embedding = format!("[{}]", vec!["0.01"; 1536].join(","));
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, 'likes window seats', $2::vector) RETURNING id",
    )
    .bind(user_id)
    .bind(&embedding)
    .fetch_one(&pool)
    .await
    .unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(weight, 1.0);
}

#[sqlx::test]
async fn memory_item_rejects_wrong_dimension_embedding(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let wrong_size_embedding = format!("[{}]", vec!["0.01"; 3].join(","));
    let err = sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, 'bad row', $2::vector)",
    )
    .bind(user_id)
    .bind(&wrong_size_embedding)
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(err
        .as_database_error()
        .unwrap()
        .message()
        .contains("expected 1536 dimensions"));
}

#[sqlx::test]
async fn agent_event_stores_jsonb_payload_and_links_to_agent_session(pool: PgPool) {
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
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '444') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let payload = serde_json::json!({"summary": "spawned booking agent"});
    sqlx::query(
        "INSERT INTO agent_events (session_id, sender_channel_identity_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'booking', 'AgentSpawned', $4)",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(agent_session_id)
    .bind(&payload)
    .execute(&pool)
    .await
    .unwrap();

    let stored_payload: serde_json::Value =
        sqlx::query_scalar("SELECT payload FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored_payload, payload);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --test memory_and_events`
Expected: FAIL — `relation "memory_items" does not exist`

- [ ] **Step 3: Write the migration**

```sql
-- migrations/0006_memory_and_events.sql
CREATE TABLE memory_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    content     TEXT NOT NULL,
    embedding   VECTOR(1536) NOT NULL,
    weight      DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX memory_items_embedding_idx
    ON memory_items USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

CREATE TABLE agent_events (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID REFERENCES sessions(id),
    sender_channel_identity_id  UUID REFERENCES channel_identities(id),
    agent_session_id            UUID REFERENCES agent_sessions(id),
    agent_type                  TEXT,
    event_type                  TEXT NOT NULL,
    payload                     JSONB NOT NULL DEFAULT '{}',
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test --test memory_and_events`
Expected: PASS (3 tests)

- [ ] **Step 5: Run the full test suite once to confirm nothing earlier regressed**

Run: `cargo test`
Expected: PASS (all tests across all six `tests/*.rs` files)

- [ ] **Step 6: Commit**

```bash
git add migrations/0006_memory_and_events.sql tests/memory_and_events.rs
git commit -m "feat: add memory_items (pgvector) and agent_events schema"
```

---

## Self-Review Notes

- **Spec coverage**: every table named across `plans/initial.md` (`sessions`, `messages`, `memory_items`, `agent_events`), `2026-07-22-sub-agent-lifecycle-design.md` (`agent_sessions` + partial index), `2026-07-22-cross-channel-identity-design.md` (`users`, `channel_identities`, `link_codes`, `sessions.chat_type`/`chat_id`, `messages.sender_channel_identity_id`), and `2026-07-25-multi-tenant-rbac-design.md` (`organizations`, `memberships`, `org_invites`, `sessions.org_id`, `session_participants`) has a task and a passing-test proof above.
- **Not covered here, by design**: the orchestrator's turn-loop logic, the auth/claims layer, and the SvelteKit frontend are separate follow-up plans per the agreed implementation order — this plan is schema-only.
- **Type consistency**: `sender_channel_identity_id`, `agent_session_id`, `org_id`, and `session_id` column names/types are used identically across Tasks 4–6's migrations and tests.
