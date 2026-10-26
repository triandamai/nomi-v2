# Relay Phase 2a: the gateway and the Data API — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apps reach their data: `https://<slug>[-dev].<domain>/rest/<table>` with the anon key (or, from 2b, a user's token) goes through Relay's gateway to PostgREST, as the project's own role, and row-level security decides what comes back. The preview gets its dev address and key.

**Architecture:** The gateway is a second router in `relay-server`, picked by hostname (control plane on the bare host or a configured control host; projects on `<slug>.<domain>` / `<slug>-dev.<domain>`). It verifies the `apikey` and `Authorization` token against the environment's signing secret, maps anon → `app_<id>_anon`, user → `app_<id>_authenticated`, service → `app_<id>_owner` (the table owner, so it bypasses the project's row-level security and nothing else's), signs a 60-second internal JWT for PostgREST with `RELAY_POSTGREST_JWT_SECRET` (`db_role` for PostgREST's `jwt-role-claim-key`, `role` = anon/authenticated/service for `nomi.role()`, `sub`, `ref`, `env`), sets `Accept-Profile`/`Content-Profile` to the project's schema (dropping the client's), strips `/rest`, and proxies to the environment's PostgREST (`RELAY_POSTGREST_DEV_URL` / `..._PRODUCTION_URL`). Provisioning grants the project roles to PostgREST's authenticator role (`RELAY_POSTGREST_ROLE`), sets per-role `statement_timeout`, and keeps the authenticator's `pgrst.db_schemas` equal to every free project's schema in that database, then `NOTIFY pgrst, 'reload config'`.

**Tech Stack:** Rust (axum, reqwest for the proxy), PostgREST 12 (docker for the end-to-end check), Postgres.

**Spec:** `relay/README.md` (PostgREST section), phase plan Phase 2 items 1, 2, 4.

## Global Constraints

- The client's paths stay as `nomi-relay` uses them: `/rest/<table>`, `/rest/rpc/<fn>`.
- PostgREST's own errors pass through untouched (the SDK reads its `{ message, code, details, hint }`); the gateway's own refusals answer in that same shape (`{ "message", "code" }`), not the control plane's envelope, so the SDK reads them the same way.
- Numeric literals are named constants; secrets never logged.

## Rulings (from the spec's open points)

- **CORS:** dev allows Nomi's preview origins (`RELAY_PREVIEW_ORIGINS`, suffix match) and localhost; production allows the project's allowed origins when Relay has any, else any origin (row-level security is the protection, as with every BaaS anon key). Cost if wrong: one setting per project.
- **Service key's database role:** the project's owner role, not a `BYPASSRLS` role, so it can't reach other projects' `nomi` rows. The unused `_service` role stays for later. Cost if wrong: a role mapping.
- **Dedicated projects:** a PostgREST per dedicated environment comes with Shipyard wiring later; 2a serves free projects and answers 501 for a dedicated one. Cost if wrong: dedicated projects wait.
- **Preview service key:** only the project's admins get the dev service key in their preview; everyone gets the dev URL and anon key.

## Review Focus

1. A token from project A sent to project B's host: refused (signature), never proxied.
2. A client-sent `Accept-Profile: app_<other>`: dropped and replaced, so no request reaches another schema.
3. An expired user token or a key for the other environment: refused.
4. A request body over the limit: refused before proxying.
5. Adding and deleting projects keeps `pgrst.db_schemas` exactly the free projects' schemas of that database.

---

### Task 1: Gateway auth (`relay-gateway::auth`)

**Files:** `relay/crates/relay-gateway/{Cargo.toml,src/lib.rs,src/host.rs,src/auth.rs}`.

**Produces:** `host::route(host, domain) -> Option<(slug, Env)>`; `auth::resolve(secret, project, env, apikey, bearer, now) -> Result<Caller { role: Kind, sub: Option<Uuid>, claims }, Refusal>`; `auth::internal_token(postgrest_secret, project, caller, now) -> String`.

- [ ] Failing unit tests: hosts (`bakery-6dfd42.api.x` → production, `bakery-6dfd42-dev.api.x` → dev, foreign domain → None); anon/service keys resolve; wrong secret, other env, other project, missing apikey, expired token refused; bearer user token (role authenticated, sub) resolves; internal token carries `db_role`, `role`, `sub`, `ref`, `exp` ≤ 60 s → implement → pass → commit.

### Task 2: The proxy (`relay-gateway::proxy`) and routing in `relay-server`

**Files:** `relay/crates/relay-gateway/src/proxy.rs`, `relay/crates/relay-server/src/app.rs` (host-based dispatch), `relay/crates/relay-core/src/config.rs` (new settings), tests `relay/crates/relay-server/tests/gateway.rs` (wiremock as PostgREST).

- [ ] Failing tests: GET `/rest/task?select=*` on the project host reaches PostgREST at `/task?select=*` with `Authorization: Bearer <internal>` (decodes to `db_role = app_<id>_anon`) and `Accept-Profile: app_<id>` even when the client sent another; POST sends `Content-Profile`; `/rest/rpc/f` → `/rpc/f`; PostgREST's status, body and `Content-Range` pass through; no apikey → 401; body over the limit → 413; unknown slug → 404; OPTIONS preflight answers CORS per the rulings → implement → pass → commit.

### Task 3: Provisioning for PostgREST

**Files:** `relay/crates/relay-control/src/backend.rs`, `src/postgrest.rs`; tests in `relay/crates/relay-control/tests/backend.rs`.

- [ ] Failing tests: after ensure, the authenticator role is a member of the project's anon/authenticated/owner roles; anon and authenticated have `statement_timeout` set; the authenticator's `pgrst.db_schemas` lists exactly the free projects' schemas; after delete it no longer lists the deleted one → implement → pass → commit.

### Task 4: Nomi gives the preview its dev access

**Files:** Relay `GET /control/projects/:id` (read the keys without changing anything); `backend/crates/nomi-relay-client` (`backend()`); `nomi-server` route `GET /api/projects/:id/relay/preview` (url + anon key for members, + service key for admins; 404 when not on Relay); project proxy allowlist; `frontend/src/lib/projectRuntime.svelte.ts` (merge into the preview env as `PUBLIC_NOMI_URL`, `PUBLIC_NOMI_KEY`, `NOMI_URL`, `NOMI_ANON_KEY`, and `NOMI_SERVICE_KEY` when given).

- [ ] Failing tests for each layer (Relay route, client, Nomi route access rules, allowlist, the runtime's env merge helper) → implement → pass → commit.

### Task 5: End to end

- [ ] Docker: Postgres + PostgREST 12 (authenticator role, `jwt-role-claim-key=.db_role`, in-db config) + relay-server; make a project, migrate a `task` table with a policy, insert as owner, then `curl` the gateway: anon sees what the policy allows, a service-key request sees all, another project's key is refused, a forged `Accept-Profile` is ignored. PR stacked on Phase 1.
