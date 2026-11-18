# Brief — design-studio-publish-status

## The request
"Make an unpublished design preview visible and retryable" (point 9B of the Shipyard & Nomi
requirements, 17 Nov 2026). Sora told the person a design was finished and viewable, but
nothing had been deployed to Shipyard, and nothing in Nomi said so.

## Who it's for
Someone who asked Sora for a design and opens it in the Design Studio
(`/chat/[sessionId]/design`, old frontend) to review it, share it, or hand it to Koda.

## Why now
A person was told the work was done while it wasn't published anywhere outside Nomi. That
breaks "no surprises": the Studio has to say when a version isn't published, why, and let them
try again.

## What the backend already gives (PR 289)
- `GET /api/sessions/:id/design` — each version has `published_url` (string or null) and the new
  `publish_error` (string or null: why its newest publish failed; null once a later one succeeds).
- `POST /api/sessions/:id/design/publish` `{version}` — publishes that version again. Answers
  `200 {version, published_url, publish_error}`; `409` "this version is already published";
  `404` "no such version". An upload can take up to about two minutes.
- Known reasons today: "publishing previews isn't set up" (Shipyard isn't configured), Shipyard's
  own refusal text, "its address couldn't be kept".
- Today the Studio only shows an "Open published" link when `published_url` is set
  (`frontend/src/routes/(app)/chat/[sessionId]/design/+page.svelte`); a missing address shows nothing.

## In scope
- The Studio shows, for the version on screen, that it isn't published and why.
- A way to publish it again from there, with its in-progress, success and failure states.
- Desktop and mobile, dark and light.

## Out of scope
- An entry point to the Design Studio from the chat or project (point 9A; separate brief).
- The Design Studio in frontend-new (point 9A; separate project).
- Element-level comments (point 6).

## Open for ux-lead
- Whether a "not set up" reason should be worded for the person differently from a Shipyard
  refusal (one is an admin problem, the other may be worth retrying).
- Whether Retry is offered when the reason is "isn't set up" (retrying can't help until an admin
  configures Shipyard).
- No Design Studio flow is listed in `flow-registry.md` yet.
