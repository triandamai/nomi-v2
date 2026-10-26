# Relay Phase 1: control plane and migrations — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Nomi Relay exists as a service: it makes a project's backend (database objects, `nomi` schema, keys), applies the project's migrations safely, and answers its schema and TypeScript types; Nomi uses it so Koda's migrations land on `dev` as he writes them and on production when the project is published.

**Architecture:** A Rust/axum service in `relay/` (its own Cargo workspace, like `backend/`), with its state in its own Postgres (`relay` schema: projects, environments, encrypted signing secrets). Free projects live in shared databases, one schema (`app_<id>`) and four roles each (`_owner` for migrations, `_anon`, `_authenticated`, `_service`); a dedicated project gets its own databases and uses `public`, same code path. Every database Relay touches has the `nomi` schema (users, identities, sessions, migrations, buckets with `project_id` and row-level security; `nomi.uid()`, `nomi.role()`, `nomi.claims()`, `nomi.project_id()` reading `request.jwt.claims`). The control plane is service-to-service (`RELAY_CONTROL_KEY`), matching Phase 0's `nomi-relay-client`.

**Tech Stack:** Rust (axum 0.7, sqlx 0.7, jsonwebtoken 9, aes-gcm 0.10), Postgres, Svelte untouched.

**Spec:** `relay/README.md`; phase plan `docs/superpowers/plans/2026-10-26-relay-service-and-koda.md` (Phase 1).

## Global Constraints

- Control-plane paths and envelope exactly as `backend/crates/nomi-relay-client` (Phase 0): `PUT/DELETE /control/projects/:id`, `POST /control/projects/:id/environments/:env/migrations`, `GET …/schema`, `GET …/types`; `{ "data": …, "error": { "code", "message", "details" } | null }`; 401 for a bad key; 422 `MIGRATION_FAILED` / `LINT_FAILED` / `MIGRATION_EDITED` with `details` `{ file, line }` where known.
- Lint rules identical to `relay/cli/src/lint.ts`; the CLI's lint test cases run against the Rust linter too.
- Environments are `dev` and `production` only.
- Every numeric literal is a named constant; secrets (signing keys, dedicated database URLs) are stored encrypted (AES-256-GCM, `RELAY_ENCRYPTION_KEY`).
- Nomi side: no prompt text outside `prompts.rs`; guides live in `nomi-agent-coding/guides/`.

## Review Focus

1. Two free projects in one shared database: one's roles can't read the other's tables or the other's `nomi.users` rows.
2. A migration edited after it was applied: refused with `MIGRATION_EDITED` naming the file; nothing applied.
3. A failing statement in the middle of a file: that file rolls back entirely, earlier files stay applied, the error names the file and the line.
4. Calling `PUT` twice: the second is a no-op answering the same keys (idempotent), no duplicate roles or errors.
5. A migration trying `set role postgres` or touching `nomi.*`: refused by lint before anything runs.

---

### Task 1: Workspace, state, keys (`relay-core`)

**Files:** `relay/Cargo.toml` (workspace), `relay/crates/relay-core/{Cargo.toml,src/lib.rs,src/config.rs,src/crypto.rs,src/keys.rs,src/names.rs}`, `relay/migrations/0001_state.sql`.

**Produces:** `Config::from_env()` (`RELAY_DATABASE_URL`, `RELAY_SHARED_DEV_URL`, `RELAY_SHARED_PRODUCTION_URL`, `RELAY_CONTROL_KEY`, `RELAY_ENCRYPTION_KEY` hex-32, `RELAY_PUBLIC_DOMAIN`, `RELAY_BIND`); `crypto::{encrypt, decrypt}`; `keys::{new_signing_secret() -> String, api_key(secret, project, env, role) -> String, Claims}`; `names::{schema_for(project, tier) -> String, role(project, kind) -> String, public_url(domain, project_slug, env) -> String}`; `Env::{Dev, Production}` with `parse`/`as_str`; `Tier::{Free, Dedicated}`.

- [ ] Failing unit tests (config needs every variable and a 32-byte key; encrypt/decrypt round trip and tamper fails; an api key decodes with the secret to `{role, ref, env}` and fails with another secret; schema/role names are `app_<uuid without dashes>` and valid identifiers ≤ 63 chars; URLs: production `https://<slug>.<domain>`, dev `https://<slug>-dev.<domain>`) → implement → pass → commit.

### Task 2: Lint (`relay-migrate::lint`)

**Files:** `relay/crates/relay-migrate/{Cargo.toml,src/lib.rs,src/sql.rs,src/lint.rs}`, tests in `src/lint.rs`.

**Produces:** `split_statements(sql) -> Vec<Statement { code, comments, line }>`, `lint(files: &[MigrationFile]) -> Vec<Issue { file, line, rule, message }>` with rules `row-level-security`, `destructive`, `outside-project` exactly as the CLI.

