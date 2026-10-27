# Relay standalone mode Implementation Plan (PR 1 of 5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One Relay instance can serve exactly one project in one environment, set up entirely
from its environment variables, so Nomi can run one per project on Shipyard.

**Architecture:**
- Standalone mode turns on when `RELAY_PROJECT` is set.
- Relay builds its settings from the environment on top of anything saved: both "shared" URLs
  are its own database, PostgREST is `RELAY_POSTGREST_URL`, the role is `authenticator`, and the
  JWT secret is derived from the encryption key.
- It creates the `authenticator` login, and points PostgREST at itself at start.
- The project is a `dedicated` backend whose two environment rows both name Relay's own
  database.
- Requests are split by path, not host: `/control/*` and `/health` go to the control plane,
  everything else to the gateway, as the one project in `RELAY_ENV`.
- Public URLs are `https://<public_domain>`. Nomi saves the host Shipyard gave the instance as
  `public_domain`.

**Tech Stack:** Rust (axum, sqlx), the `relay/` workspace; GitHub Actions for images.

**Spec:** `docs/superpowers/specs/2026-10-27-relay-on-shipyard-design.md` ("Relay changes").

## Global Constraints

- Nothing changes for a Relay started without `RELAY_PROJECT`. Every existing test stays green
  unchanged.
- Named constants for every literal role name, path prefix and env var name.
- No secret (PGRST password, JWT secret, control key) is logged or appears in a settings view.
- Commits carry no AI attribution.

## Review Focus

- **A second, different project id sent to a standalone Relay.** It's refused with
  `409 CONFLICT`, and nothing is recorded. (Task 3 test.)
- **A standalone Relay restarted.** The JWT secret and the authenticator password are the same
  as before, and PostgREST keeps working. (Task 2: the derived secret is deterministic.)
- **`PUT /control/settings` on a standalone Relay without shared URLs or PostgREST URLs.**
  It's accepted, and the environment's values win over anything sent. (Task 2 test.)
- **Gateway calls on a standalone host whose path starts with `/control`.** They're never
  served as app traffic, and still need the control key. (Task 4 test.)
- **An app request whose Host header is a stray subdomain.** It's still served as the one
  project. Standalone ignores hosts. (Task 4 test.)

---

### Task 1: Standalone config from the environment

**Files:**
- Modify: `relay/crates/relay-core/src/config.rs`

**Interfaces:**
- Produces:
  - `pub struct Standalone { pub project: Uuid, pub env: Env, pub postgrest_url: String, pub pgrst_password: String, pub functions: Option<FunctionsRuntime> }`
  - `Config.standalone: Option<Standalone>`
  - consts `ENV_PROJECT = "RELAY_PROJECT"`, `ENV_ENV = "RELAY_ENV"`,
    `ENV_POSTGREST_URL = "RELAY_POSTGREST_URL"`, `ENV_PGRST_PASSWORD = "RELAY_PGRST_PASSWORD"`,
    `ENV_FUNCTIONS_URL = "RELAY_FUNCTIONS_URL"`, `ENV_FUNCTIONS_SECRET = "RELAY_FUNCTIONS_SECRET"`.

- [ ] Tests in `config.rs`:
  - `without_relay_project_it_is_not_standalone`: `standalone.is_none()`.
  - `relay_project_reads_the_standalone_settings`: `RELAY_ENV=dev` gives `Env::Dev`, and an
    unset `RELAY_ENV` defaults to `Production`. The functions runtime is present only when both
    its URL and secret are set.
  - `standalone_needs_postgrest_and_its_password`: the error names the missing variable. A bad
    UUID names `RELAY_PROJECT`, and a bad env names `RELAY_ENV`.
- [ ] Implement in `from_lookup`. Run `cargo test -p relay-core config`. Commit.

### Task 2: Settings from the environment, the authenticator, and start

**Files:**
- Modify:
  - `relay/crates/relay-core/src/settings.rs`
  - `relay/crates/relay-control/src/lib.rs`
  - `relay/crates/relay-control/src/settings.rs`
  - `relay/crates/relay-control/src/postgrest.rs`
  - `relay/crates/relay-server/src/main.rs`
- Test: `relay/crates/relay-control/tests/standalone.rs` (new, `sqlx::test`, like the other
  control tests).

**Interfaces:**
- Consumes: `Standalone` (Task 1).
- Produces:
  - `relay_core::settings::Settings::standalone(saved: Settings, database_url: &str, encryption_key: &[u8; 32], standalone: &Standalone) -> Settings`
    (pure).
  - `pub const AUTHENTICATOR: &str = "authenticator"` in `relay-control::postgrest`.
  - `pub async fn ensure_authenticator(pool: &PgPool, password: &str) -> Result<(), RelayError>`
    in `relay-control::postgrest`.
  - `Relay::start_standalone(state: PgPool, encryption_key: [u8; 32], database_url: String, standalone: Standalone) -> Result<Relay, RelayError>`.
  - `Relay::standalone(&self) -> Option<&StandaloneMode>`, where
    `pub struct StandaloneMode { pub config: Standalone, pub database_url: String }`.

`Settings::standalone` keeps `public_domain`, `preview_origins`, `shared_smtp` and `storage`
from `saved`. It sets:
- both shared URLs to `database_url`;
- both PostgREST URLs to `postgrest_url`;
- `postgrest_role` to `AUTHENTICATOR`;
- `postgrest_jwt_secret` to `keys::role_password(encryption_key, "pgrst.jwt_secret")`;
- `functions` to `standalone.functions` when set, else `saved.functions`.

`ensure_authenticator`:
- `CREATE ROLE authenticator LOGIN NOINHERIT PASSWORD …` when missing, else
  `ALTER ROLE … LOGIN PASSWORD …`;
