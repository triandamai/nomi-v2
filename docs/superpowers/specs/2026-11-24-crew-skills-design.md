# Crew skills — design

Date: 2026-11-24
Scope: project 3 of 3. Branch `feat/crew-skills`, from `main`.

## Intent

Every crew member starts from a base: its soul.md, its tools and its built-in skills. A person
can grow any member past that base by adding skills of their own, typed, uploaded or imported
from a link. The member "evolves" for that person and keeps everything it started with.

A skill follows the Agent Skills convention that PR #356 brings to Sora's library: a `SKILL.md`
with frontmatter (`name`, `description`, `license`, `source`) and a body, plus optional
`references/*.md`. The member sees a short index of its skills and reads a skill's text only when
it needs it, so fifty skills cost almost nothing per message.

## Goals

1. A person can add skills to any crew member they can open (built-in, an admin's dynamic agent,
   or one of their own), in three ways:
   - **Typed** in the app;
   - **Uploaded** as a `SKILL.md` or a `.zip` of a skill folder;
   - **Imported** from a link (a GitHub folder or file, or any `https` URL to a `.md`).
2. The member uses those skills in chat: an index in its tools, and `read_skill` to read one.
3. Built-in skills (compiled into a member, like Sora's library) appear next to the person's own,
   read-only. Where names clash, the person's skill wins for that person.
4. The web app (`frontend-new`) shows a member's skills and its soul.md, and lets the person
   manage both. frontend-new has no soul.md editor yet; the backend for it exists.

## Non-goals

- Sharing skills between people, or a skill marketplace.
- Versions of a skill. soul.md keeps versions; a skill is replaced in place. Link imports keep
  their source, so "update from source" is the way back.
- Skills granting tools. `@tool` in soul.md grants a tool; in a skill it means nothing.
- The old web app (`frontend/`, being deprecated) and the native app.
- Moving Sora's library onto the shared parser. That follows once PR #356 merges.

## Anti-goals

- A member's instructions growing with every skill. Only the index goes into each turn, and the
  index is capped.
- An imported link reaching the server's own network. Imports use the same public-host guard as
  `read_web_page`.
- A link import changing a member silently. The text is copied at import time; only "update from
  source" fetches again.

## Approaches

- **A. Skills in Postgres, read through one shared `read_skill` tool (chosen).** Each added skill
  is a row with its references. Built-in skills stay compiled into their member. One tool crate
  serves both. Fits PR #356's format and project 1's tool crates; nothing new to run.
- **B. Skill files in S3, index in Postgres.** Suits big reference sets, but adds uploads,
  cleanup and a second store that can be down. PR #356's 16 KB limits keep skills small, so it
  buys nothing here.
- **C. Append skills to soul.md.** Every turn would carry every skill in full, which is what
  read-on-demand exists to avoid. Ruled out.

**Chosen: A** (user's pick, 2026-11-24).

## Design

### The shared format: `nomi-skills` (new crate)

The format, parsed and checked in one place:

- `Frontmatter { name, description, license, source }`, and `parse(text) -> Result<(Frontmatter,
  body), SkillError>`. Parsing matches PR #356's: a leading `---` block, `key: value` lines,
  quoted values unquoted, unknown keys ignored.
- **The parser is PR #356's, copied with its tests** from `feat/sora-library-v2`
  (`nomi-agent-design/src/library.rs`), so Sora's library can adopt `nomi-skills` unchanged later.
  - It also accepts CRLF line endings and a leading BOM, normalised before parsing.
  - Values are single-line; a repeated key keeps the last value.
  - The folder name is ignored: the frontmatter `name` is the skill's name.
  - *(builder's-check rulings, my call)*
- `validate` enforces the limits below and names what failed (`SkillError` has one variant per
  rule, so the API can say which).
- Index formatting for the tool (see `read_skill`).

Limits (each a named const):

| Rule | Value |
|---|---|
| `name` | 1–64 chars: lowercase letters, digits, `-`; starts with a letter (Agent Skills rule) |
| `description` | 20–1024 chars (PR #356's rule) |
| `license`, `source` | optional for a person's skill; a link import sets `source` to its URL when empty |
| `SKILL.md` size | ≤ 16 000 bytes (PR #356's `MAX_SKILL_BYTES`) |
| each reference | ≤ 16 000 bytes, a `.md` name of 1–64 chars, same charset as `name` |
| references per skill | ≤ 20 |
| a person's skills per member | ≤ 30, switched-off ones included |
| index line | name plus the description cut to 150 chars |

### Storage (migration)

- `user_agent_skills`: `id uuid pk`, `user_id`, `agent_type text`, `name`, `description`,
  `license`, `source`, `body text` (the whole SKILL.md as written), `enabled bool default true`,
  `origin text` (`typed` | `upload` | `link`), `source_url text null`, `created_at`, `updated_at`.
  Unique `(user_id, agent_type, name)`. `user_id` cascades on user delete.
- `user_agent_skill_references`: `skill_id` (cascade), `name`, `text`. Unique `(skill_id, name)`.
- `agent_type` is the member's key, as soul.md and permissions use it. Deleting a person's own
  member (`delete_member`) deletes its skills in the same transaction.

### Built-in skills

- `SubAgent::builtin_skills(&self) -> &'static [BuiltinSkill]`, default empty. `BuiltinSkill`
  carries name, `SKILL.md` text and references, all `&'static str`.
- No member on `main` has built-in skills yet. Sora's library plugs in after PR #356 merges.

### The `read_skill` tool: `nomi-tools-skills` (new crate)

- A `Tool` impl, `group = "skills"`, `approval() == Never`.
- **Offered** only when the member has at least one skill for this person (built-in, or the
  person's enabled ones). `OfferContext` gains `skills: Vec<SkillEntry>` (name, description),
  loaded by the engine for (user, member) once per turn, the way `reminder_targets` is.
- **Description:** what the tool does, then the index, one line per skill, the person's first.
  With the caps above the index stays under about 5 KB.
- **Input:** `{ "name": string, "reference"?: string }`.
- **Execute:** finds the skill by name for `ctx.user_id` and `ctx.agent_type`. A person's enabled
  skill is used before a built-in one of the same name. Returns:
  - the body without frontmatter, then the names of its references;
  - or one reference whole (name matched case-insensitively).
  - An unknown name or reference returns an error that lists what exists.
- **Where it applies:** wherever the member runs: chat, background, scheduled and delegated
  turns, since they share the engine.
- **Passing built-in skills in:** the tool crate sees only `ToolContext`, so built-in skills reach
  it as `nomi-skills` types. The skills index travels in `OfferInputs`, because `offer.rs` reads
  no database.
- **Activity:** it posts a step, "Read skill <name>", so the person can see which skills a
  member used.
- Granted generic tools and `find_tools` work as for the other tool crates. Engine wiring follows
  project 1: registered in `engine_catalog_tools()`.

### API (`nomi-server`, `routes/crew_skills.rs`)

All under the member routes, for members the person can see (the same check soul.md uses):

| Route | Does |
|---|---|
| `GET /api/agents/:type/skills` | `{ builtin: [{name, description, references}], mine: [{id, name, description, license, source, enabled, origin, source_url, references, updated_at}] }` |
| `GET /api/agents/:type/skills/:id` | one skill whole: `text` (the SKILL.md) and `references: [{name, text}]` |
| `POST /api/agents/:type/skills` | typed: `{ text, references?: [{name, text}] }` |
| `POST /api/agents/:type/skills/upload` | multipart `file`: a `.md` or a `.zip` |
| `POST /api/agents/:type/skills/import` | `{ url }` |
| `PUT /api/agents/:type/skills/:id` | `{ text?, references?, enabled? }`. A rename is allowed when the new name is free |
| `POST /api/agents/:type/skills/:id/refresh` | fetches a link skill again and replaces it. If the name changed at the source, the skill is renamed when the new name is free; otherwise `name_taken` |
| `DELETE /api/agents/:type/skills/:id` | removes it |

- Writes answer with the saved skill, or `400 { error: <code>, detail }`. Codes: `invalid_skill`
  (with the failed rule), `name_taken`, `too_many_skills`, `too_large`, `unsupported_file`,
  `not_a_skill`, `fetch_failed`, `private_address`, `rate_limited`, `not_a_link_skill`. `404` for an unknown member
  or skill.
- **Zip upload:** read in memory, never written to disk.
  - It holds one skill: `SKILL.md` at the root or inside one top folder, and `references/*.md`
    beside it.
  - Other files are skipped and named in the reply (`skipped: [...]`).
  - Caps: ≤ 64 entries, ≤ 512 KB uncompressed in all, each entry within the limits above.
  - Uses the workspace's `zip` 2 crate.
- **Link import:**
  - `https://github.com/<owner>/<repo>/tree/<ref>/<path>` is a folder.
    - It is listed with two GitHub contents API calls, one for the folder and one for
      `references/`. The files themselves come from `raw.githubusercontent.com`, which isn't
      rate-limited the same way.
    - Unauthenticated, that is about 30 folder imports an hour per server.
    - If `NOMI_GITHUB_TOKEN` is set, it is sent to `api.github.com` only and never logged. Without
      it, imports still work.
    - A GitHub 403 or 429 returns `rate_limited`.
    - *(my call)*
  - `https://github.com/<owner>/<repo>/blob/<ref>/<path>` and `raw.githubusercontent.com` URLs are
    one file.
  - Any other `https` URL to a `.md` file is one file.
  - Safety:
    - `https` only;
    - `nomi_tools_web::ensure_public_host` before every request, and on redirects;
    - a 10 s timeout;
    - the zip caps on total size;
    - at most 21 requests per import.
  - `source_url` keeps what the person pasted; `source` defaults to it.
- An imported or uploaded skill whose name is taken returns `name_taken`. The UI then offers to
  replace that skill (a `PUT`) or cancel.

### Web app (`frontend-new`)

On Settings › Crew, the member page gets:

- **A Skills tab** with:
  - the built-in skills (read-only, can be opened to read);
  - the person's own skills, each with its origin (typed, uploaded, or the link's host), an
    on/off switch, edit, delete, and "update from source" for link skills;
  - "Add skill", with three choices: write (an editor that starts from a SKILL.md template),
    upload, from link.
- **A soul.md tab:** the editor and its versions with restore, using the existing soul routes.
- Five states each: empty, loading, error, populated, mobile. Error copy per API error code.

This is user-facing, so it goes through the repo's UX pipeline (`/ship`):

1. `ux-lead` checks the flow against the pen;
2. `pen-designer` draws the missing frames in `Nomi.pen`;
3. `ui-engineer` builds the verified contract.

The pipeline needs the Pencil MCP tools.

### Order of work

1. **Backend PR** (this branch): `nomi-skills`, migration, `nomi-tools-skills`, engine wiring,
   API, tests.
   - Every error code it adds is listed for `ux-lead`, so its copy is settled in step 2.
   - The implementation plan covers this PR only.
2. **Web PR**, after the backend merges (frontend-new is now the app to build in, said
   2026-11-24; `frontend/` is being deprecated) (frontend-new uses the API exactly as it is): the screens
   through `/ship`.

## Testing

- **`nomi-skills`** (unit): parsing edge cases, as in PR #356: quotes, a colon inside a value, a
  missing or unclosed block. Plus every limit, both just inside and just over.
- **`nomi-tools-skills`:**
  - offered only when the member has a skill;
  - the index lists the person's skills before built-in ones and leaves out disabled ones;
  - a read returns the body and reference names, and reads one reference;
  - a person's skill shadows a built-in one of the same name;
  - unknown name or reference errors list what exists;
  - another person's skills are never visible.
- **API** (`#[sqlx::test]`):
  - each route's success path and each error code;
  - the zip layouts: root, one folder, extra files skipped, too big, two `SKILL.md`s;
  - link imports against a local stub server, with `NOMI_WEB_ALLOW_PRIVATE=1` in the test only;
  - a private-address URL refused without the flag;
  - refresh replaces the skill;
  - deleting a member deletes its skills.
- **Engine:** a turn where the member has a skill offers `read_skill`, and a turn where it has
  none doesn't. The offer snapshot for built-in members without skills is unchanged.
- Coverage floors: the new crates get floors one point under what they reach.

## Left to the builder

- Module layout inside the new crates, and the exact text of the tool description (it lives in
  `nomi-agent-core/src/prompts.rs` with the other prompts).
- Whether `OfferContext.skills` is loaded in `engine.rs` or `offer.rs`, as long as it is one query
  per turn.
- Struct names not fixed above.
