# Relay settings in admin — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Relay starts from three environment variables; everything else is set from Nomi's admin, stored encrypted in databases; the PostgREST secret is Relay's own and never typed.

**Architecture:** Relay keeps its settings as one sealed JSON row (`relay.settings`, AES-GCM with `RELAY_ENCRYPTION_KEY`), loaded at start and swapped live when saved through `GET/PUT /control/settings`. Saving checks both shared databases (reachable, PostgREST's role exists) and both PostgRESTs (answer HTTP), generates the PostgREST JWT secret once, and writes `pgrst.jwt_secret`, `pgrst.jwt_role_claim_key = '.db_role'` and `pgrst.db_schemas` on PostgREST's role in each shared database (`ALTER ROLE … IN DATABASE`, then `NOTIFY pgrst, 'reload config'`). Until saved, Relay answers `503 NOT_CONFIGURED`. Nomi keeps the Relay connection (control URL + key) in `relay_connection`, encrypted with `SETTINGS_ENCRYPTION_KEY`, falling back to `RELAY_CONTROL_URL`/`RELAY_CONTROL_KEY`; an admin page edits the connection and Relay's settings.

**Spec:** the approved design in conversation (2026-10-27): Relay env = `RELAY_DATABASE_URL`, `RELAY_ENCRYPTION_KEY`, `RELAY_CONTROL_KEY` (+ optional `RELAY_BIND`); settings = public domain, shared dev/production database URLs, PostgREST dev/production URLs, PostgREST role, preview origins; database passwords never shown back; preview origins default to where Nomi's previews run.

## Global Constraints
- Numeric literals named; prompts untouched; no AI attribution in commits/PRs.
- Database URLs are answered masked (`postgres://user@host:port/db`), never with the password.
- Frontend: theme tokens, en + id messages, 44px targets, works at 390px and in dark mode.

## Review Focus
1. Saving with a wrong database password: refused with which check failed; the old settings keep serving.
2. Saving without database URLs keeps the stored ones (the form never has the passwords).
3. A non-admin can't read or change either the connection or Relay's settings.
4. The worker process picks up a connection saved in the server process (cache with a short TTL, not process memory only).
5. Relay restarted after settings were saved serves without any of the old environment variables.

### Task 1: Relay settings store and live swap (`relay-core::settings`, `relay-control::settings`, migration `0002_settings.sql`)
- [ ] Failing tests: settings round-trip sealed; masked URLs drop passwords; `Relay` without settings → `NOT_CONFIGURED`; save with an unreachable database → rejected naming it and nothing stored; save keeps stored URLs when omitted; save writes the secret, claim key and schemas on PostgREST's role → implement → pass → commit.
### Task 2: Control routes and gateway read live settings; env shrinks
- [ ] Failing tests: `GET /control/settings` before/after; `PUT` invalid → 422 `SETTINGS_INVALID` with checks; gateway uses saved PostgREST URLs and secret; `Config` needs only the three variables → implement → pass → commit.
### Task 3: Nomi stores the Relay connection (`relay_connection` migration, `relay_settings.rs`)
- [ ] Failing tests: stored connection wins over env; env fallback; cache refreshes after TTL / invalidation → implement → wire into `ServerRelayOps`, deploy, preview route → pass → commit.
### Task 4: Admin API and page (`/api/admin/relay`, `frontend/src/routes/admin/(protected)/relay`)
- [ ] Failing tests: admin-only; GET masks the key and passes Relay's settings/status; PUT connection; PUT settings proxied with Relay's errors → implement page → vitest/svelte-check → commit.
### Task 5: End to end with real PostgREST started from only its database URL; PR.
