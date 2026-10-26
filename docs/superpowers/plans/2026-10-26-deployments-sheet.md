# Deployments sheet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Part 1 of the project sheets redesign: the Deployments sheet and publish details to the approved mockup, rollback that works end to end, and Koda knowing about a failed publish.

**Architecture:** The backend's deploy overview gains what the rows need (which publish is live per environment, the version a rollback returned to, whether an agent started it). Rollback refuses targets that can't work. Koda's brief carries the last failure per environment. The frontend splits today's `DeployPanel.svelte` into the list, a details view pushed inside the same sheet, and pure helpers in `$lib/deployView.ts` that the views and tests share.

**Tech Stack:** Rust/axum + sqlx (`nomi-server`, `nomi-agent-core`, `nomi-agent-coding`), Svelte 5 + SvelteKit, paraglide messages, vitest.

**Spec:** `docs/superpowers/specs/2026-10-26-project-sheets-redesign-design.md` (§1 Deployments), mockup `docs/superpowers/specs/assets/2026-10-26-project-sheets/deployments.html`.

## Global Constraints

- Colours, type and shapes only from `frontend/src/lib/styles/material3.css` roles; cards on `--md-sys-color-surface-container-lowest`, things inside on `surface-container-low`; never `#fff`.
- No accent stripes, uppercase eyebrows with icons, glowing/pulsing dots, gradient text or emoji icons (`frontend/CLAUDE.md`).
- Every interactive element: hover + `focus-visible`, 44px target on phones; checked at 390px and in dark mode.
- All copy through paraglide (`frontend/messages/en.json` and every other locale file in that folder).
- Numeric literals get named constants; LLM prompt text lives in `nomi-agent-core/src/prompts.rs`.
- Shipyard step status vocabulary: `succeeded | failed | running | pending | skipped`; phases in order `validate, build, configure, deploy, finalize`.

## Review Focus

1. A project whose newest publish failed: the *older* succeeded one is "Live", and Rollback is not offered on it.
2. Two environments: "live" and rollback are per environment; the env chips filter the list.
3. A rollback target whose source wasn't kept, or that failed: no Rollback button, and the API refuses with a clear reason.
4. A deployment whose steps have no timestamps (pending/skipped): the step log view shows the step's `error` or "no log for this step", never the whole log mislabelled.
5. A running publish: the details view follows it live and lands on the failed step when it fails.

---

### Task 1: Overview tells live, rollback version and who started it

**Files:**
- Create: `backend/migrations/0082_publish_started_by_agent.sql`
- Modify: `backend/crates/nomi-server/src/routes/deploy.rs` (`PublishRow`, `PUBLISH_COLUMNS`, `overview`, `start_publish`, `send_publish`)
- Modify: `backend/crates/nomi-agent-core/src/project_ops.rs` (`ProjectOps::publish`, `publish_from_tool`), callers in `nomi-agent-planning`, `nomi-agent-coding`, `nomi-agent-inspector`, `nomi-server/src/project_ops.rs`
- Test: `backend/crates/nomi-server/tests/deploy_routes.rs`

**Interfaces:**
- Produces: `PublishRow` gains `is_live: bool` (the newest `succeeded` publish of its environment among those returned), `rollback_of_version: Option<i64>` (the `files_version` of the publish it rolled back to), `started_by_agent: Option<String>` (crew display name, e.g. "Koda"). Frontend type `ProjectPublish` gets the same three fields.
- Produces: `ProjectOps::publish(&self, user_id, project_id, environment, agent: Option<&str>)`; `publish_from_tool(ops, user_id, agent: &str, input)`.

