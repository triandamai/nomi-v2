# Sora design agent: N0 architecture discovery

Date: 2026-11-15. Inspected at `feat/tool-catalog` (`3ffa22b2`), which is already merged into
`main`; `main` is 19 commits ahead, none of them touching `backend/` or `frontend/`.
No production code was changed in N0.

## 1. Baseline tests

| Suite | Command | Result |
|---|---|---|
| Backend unit tests (29 crates) | `cargo test --workspace --lib --no-fail-fast` | 422 passed, 0 failed |
| Frontend unit tests (`frontend/`) | `vitest run` | 178 files, 1388 tests passed |
| Backend integration tests (204 files under `crates/*/tests/`) | not run | need Postgres with pgvector; none reachable on this machine |
| Frontend type check, Playwright e2e | not run in N0 | svelte-check last passed on this commit earlier today |

CI (`.github/workflows/backend.yml`) runs the integration tests against `pgvector/pgvector:pg16`
and enforces per-crate coverage floors (`backend/coverage-floors.json`; a new crate gets the
default: 80% lines, 75% functions).

## 2. How an agent exists today

- An agent is a `SubAgent` (`nomi-agent-core/src/subagent.rs`) in its own crate, wired in with one
  line in `nomi_server::build_agent_registry` (`nomi-server/src/lib.rs`).
- Its crew name comes from `nomi-agent-core/src/crew.rs` (`"planning" => "Rena"`,
  `"coding" => "Koda"`), also mapped in `nomi-mail/src/brand.rs` and `frontend/src/lib/crewNames.ts`
  / `crew.ts`.
- Every prompt lives in `nomi-agent-core/src/prompts.rs`.
- Capabilities are trait switches, not code: `asks_questions` (`ask_user`), `supports_plans`
  (`write_plan`), `reads_plans` (`read_plan`), `can_delegate` (`delegate_to_agent`),
  `works_in_background`, `supports_todos`, `uses_memory`, `domain_context` (extra prompt for this
  chat only), `chat_tools` (tools for this chat only), `validate_delegation_task`.
- Tools outside the agent loop implement `Tool` (`tool_catalog.rs`) and are collected into the
  shared `ToolCatalog` (51 tools), searchable with `find_tools`.
- Routing is in `nomi-turn` (`lib.rs`, `routing.rs`, `projects.rs`): a build request always goes to
  Rena in a project chat; publish requests go to Rena or Koda.

**There is no Sora, no `DesignBrief`, no design tables and no design UI anywhere in the repo.**

## 3. Rena today (`nomi-agent-planning`)

- Tools of her own: `create_project`, `publish_project`. Everything else is engine-level.
- Discovery is prompt-only (`PLANNING_APP_PLAYBOOK`): one `ask_user` card, at most five questions,
  "once", including 3-4 look directions. The `ask_user` tool description itself says to put
  everything in one call and never ask again later.
- Her output is a markdown plan saved by `write_plan` into `agent_plans`
  (`0024_agent_plans.sql`: title, content or S3 key, integer `version`, per agent session). It is
  versioned but unstructured: no fields, no fact/assumption distinction.
- Handoff: `delegate_to_agent` with target `coding` and a task starting `Project <uuid>: `.
- Pinned by tests: `nomi-agent-planning/tests/planning_context.rs`, `planning_tools.rs`,
  `nomi-agent-core/tests/engine.rs`, `nomi-turn/tests/turn_project_routing.rs`.

## 4. Koda's contract (`nomi-agent-coding`)

- Accepts a delegation whose task starts with `Project <uuid>:`; runs in the background.
- `brief.rs::task_brief` builds his starting brief: project, file list, the newest plan written
  by the `planning` agent (`plans::latest_plan_by`), stack guides, Relay state, last failed publish.
- A design handoff can be added as one more section of this brief without changing the contract.

## 5. Shipyard (`nomi-shipyard`)

- The client covers orgs, apps, environments, publish (manifest + zipped source), deployments,
  logs, metrics, secrets, domains, webhooks.
- **It has no static artifact call.** A publish is always a Docker build of an app environment;
  the `static` stack is an image running `serve` (`manifest.rs::dockerfile`). Shipyard's public
  docs mention a `static` service kind (nginx, SPA fallback), which Nomi does not use yet.
- Whether Shipyard has, or will get, a "generic static artifact API" could not be confirmed from
  the repo or the docs page. This decides N6/N7.

## 6. Previews and isolation

- Browser runtime: the preview iframe loads `/projects/:id/preview/…` from Nomi's own origin with
  `sandbox="allow-scripts"` (an opaque origin, so scripts cannot read Nomi cookies or storage), via
  an authenticated SvelteKit proxy to `GET /api/projects/:id/preview/*`. No CSP headers are set.
- Sandbox runtime (boat.dev): the iframe loads the sandbox's own URL, a separate origin.
- There is no dedicated preview host, no `postMessage` bridge, no element selection or annotation
  code. Reusable UI: `ProjectPreview.svelte`, `project/PreviewDevices.svelte`,
  `PreviewAddressBar.svelte`, `ProjectVersions.svelte`, `PlanCard.svelte`, `SheetFrame.svelte`.

## 7. Providers

| Capability | Today |
|---|---|
| Web research | Exists: `web_search` (Tavily, Serper, Brave, SearXNG, admin-configured) and `read_web_page` (public hosts only), offered to every agent (`web.rs`) |
| Image input | Exists: `ContentBlock::Media`, routed to a model that accepts it |
| MCP | Storage only: `mcp_servers` table and `/api/connections/mcp`; `MCP_CALLABLE = false`, no client |
| Browser automation, Figma, image generation | Nothing |
| Static artifact deployment | Nothing (see 5) |

## 8. Flags, storage, realtime

- No general feature-flag system. The pattern to reuse: an admin settings section
  (`nomi-settings/src/config.rs`) plus `Rollout` (off, allowlist, percent, all), as the sandbox
  runtime does; and Admin → Tools switches (`tools.rs::ToolSwitches`).
- Files: `nomi_storage::ProjectStore` (S3). Realtime: MQTT envelopes in `nomi-realtime/src/lib.rs`.
- Next migration number: `0146`.

## 9. Where the brief and the repo disagree

1. **Progressive discovery vs. one card.** Rena's playbook and `ask_user` both say "ask once,
   everything in one card". Progressive discovery needs a design-specific playbook that relaxes
   this for design work only, behind a flag, so trips and plain app requests keep today's behaviour.
2. **Shipyard static artifacts** do not exist in the client (see 5).
3. **Separate preview origin** does not exist; today's isolation is an iframe sandbox on Nomi's
   own origin. A separate origin needs a hostname and a route that serves without Nomi cookies.
4. **MCP** has no client; N3 would be building one, not wiring one.
5. **`frontend-new/`** is owned by another team; the Design Studio would be built in `frontend/`.

## 10. Proposed shape for N1 onward (to confirm, not built)

- New crate `nomi-agent-design` (`agent_type` `design`, crew name Sora), prompts in `prompts.rs`,
  its tools as `Tool`s.
- New additive tables from `0146`: design briefs (versioned JSON), directions, immutable artifact
  versions, feedback, approvals, handoffs.
- Provider traits in `nomi-agent-core` following `ProjectOps` / `RelayOps` (`No…` default, server
  implementation, fake for tests).
- One admin settings section `design` with a rollout, gating Sora, Rena's design playbook and the
  Studio route.
- Koda: one optional "approved design" section in `task_brief`.
