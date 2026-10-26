# Relay Phase 2b: sign-in — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** an app's users sign in to its Relay backend the way `nomi-relay`'s `auth` already calls it: email codes, sessions that refresh with rotating tokens, the signed-in user, and `auth.admin.*` with the service key. Then (2b-2) OAuth.

**Spec:** `relay/README.md` (Auth, Security); phase plan Phase 2 item 3; the SDK contract in `relay/sdk/js/src/auth.ts` (paths, bodies, `Session`/`User` shapes, errors as `{ message, code }`).

**Split:** 2b-1 (this plan's Tasks 1–5, one PR): email codes, sessions, user and admin APIs, the shared sender. 2b-2 (its own PR): OAuth with PKCE (Google, GitHub, Apple with a `.p8`-signed secret, OIDC discovery), `linkIdentity`/`unlinkIdentity`, and Nomi sending each project's auth config and its secrets (`NOMI_SMTP_PASSWORD`, `NOMI_OAUTH_<P>_SECRET`) to Relay.

## Global Constraints
- Paths and shapes exactly as the SDK calls them: `POST /auth/otp {email, create_user}`, `POST /auth/verify {email, code}` → `Session`, `POST /auth/token?grant_type=refresh_token {refresh_token}` → `Session`, `GET/PATCH /auth/user` → `User`, `POST /auth/logout` (bearer = access token), `GET/POST /auth/admin/users`, `GET/PATCH/DELETE /auth/admin/users/:id`.
- `Session = { access_token, refresh_token, token_type: "bearer", expires_at (unix s), user }`; `User = { id, email, email_verified, created_at, last_sign_in_at, identities, user_metadata }`.
- Access tokens 15 minutes (`ACCESS_TOKEN_SECONDS = 900`), signed with the environment's key, carrying `sid`; refresh tokens 32 random bytes, stored as SHA-256, valid `session_ttl_days` (default 30).
- Codes: six digits, 10 minutes, 5 tries, stored hashed. Shared sender limits: 5 codes per email per hour, 100 per project per hour; a project's own SMTP: 5 per email per hour, 1000 per project per hour.
- Numeric literals named; no prompts outside `prompts.rs`; no AI attribution.

## Rulings
- **Shared sender:** its SMTP server is part of Relay's settings (Admin → Integrations → Relay), sealed like the rest; no new environment variables. Without it (and without a project SMTP), `/auth/otp` answers 503 `RELAY_NO_SENDER`. Cost if wrong: a settings field.
- **Per-project auth config** (methods, redirect URLs, session days, SMTP with password) is kept sealed in Relay's state per environment (`relay.auth_configs`), set through `PUT /control/projects/:id/environments/:env/auth`; Nomi starts sending it in 2b-2. Default: `["email_code"]`, 30 days, the shared sender.
- **`create_user: false` with an unknown email** answers 200 and sends nothing, so the endpoint doesn't reveal who has an account.
- **Changing email through `PATCH /auth/user`** is refused (`EMAIL_CHANGE_UNSUPPORTED`) until it can be verified; `user_metadata` merges.
- **Refresh reuse:** using a refresh token that was already rotated or revoked revokes every session in its family and answers 401.
- **Access tokens after sign-out:** stateless on `/rest` for their 15 minutes; `/auth/*` checks the session (`sid`) is still live.

## Review Focus
1. A refresh token used twice: the second use fails and the newer token from the first use stops working too.
2. A code guessed wrong 5 times: the code is dead even if the 6th guess is right.
3. A user token from project A on project B, or dev on production: refused (signature and `ref`/`env`).
4. `auth.admin.*` with the anon key or a user token: refused.
5. Deleting a user ends their sessions: their refresh token fails right away.

### Task 1: Tokens carry the session; codes table (`relay-core::keys`, `nomi_schema.sql`)
- [ ] Failing tests: `user_token` takes and returns `sid`; `nomi.codes` exists after provisioning → implement → commit.
### Task 2: `relay-auth` crate (codes, sessions, users, mail, config)
- [ ] Failing tests (sqlx): code sent through the chosen sender (project SMTP over shared) with a recording outbox; verify creates the user (verified) and an `email` identity, returns a session; wrong code ×5 kills it; expired refused; limits refuse the 6th per email; refresh rotates; reuse revokes the family; revoke by user; admin list/get/create/update/delete; config sealed round-trip → implement → commit.
### Task 3: Gateway `/auth/*` and control `.../auth` (`relay-server`)
- [ ] Failing tests through the router: otp → verify → user → refresh → logout with the SDK's shapes; admin needs the service key; other project's token refused; `PUT …/auth` stores config → implement → commit.
### Task 4: Shared sender in Relay's settings and the admin page
- [ ] Failing tests: settings keep the SMTP password when omitted, mask it, check the server answers; frontend body/fields → implement → commit.
### Task 5: End to end with the real SDK (`nomi-relay` in Node) against Relay, PostgREST and a local SMTP catcher; PR stacked on #80.

---

## 2b-2: OAuth, linking, and Nomi sending sign-in settings

**Goal:** `signInWithOAuth('google' | 'github' | 'apple' | 'oidc:<id>')`, `exchangeCodeForSession`, `linkIdentity` and `unlinkIdentity` work as the SDK calls them; a project's `nomi/backend.json` `auth` block and its secrets reach Relay without anyone copying them.

**Contract (SDK):** `GET /auth/authorize/:provider?redirect_to&scopes&code_challenge&code_challenge_method=S256&apikey` (a browser navigation, so the key is in the query); Relay's callback sends the browser to `redirect_to?code=…`; `POST /auth/token?grant_type=pkce {code, code_verifier}` → `Session`; `POST /auth/identities {provider, redirect_to, scopes, code_challenge, code_challenge_method}` → `{url}`; `DELETE /auth/identities/:id`.

**`backend.json` → `AuthConfig`:** `methods` (`email_code`, `google`, `github`, `apple`, `oidc:<id>`), `google {client_id}`, `github {client_id, api_base?, web_base?}` (Enterprise), `apple {client_id, team_id, key_id}`, `oidc [{id, issuer, client_id}]`, `redirect_urls`, `session_ttl_days`, `smtp` (no password). **Secrets** are project secrets by name, forwarded to Relay when set: `NOMI_SMTP_PASSWORD`, `NOMI_OAUTH_GOOGLE_SECRET`, `NOMI_OAUTH_GITHUB_SECRET`, `NOMI_OAUTH_APPLE_KEY` (the `.p8`), `NOMI_OAUTH_OIDC_<ID>_SECRET`.

### Rulings
- **Google and Apple are OpenID Connect** (discovery from `https://accounts.google.com`, `https://appleid.apple.com`); only GitHub has its own code. ID tokens are checked against the issuer's JWKS (RS256/ES256), `iss`, `aud = client_id`, `exp` and the flow's `nonce`. Cost if wrong: provider-specific code later.
- **Apple's client secret** is an ES256 JWT signed from the `.p8` per token exchange, valid 5 minutes. Apple's `form_post` callback is accepted (POST form) as well as GET.
- **Callback URL** is `https://<host>/auth/callback/<provider>`, the host as requested; one per provider, never the preview's origin.
- **`redirect_to`** must start with one of `redirect_urls`; on `dev`, Relay's preview origins and `localhost` are allowed too. Otherwise 400 before leaving for the provider.
- **Linking:** a provider-verified email joins the user who has that verified email; an unverified one makes a new user without an email (the address stays on the identity). Linking an identity that belongs to another user is refused; the last identity can't be removed.
- **Flows** live in `nomi.oauth_flows`: state and the one-time code hashed, 10 minutes to come back from the provider, 5 minutes to trade the code, PKCE `S256` only.
- **Nomi sends** `backend.json`'s `auth` to `dev` when Koda writes it and to `production` when the project is published (before its migrations), and forwards the secrets above when an admin sets or removes them; Relay keeps them sealed (`relay.auth_secrets`). `GET …/auth` answers the config and which switched-on providers are missing a secret, and `describe_backend` says so.

### Review Focus (2b-2)
1. A forged or replayed `state`, or a callback after 10 minutes: refused.
2. A wrong `code_verifier`, or the code traded twice: refused.
3. `redirect_to` on another site: refused before leaving for the provider.
4. An ID token for another `aud`, expired, with the wrong `nonce` or a bad signature: refused.
5. An unverified provider email equal to an existing user's: never joins that user.

### Task 6: Providers (`relay-auth::oauth`) — failing tests with wiremock as an OIDC issuer (discovery, JWKS, token) and as GitHub; Apple secret JWT verifies with the key's public half → implement → commit.
### Task 7: Flows and routes — authorize/callback/token(pkce)/identities through the router → implement → commit.
### Task 8: Secrets and status — `PUT/DELETE /control/projects/:id/auth/secrets/:name`; `GET …/auth` lists missing secrets → implement → commit.
### Task 9: Nomi — client calls; `after_write` on `nomi/backend.json`; publish sends production; project secrets forward; `describe_backend` → implement → commit.
### Task 10: Koda's relay guide documents `backend.json` auth and secret names; end to end with the SDK and a wiremock OIDC provider; PR stacked on #82.
