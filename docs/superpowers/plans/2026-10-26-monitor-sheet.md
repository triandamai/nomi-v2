# Monitor sheet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Part 2 of the project sheets redesign: Monitor rebuilt to the approved mockup on the data Shipyard gives Nomi today, and a Shipyard to-do list for the rest.

**Architecture:** Frontend only. Pure helpers in `$lib/monitorView.ts` (log levels and filtering, peaks, chart paths, event tones, uptime) with tests; `MonitorPanel.svelte` rebuilt on them, reusing `StatusSquare` from part 1; the project page passes `deploy` so Monitor knows the live services. `docs/shipyard-todo.md` lists what Shipyard must add.

**Tech Stack:** Svelte 5, paraglide, vitest.

**Spec:** `docs/superpowers/specs/2026-10-26-project-sheets-redesign-design.md` (§2 Monitor), mockup `docs/superpowers/specs/assets/2026-10-26-project-sheets/monitor.html`.

## Global Constraints

Same as part 1 (`2026-10-26-deployments-sheet.md`): theme roles only, no AI-looking decoration, hover/focus/44px, 390px and dark checked, all copy through paraglide (en + id), named constants.

## Review Focus

1. A project that was never published: one clear empty state, no polling.
2. Shipyard unreachable or usage failing: the error shows once, the rest of the sheet still works.
3. A log line with "error" inside a word (e.g. "errorless") or in another case ("ERROR"): levels match whole words, any case.
4. Several containers (web + db): one CPU/memory card each, web first.
5. No numbers Shipyard doesn't give: no visitors/requests tiles with values.

## Rulings carried from the spec

- The 1h / 24h / 7d switch and the number tiles wait for Shipyard history; until then one quiet line says where they'll appear. (Spec left it to the builder; anti-goal: no fake numbers.)
- CPU/memory charts cover what the sheet sampled while open, labelled as such.

---

### Task 1: `monitorView` helpers

**Files:** Create `frontend/src/lib/monitorView.ts`, `frontend/src/lib/monitorView.test.ts`.

**Produces:**
- `logLevel(line: string): 'error' | 'warn' | 'info'` (whole words, any case: error, fatal, panic, exception → error; warn, warning → warn)
- `filterLogs(lines: string[], level: 'all' | 'warn' | 'error', query: string): string[]` (warn keeps warn and error; query is case-insensitive substring)
- `peak(values: number[]): number | null`
- `chartPaths(values: number[], width: number, height: number, max?: number): { line: string; area: string } | null` (null under 2 points; `max` defaults to `Math.max(100, ...values)`)
- `eventTone(type: string): 'failed' | 'running' | 'succeeded'` (crashed/failed → failed, resource_alert → running, succeeded → succeeded)
- `liveSince(services: ShipyardService[]): string | null` (the web service's `last_deployment.created_at`, else the first service's)
- `orderContainers(containers: ContainerUsage[]): ContainerUsage[]` (db last)

- [ ] Failing tests for each, with the Review Focus cases → run → implement → pass → commit "Monitor view helpers".

### Task 2: The Monitor sheet

**Files:** Modify `frontend/src/lib/components/project/MonitorPanel.svelte` (rebuild), `frontend/src/routes/(app)/projects/session/[sessionId]/+page.svelte` (pass `deploy`), `frontend/messages/*.json`.

- [ ] Health card: status square, "Running · up since <ago>" (or the service's real state), "<running> of <replicas> running" per service; empty state when unpublished.
- [ ] One quiet line: "Visitors, requests, errors and response time show here once Shipyard reports them."
- [ ] CPU and memory per container (web first): now and peak, area chart from `chartPaths`, "Since you opened this" caption; side by side on desktop, stacked under 600px.
- [ ] Events as rows: `StatusSquare` with `eventTone`, sentence, time; crashes, resource alerts, failed publishes and publishes that went live.
- [ ] Logs: All / Warnings / Errors segmented control, search, Follow live toggle, theme-following mono block (same look as the Deployments log), lines coloured by `logLevel`.
- [ ] `npm run check`, `npx vitest run` → pass; commit "Monitor sheet rebuilt".

### Task 3: Shipyard to-do list

**Files:** Create `docs/shipyard-todo.md`.

- [ ] Per missing metric: what Nomi needs, resolution and range, the API shape Nomi would call, how alerts relate. Commit "Shipyard to-do: what Monitor needs".

### Task 4: Check it for real

- [ ] Run the app locally with a fake Shipyard serving usage, logs and events; screenshots at 1280/390, light/dark; fix what looks off; open the PR stacked on part 1.