- tolerates a duplicate-object race, as `backend::ensure_role` does.

`start_standalone`:
1. loads the saved settings (or default);
2. applies `Settings::standalone`;
3. connects;
4. runs `ensure_authenticator`, then `settings::point_postgrest` (made `pub(crate)`) on its
   own database;
5. replaces live.

`settings::save`: when `relay.standalone()` is set, applies `Settings::standalone` to the merged
settings before the checks.

- [ ] Unit test in `relay-core/src/settings.rs`: `standalone_settings_point_everything_at_its_own_database`.
  - Every field is as listed above.
  - The JWT secret is the same on two calls and differs for a different key.
  - The saved preview origins are kept.
- [ ] `tests/standalone.rs::a_standalone_relay_creates_the_authenticator_and_points_postgrest`:
  - after `start_standalone`, `pg_roles` has `authenticator` with `rolcanlogin`;
  - the role's `pgrst.jwt_secret` setting in that database equals the derived secret;
  - `relay.live()` is ok without any saved settings.
- [ ] `tests/standalone.rs::saving_settings_keeps_the_environment_values`:
  - `save` with empty shared and PostgREST URLs and `public_domain = "app-backend.apps.test"`
    succeeds;
  - the live settings still name its own database and `RELAY_POSTGREST_URL`.
  - The PostgREST check needs an answering URL, so the test points `postgrest_url` at a
    wiremock server, as other settings tests do.
- [ ] `main.rs` calls `start_standalone` when `config.standalone` is set. Run
  `cargo test -p relay-core -p relay-control`. Commit.

### Task 3: One project, as a dedicated backend in its own database

**Files:**
- Modify: `relay/crates/relay-control/src/backend.rs`, `relay/crates/relay-control/src/lib.rs`
- Test: `relay/crates/relay-control/tests/standalone.rs`

**Interfaces:**
- Produces: `Relay::public_url(&self, slug: &str, env: Env) -> Result<String, RelayError>`.
  - Standalone: `https://{public_domain}`.
  - Otherwise: `names::public_url(...)`.

In standalone, `ensure_backend`:
- refuses any id but `standalone.config.project` with
  `RelayError::rejected("CONFLICT", "this Relay serves one project")`;
- otherwise ignores the requested tier and databases, and uses `Tier::Dedicated` with both
  environments set to `standalone.database_url`.

`backend_keys` uses `relay.public_url`.

- [ ] `tests/standalone.rs::it_serves_only_its_own_project`:
  - `ensure_backend` for the configured id succeeds with `tier == "dedicated"`;
  - `environments["production"].url == "https://app-backend.apps.test"`, after saving that
    public domain;
  - a different id is `Rejected { code: "CONFLICT", .. }`;
  - `relay.projects` has one row.
- [ ] `tests/standalone.rs::migrations_land_in_public`: `migrations::apply` with
  `create table task (id int)` for the standalone project's production env creates
  `public.task`.
- [ ] Implement. Replace `public_url` in `backend.rs` and in `relay-server/src/auth_routes.rs`
  (the callback URL) with `relay.public_url`. Run the control tests. Commit.

### Task 4: Path-split routing on a standalone host

**Files:**
- Modify: `relay/crates/relay-server/src/app.rs`
- Test: `relay/crates/relay-server/tests/standalone.rs` (new; follow the existing server tests'
  setup).

In `router`, when `relay.standalone()` is set:
- a path that is `/health` or starts with `CONTROL_PREFIX = "/control"` goes to the control
  router;
- anything else goes to `gateway.handle(slug, env, request)`, with the project's slug read from
  `state::environment(relay, project, env)` and `env` from the config;
- before `ensure_backend` (no row yet), the gateway answers `503` with code `NOT_CONFIGURED`
  in PostgREST's error shape (`gateway::refuse`).

Without standalone, routing is unchanged.

- [ ] `tests/standalone.rs`:
  - `control_calls_still_need_the_control_key`: `GET /control/settings` without a key is `401`.
  - `any_host_reaches_the_one_project`: after `ensure_backend`, `GET /rest/` with the anon key
    and `Host: whatever.example` reaches the gateway, not the control plane. Assert it isn't the
    control plane's `404`/`401`, and that the request reached the stand-in PostgREST.
  - `before_the_backend_exists_the_gateway_says_not_configured`: `503` with `NOT_CONFIGURED`.
- [ ] Implement. Run `cargo test -p relay-server`. Commit.

### Task 5: Images published from CI, and the README

**Files:**
- Modify: `.github/workflows/relay-service.yml`, `relay/README.md` ("Setting Relay up")

- [ ] Add a job `images` to `relay-service.yml`:
  - runs on `push` to main only, after `test` and `functions-runtime`;
  - `permissions: packages: write`;
  - `docker/login-action` to `ghcr.io` with `GITHUB_TOKEN`;
  - `docker/build-push-action` builds `relay/Dockerfile` as `ghcr.io/triandamai/nomi-relay`
    and `relay/functions-runtime/Dockerfile` as `ghcr.io/triandamai/nomi-relay-functions`;
  - tags are `sha-<short sha>` and the version in `relay/sdk/js/package.json`.
- [ ] README: a "Standalone" paragraph listing the variables, the path split, and that Nomi runs
  one per project.
- [ ] `actionlint` if available, else check the YAML parses (`python3 -c "import yaml…"`). Commit.

### Finish

- [ ] Run `cargo test --workspace` locally for the tests that don't need Postgres (`cargo test
  --workspace --lib`). The `sqlx::test` suites run in CI (Docker is down on this machine).
- [ ] Push `relay/standalone`, open PR 1, and check CI.