- [ ] Port every case from `relay/cli/test/lint.test.ts` as failing Rust tests → implement → pass → commit.

### Task 3: Backends (`relay-control::backend`)

**Files:** `relay/crates/relay-control/{Cargo.toml,src/lib.rs,src/backend.rs,src/nomi_schema.sql,src/state.rs}`; tests `relay/crates/relay-control/tests/backend.rs` (`sqlx::test`, the test database standing in for state and both shared databases).

**Produces:** `ensure_backend(ctx, project_id, name, tier, dedicated: Option<Databases>) -> BackendKeys { id, environments: { dev|production: { url, anon_key, service_key } } }` (idempotent); `delete_backend(ctx, project_id)`; `ctx` holds the state pool, shared pools, config.

- [ ] Failing tests: PUT makes schema, four roles, `nomi` schema, state rows, keys; second call answers the same keys; two free projects can't see each other (as A's `_authenticated` role with A's claims, B's table and B's `nomi.users` rows are invisible/denied); a dedicated project uses `public` in its own database; delete removes schema, roles and `nomi` rows of exactly that project → implement → pass → commit.

### Task 4: Migrations (`relay-migrate::apply`)

**Files:** `relay/crates/relay-migrate/src/apply.rs`; tests `relay/crates/relay-control/tests/migrations.rs`.

**Produces:** `apply(pool, backend: &BackendRef, files) -> Result<Applied { applied: Vec<String> }, MigrationError { code, message, file, line }>`: lint first; checksums (sha256) vs `nomi.migrations` (refuse edited); each new file in its own transaction with `SET LOCAL role <owner>`, `SET LOCAL search_path`, `SET LOCAL statement_timeout`; records the row; `NOTIFY pgrst, 'reload schema'` after.

- [ ] Failing tests: applies in order and records; re-run applies nothing; edited file refused; failing statement on line 3 of file 2 → file 2 rolled back, file 1 kept, error `{file, line: 3}`; lint failure refused before anything runs; timeout constant is set → implement → pass → commit.

### Task 5: Schema and types (`relay-migrate::{describe,types}`)

**Files:** `relay/crates/relay-migrate/src/describe.rs`, `src/types.rs`; tests in `relay/crates/relay-control/tests/schema.rs`.

**Produces:** `describe(pool, schema) -> Schema { tables: [{ name, rls_enabled, columns: [{ name, type, nullable, default }], policies: [{ name, command, using, check }] }] }`; `typescript(schema) -> String` in the shape `nomi-relay`'s `DatabaseDef` expects (`export interface Database { tables: { t: { Row; Insert; Update } }; functions: {} }`), Insert making columns with defaults or nullable optional, Update all optional.

- [ ] Failing tests on a migrated schema (`task` with identity id, `owner uuid default nomi.uid()`, `title text not null`, `done boolean default false`, `created_at timestamptz default now()`, RLS and a policy) → implement → pass → commit.

### Task 6: The service (`relay-server`)

**Files:** `relay/crates/relay-server/{Cargo.toml,src/main.rs,src/app.rs,src/envelope.rs}`, tests `relay/crates/relay-server/tests/control.rs`; `relay/Dockerfile`.

- [ ] Failing tests through the router: no/wrong bearer → 401 envelope; PUT then migrations then schema then types round trip; migration failure → 422 with details; unknown env → 404 → implement → pass → commit. Dockerfile builds `relay-server`.

### Task 7: Nomi uses it

**Files:** `backend/crates/nomi-relay-client/src/lib.rs` (ensure_backend returns keys), `backend/crates/nomi-server/src/relay_ops.rs` (store production keys as project secrets on Shipyard), `backend/crates/nomi-server/src/routes/deploy.rs` (publish applies production migrations first), `backend/crates/nomi-agent-coding/src/{lib.rs,relay.rs,brief.rs,guides.rs}`, `backend/crates/nomi-agent-coding/guides/relay.md`.

- [ ] Failing tests: writing a file under `nomi/migrations/` in a relay project returns the apply result appended to the write's outcome and, on success, writes `src/lib/nomi.types.ts`; a drizzle project's write does nothing extra; the brief of a relay project lists the dev tables and preloads the relay guide; publishing a relay project applies production migrations first and a failure stops the publish with the error (wiremock Relay); ensure_backend stores `NOMI_URL`, `NOMI_ANON_KEY`, `NOMI_SERVICE_KEY` (production) as project secrets when Shipyard is set up → implement → pass → commit.

### Task 8: End to end

- [ ] Run Relay and Nomi locally against Postgres: create a relay project, apply `0001_init.sql` to dev through Koda's tool path, read types, publish path applies production; break a migration and see file and line. Final review; PR.
