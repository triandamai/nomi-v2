# Crew agents audit: Rena, Sora, Koda, Iris

17 Nov 2026 · point 3 of the consolidated requirements · read-only review of `main` (e11135b6)

Status: findings 1 and 2 are fixed in `fix/koda-runtime-knowledge`. The rest are open.

## How the crew is put together

All four agents are `SubAgent` implementations (`nomi-agent-core/src/subagent.rs`), run by the same engine (`engine.rs`, about 106 KB). Each agent supplies:
- a system prompt (all in `prompts.rs`);
- its own tools;
- per-chat context injected each turn.

The engine adds tools every agent shares:
- `ask_user`;
- `read_plan`;
- web search and reading;
- `ask_agent`, `post_crew_note` and the notes board (`crew_talk.rs`);
- `delegate_to_agent` (for agents that can delegate);
- `complete_task`.

| Agent | Crate | Own tools | Context injected per turn | Can delegate |
|---|---|---|---|---|
| **Rena** (planning) | `nomi-agent-planning` | `create_project`, `publish_project`, `save_design_brief` (only while Sora is on) | App playbook (5.3 KB) on app requests; design discovery (3.2 KB) while Sora is on; questions already asked | Yes: to Sora, Koda, Iris |
| **Sora** (design) | `nomi-agent-design` | `propose_directions`, `save_preview`, `read_preview`, `create_handoff`, `review_build`, `search_references`, `list_boards`, connected-tool tools (MCP) | Design brief, last directions, last preview manifest + Studio link, open Studio feedback, approval state | Yes: to Koda after approval |
| **Koda** (coding) | `nomi-agent-coding` | Files (`list/read/search/write/write_files/edit_file`), `add_attachment`, `run_command`, `read_guide`, `publish_project`, Relay tools on Relay projects | A **brief** built once per build: project, files, plan, approved design (handoff and preview files), last failed check, and the stack, preview and relay guides already read | No |
| **Iris** (inspector) | `nomi-agent-inspector` | `app_status`, `read_logs`, `list_tables`, `query_database` (SELECT only), `request_access` | None beyond the task | No |

**The flow:**
1. Rena plans and saves the brief.
2. Sora proposes directions, builds the preview, gets feedback in the Studio, and the person approves.
3. Sora writes the handoff and delegates to Koda.
4. Koda builds.
5. Iris looks into the running app.

Without Sora, Rena hands over straight to Koda.

**The knowledge base** is these pieces:
- **Koda's guides:** 8 Markdown files compiled into the binary (`nomi-agent-coding/guides`, 46 KB). Version pins are current and match each other (SvelteKit 3, Svelte 5.57, Vite 8, Tailwind 4.3, TypeScript 6, React 19.2, Vue 3.5, Astro 7).
- **Rena's playbooks.**
- **The design brief** (`design_brief.rs`).
- **Crew notes.**
- **The person's memory**, which every agent shares.

## Findings

Ordered by what a person would notice.

### Important

1. **Koda's knowledge of the preview describes only the browser runtime.**
   - `guides/preview.md` opens with "The project runs in the user's browser in a WebContainer". Step 4 of his system prompt and `runtime::NOT_OPEN` say the same.
   - But `run_command` already runs in a Boat sandbox for sandbox projects (`runtime.rs:145`), and those projects get the same guide. I haven't checked what share of projects run in a sandbox.
   - **Effect:** Koda may avoid things that work fine in a sandbox (the guide lists what "can't run there"). He may also tell the person to "open the project" when the sandbox would simply wake.
   - **Fix:** give `preview.md` a sandbox section and pick the right one in the brief by runtime. Make `NOT_OPEN` runtime-aware too.

2. **Koda's prompt still tells him to choose a look when Sora already designed one.**
   - Step 5 of `CODING_SYSTEM_PROMPT` says "pick one direction from the design guide, define its colours and fonts".
   - When Sora's design was approved, the brief instead says "the design wins".
   - **Effect:** two instructions pull against each other. On a long build where the brief has been trimmed, the system prompt is what remains.
   - **Fix:** make step 5 conditional, as Rena's playbook is ("when your brief has an approved design, match it; otherwise pick a direction…").

3. **Sora's whole job is one 4.8 KB paragraph, sent on every turn.**
   - Directions, building in pieces, Studio feedback, approval, handoff and review are all always present, whatever phase the chat is in.
   - **Effect:** it costs tokens on every one of her calls (it now shows up on the project's Tokens & cost sheet). It also makes it easy for one rule to drown another.
   - **Fix:** do what Rena already does with `domain_context`. Keep a short core prompt and add each phase's rules only when it applies:
     - preview rules once a direction is picked;
     - feedback rules while feedback is open;
     - handoff rules once a version is approved.

4. **Prompt text lives outside `prompts.rs`, against the repo rule.**
   - `nomi-agent-design/src/lib.rs`: `BRIEF_INTRO`, `DIRECTIONS_INTRO`, `PREVIEW_INTRO`, `FEEDBACK_INTRO`, `NOT_APPROVED`.
   - `nomi-agent-design/src/tools.rs`: `DRAFT_KEPT`.
   - `nomi-agent-coding/src/brief.rs`: `design_section` and the other brief headings.
   - `nomi-agent-planning/src/discovery.rs`: `ALREADY_ASKED_HEADING`.
   - `nomi-agent-core/src/design_review.rs`: `NOT_CHECKED`.
   - `nomi-agent-coding/src/runtime.rs`: `NOT_OPEN`.
   - **Effect:** anyone tuning what the crew is told can't see it all in one place, and the "only in prompts.rs" check can't hold.
   - **Fix:** move them into `prompts.rs`; it's mechanical.

### Minor

5. **Iris is told "Rena publishes it", but Koda can publish too** (`publish_project` is on both). The wording holds today, but it will drift if publishing changes. Name the tool rather than the agent.
6. **Iris has no way to hand a fix to Koda.**
   - Her answers end with "Koda should…", and the person has to relay it.
   - Letting her delegate a fix, with the person's OK, would close the loop that Rena's playbook already sends her into.
7. **`engine.rs` is about 106 KB in one file.** It works, but it's the place a crew change is most likely to break something unseen. Splitting out the cut-off retries, the trimming and the tool dispatch would make reviews tractable.
8. **Koda's guides total 46 KB, and the brief puts the stack, preview and relay guides in front of him at the start.**
   - On a Relay project the brief carries about 26 KB of guides before any project files.
   - That's worth checking against the project token figures now that they're visible.

### Checked and sound
- **Point 8's fix.** The cut-off retry advice is per agent (`cut_off_hint`), and Sora builds in pieces.
- **Point 6.** Element-level feedback runs end to end: you pick an element in the Studio, it's stored with its `data-design-id`, Sora sees it, and her next version marks it resolved.
- **Approval.** No agent can approve a design; only the person can, in the Studio. The approval route starts Sora's handoff.
- **Iris is read-only.** `query_database` is SELECT-only, and contents reach her only after an approved `request_access`.
- **Version pins.** The guides agree with each other and with Koda's prompt.

## Suggested order
1. Koda's preview knowledge (1) and the step 5 conflict (2). Each is small, and both change what gets built.
2. Split Sora's prompt by phase (3), then measure the per-project token difference.
3. Move prompt text into `prompts.rs` (4).
4. Minors as convenient.