- [ ] **Step 1:** Migration: `ALTER TABLE project_publishes ADD COLUMN started_by_agent TEXT;` with a comment saying it's the crew member who called `publish_project`, null when a person pressed Publish.
- [ ] **Step 2: Failing tests** in `deploy_routes.rs`: `overview_marks_the_newest_success_per_environment_live` (production: succeeded v1, failed v2 → v1 `is_live`, v2 not; staging: succeeded v3 → `is_live`), `a_rollback_row_names_the_version_it_returned_to` (insert a publish with `rollback_of` = v1's id → `rollback_of_version == 1`), `an_agent_publish_says_who` (row with `started_by_agent = 'Koda'` comes back with it).
- [ ] **Step 3:** Run `cargo test -p nomi-server --test deploy_routes` → the three fail (unknown fields).
- [ ] **Step 4:** Add the columns to `PUBLISH_COLUMNS` (`LEFT JOIN project_publishes rb ON rb.id = pp.rollback_of` for the version), compute `is_live` in `overview` over the returned rows (first `succeeded` per `environment`, rows are newest first). `publish_row` returns `is_live: false`. `start_publish`/`send_publish` take `agent: Option<&str>` and store it; the HTTP publish route passes `None`. Thread `agent` through `ProjectOps::publish`; agents pass their crew display name.
- [ ] **Step 5:** Run the server tests and `cargo test --workspace` → pass.
- [ ] **Step 6:** Commit "Deploy overview says which publish is live, what a rollback returned to, and who started it".

### Task 2: Rollback refuses what can't work; Koda's brief knows the last failure

**Files:**
- Modify: `backend/crates/nomi-server/src/routes/deploy.rs` (`rollback`)
- Modify: `backend/crates/nomi-agent-coding/src/brief.rs` (`task_brief`)
- Test: `backend/crates/nomi-server/tests/deploy_routes.rs`, `backend/crates/nomi-agent-coding/tests/brief.rs`

**Interfaces:**
- Consumes: Task 1's `PublishRow`.
- Produces: `rollback` answers `409` "only a publish that succeeded can be rolled back to" for a non-`succeeded` target, `410` (existing) when the source wasn't kept, `403` for non-admins (existing).

- [ ] **Step 1: Failing tests:** `rolling_back_to_a_failed_publish_is_refused` (409, no Shipyard call: mock `.expect(0)`); `rolling_back_creates_a_publish_that_returns_to_the_old_source` (wiremock Shipyard accepts; new row has `rollback_of` = target and Shipyard received the target's stored source bytes); in `brief.rs`: `the_brief_names_the_last_failed_publish` (production publishes: succeeded then failed with `failure` phase `build` step `build_image` message "npm run build exited with code 1" → brief contains "The last publish to production failed at build › build_image: npm run build exited with code 1"), `a_later_success_clears_it` (failed then succeeded → no such line).
- [ ] **Step 2:** Run them → fail.
- [ ] **Step 3:** In `rollback`, select `status` with the old row and refuse non-`succeeded` before reading source. In `task_brief`, for each environment take its newest non-running publish; if `failed`, add one line built from `failure` (`phase`, `step`, `message`, falling back to `error`). The line's wording is a const in `brief.rs` next to the brief's other text (it's brief copy, not a system prompt).
- [ ] **Step 4:** Run tests → pass; `cargo test --workspace` → pass.
- [ ] **Step 5:** Commit "Rollback only goes back to a publish that worked; Koda's brief names the last failed publish".

### Task 3: `deployView` helpers

**Files:**
- Create: `frontend/src/lib/deployView.ts`, `frontend/src/lib/deployView.test.ts`
- Modify: `frontend/src/lib/types.ts` (`ProjectPublish` + Task 1 fields; `DeploymentDetail.steps[]` gains `phase: string`, `error?: string | null`)

**Interfaces:**
- Produces:
  - `PHASE_ORDER: readonly ['validate','build','configure','deploy','finalize']`
  - `environmentsOf(publishes: ProjectPublish[]): string[]` (production first, then others alphabetically, unique)
  - `canRollBack(publish: ProjectPublish, role: 'admin' | 'member'): boolean` (admin, `succeeded`, `can_roll_back`, not `is_live`)
  - `groupSteps(steps: DeploymentStep[]): { phase: string; steps: DeploymentStep[] }[]` (in `PHASE_ORDER`, unknown phases after, empty phases dropped)
  - `focusStep(steps: DeploymentStep[]): DeploymentStep | null` (the failed step, else the running one, else the last succeeded)
  - `stepLog(logs: LogLine[], step: DeploymentStep): LogLine[] | null` (lines with `started_at <= at <= finished_at`, `finished_at` open-ended while running; `null` when the step has no `started_at`)
  - `durationLabel(startIso: string, endIso: string | null): string | null` (`< 90s` → `"42s"`, else `"3 min"`; null while running)
  - `normalizeDeployment(raw: unknown, fallbackId: string): DeploymentDetail` (moved from `DeployPanel.svelte`'s inline normalizer, keeping its behaviour, plus `phase`/`error` on steps)

- [ ] **Step 1: Failing tests** for each function with the Review Focus cases: newest-failed keeps `canRollBack(live)=false`; `environmentsOf` order; `groupSteps` on the docs' example (`clone_repo` build succeeded, `build_image` build failed, `deploy` deploy skipped → `[{build:[clone_repo, build_image]},{deploy:[deploy]}]`); `focusStep` picks `build_image`; `stepLog` window and `null` for a skipped step; `durationLabel` 42s / "3 min" / null.
- [ ] **Step 2:** `npx vitest run src/lib/deployView.test.ts` → fail (module missing).
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run → pass.
- [ ] **Step 5:** Commit "Deploy view helpers: environments, rollback rule, steps by phase, a step's log".

### Task 4: The Deployments list

**Files:**
- Modify: `frontend/src/lib/components/project/DeployPanel.svelte` (rewrite the list part; the details part moves to Task 5)
- Create: `frontend/src/lib/components/project/PublishRow.svelte`
- Modify: `frontend/messages/*.json`

**Interfaces:**
- Consumes: Task 3 helpers; `ProjectDeploy` (`overview`, `publish()`, `rollback(id)`, `busy`, `running`, `changes`).
- Produces: `PublishRow.svelte` props `{ publish: ProjectPublish; canRollBack: boolean; onopen: () => void; onrollback: () => void }`. `DeployPanel` keeps prop `{ deploy: ProjectDeploy }` and gains `onaskfix?: (text: string) => void` (passed by the project page; see Task 5).

- [ ] **Step 1:** Rebuild to the mockup: env chips (only when `environmentsOf` > 1; selection filters rows and the Live card), the Live card (live publish of the selected env: "Live since …", address, "Version N · published by …", Open), empty state with Publish when nothing is live, rows in one container with the status square / title + tag (Running, Failed, Live, "Rolled back to version N") / meta line (when · by person or agent · duration or "broke at <phase> › <step>") / the error box for failed rows (`failure.message` or `error`). Publish stays in the sheet's title-bar actions on the project page as today.
- [ ] **Step 2:** Rollback: a row action and the existing confirmation dialog, with the copy "Go back to version N? The app's code returns to version N; its secrets stay as they are now." Shown only when `canRollBack`.
- [ ] **Step 3:** `npm run check` → 0 errors; `npx vitest run` → pass.
- [ ] **Step 4:** Commit "Deployments sheet: live card, every publish with failures in place, rollback on earlier successes".

### Task 5: Publish details

**Files:**
- Create: `frontend/src/lib/components/project/PublishDetails.svelte`
- Modify: `frontend/src/lib/components/project/DeployPanel.svelte` (selected publish → details view with a back button), `frontend/src/routes/(app)/projects/session/[sessionId]/+page.svelte` (pass `onaskfix`), `frontend/messages/*.json`

**Interfaces:**
- Consumes: Task 3 helpers, `GET deploy/publishes/:id`, `GET deploy/deployments/:id?environment=`.
- Produces: `PublishDetails.svelte` props `{ deploy: ProjectDeploy; publish: ProjectPublish; liveVersion: number | null; onback: () => void; onaskfix?: (text: string) => void; onrollback: () => void }`.

- [ ] **Step 1:** Header row inside the sheet body: back button, "Version N", "status · time · environment". Failed: error callout ("Broke at <phase> › <step>", message, "Version M is still live" when `liveVersion`, "Koda has been told in the project chat"), buttons **Ask Koda to fix it** (calls `onaskfix` with the localized fix request; the page sends it through the chat composer's send path) and **Publish again** (admins). Succeeded and not live: **Roll back to this version**.
- [ ] **Step 2:** Service tabs when the publish has several deployments (web first, as today). Steps grouped by phase (`groupSteps`), each with status square, name, duration; the `focusStep` is selected; selecting a step shows `stepLog` (or the step's `error`, or "No log for this step"); a "Whole log" toggle shows everything. Log block dark monospace, error lines in the error colour.
- [ ] **Step 3:** Live follow: keep today's behaviour (poll the open deployment every 3s while running, refetch on `deploy.changes`), and move the selection to the failed step when the publish fails.
- [ ] **Step 4:** `npm run check`, `npx vitest run` → pass.
- [ ] **Step 5:** Commit "Publish details: where it broke, steps by phase, each step's log, ask Koda to fix it".

### Task 6: Check it for real

- [ ] **Step 1:** Run the app locally (backend + frontend) with a project whose publishes are seeded to cover: running, failed at build, failed at deploy with runtime logs, succeeded (live), older succeeded, a rollback. If Shipyard isn't reachable, seed rows and stub Shipyard's deployment endpoint with the docs' example body.
- [ ] **Step 2:** Screenshots at 1280px and 390px, light and dark; fix what looks off against the mockup and `frontend/CLAUDE.md`. Check keyboard focus and 44px targets.
- [ ] **Step 3:** Roll back once against a real Shipyard environment if credentials exist locally; otherwise record that it wasn't possible.
- [ ] **Step 4:** Commit fixes; open the PR with the screenshots.
