# Crew Skills (backend) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A person can add SKILL.md skills (typed, uploaded, or imported from a link) to any crew
member, and the member reads them on demand through `read_skill`.

**Architecture:**
- `nomi-skills` (leaf crate): the format (parse, validate, index lines).
- `nomi-tools-skills`: storage, zip/link import and the `read_skill` tool.
- The engine offers the tool when the member has skills.
- `nomi-server` exposes the routes.

**Tech Stack:** Rust workspace, axum 0.7, sqlx 0.7 (`#[sqlx::test(migrations = "../../migrations")]`), reqwest, zip 2.

**Spec:** `docs/superpowers/specs/2026-11-24-crew-skills-design.md`

## Global Constraints

- Backend only. No changes in `frontend/`, `frontend-new/` or `native/`.
- No AI attribution in commits or the PR. Commits use Conventional Commits.
- Every numeric literal is a named const. LLM-facing text (the tool description) lives in
  `nomi-agent-core/src/prompts.rs` and is passed into the tool crate, as `SpeakCopy` does.
- Limits, verbatim from the spec:
  - `name`: 1–64 chars of `[a-z0-9-]`, starting with a letter.
  - `description`: 20–1024 chars.
  - `SKILL.md` and each reference ≤ 16 000 bytes.
  - ≤ 20 references per skill.
  - ≤ 30 skills per person per member, switched-off ones included.
  - Index line: the name plus the description cut to 150 chars.
  - Zip: ≤ 64 entries, ≤ 512 KB uncompressed.
  - Link import: 10 s timeout, ≤ 21 requests.
- Error codes, exactly:
  - `invalid_skill`, `name_taken`, `too_many_skills`, `too_large`;
  - `unsupported_file`, `not_a_skill`;
  - `fetch_failed`, `private_address`, `rate_limited`, `not_a_link_skill`.
- Build with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`.
- Tests need:
  - `DATABASE_URL=postgres://nomi:nomi@localhost:55432/nomi` and MQTT on 1883;
  - to run sequentially, never two cargo test runs at once.

## Review Focus

1. **Another person's skills:** never listed, offered or readable. Every store query filters by
   `user_id` and `agent_type`. Test in Tasks 2 and 3.
2. **A zip with path tricks:** `../`, absolute paths, nested folders two deep, two `SKILL.md`
   files, or a zip bomb. These are refused or skipped, nothing touches disk, and the size cap
   counts uncompressed bytes as they are read (not the header's claim). Test in Task 4.
3. **A link that redirects to a private address, or a slow or huge response:** refused by
   `ensure_public_host` on every hop, with the timeout and size cap enforced. Test in Task 4.
4. **Built-in members with no skills:** their offered tools are exactly as before. The
   `tool_offer_snapshot` test stays unchanged. Test in Task 3.
5. **Concurrent adds near the cap:** two adds at 29 skills can't both land above 30. Count and
   insert in one transaction that locks the person's row set
   (`SELECT ... FOR UPDATE` on the user's skills for that member, or an advisory lock on
   `(user_id, agent_type)`). Test in Task 2.

---

### Task 1: `nomi-skills`, the format

**Files:**
- Create `backend/crates/nomi-skills/{Cargo.toml,src/lib.rs}`.
- Copy the parser and its tests from `origin/feat/sora-library-v2:backend/crates/nomi-agent-design/src/library.rs`: `Frontmatter`, `unquote`, `frontmatter`, and the frontmatter tests.

**Interfaces (produces):**
```rust
pub const MAX_SKILL_BYTES: usize = 16_000;
pub const MAX_REFERENCES: usize = 20;
pub const MAX_SKILLS_PER_MEMBER: i64 = 30;
pub const MAX_NAME_CHARS: usize = 64;
pub const MIN_DESCRIPTION_CHARS: usize = 20;
pub const MAX_DESCRIPTION_CHARS: usize = 1024;
pub const INDEX_DESCRIPTION_CHARS: usize = 150;
pub struct Frontmatter { pub name: String, pub description: String, pub license: String, pub source: String }
pub fn frontmatter(text: &str) -> Option<(Frontmatter, &str)>;          // #356's, plus BOM/CRLF normalised first
pub enum SkillError { NoFrontmatter, BadName, DescriptionLength, TooLarge, TooManyReferences, BadReferenceName }
pub struct Reference { pub name: String, pub text: String }
pub struct ParsedSkill { pub meta: Frontmatter, pub text: String, pub references: Vec<Reference> }
pub fn validate(text: &str, references: Vec<Reference>) -> Result<ParsedSkill, SkillError>;
pub struct BuiltinSkill { pub name: &'static str, pub text: &'static str, pub references: &'static [(&'static str, &'static str)] }
#[derive(Clone, Debug, PartialEq)] pub struct SkillEntry { pub name: String, pub description: String }
pub fn index_line(entry: &SkillEntry) -> String;                         // "- name: description…" cut at INDEX_DESCRIPTION_CHARS chars
pub fn body(text: &str) -> &str;                                        // text without frontmatter, trimmed
```
`frontmatter` takes `&str` but normalises, so it returns owned data where needed. Pick the
signature that keeps #356's tests passing unchanged.

- [ ] **Step 1: Tests.**
  - #356's frontmatter tests, copied.
  - `crlf_and_bom_parse`.
  - `repeated_key_keeps_last`.
  - `validate` for each rule, just inside and just over: a 64- vs 65-char name; `"1abc"`
    refused; uppercase refused; 19 vs 20 and 1024 vs 1025 description chars; 16 000 vs 16 001
    bytes for SKILL.md and for a reference; 20 vs 21 references; a reference name with `/` or
    `..` refused.
  - `index_line_cuts_on_a_char_boundary`: a multibyte description.
- [ ] **Step 2:** `cargo test -p nomi-skills`. Expected: FAIL, since the items are missing.
- [ ] **Step 3:** Implement. Reference names drop a trailing `.md` and follow the same charset
  rule as `name`.
- [ ] **Step 4:** `cargo test -p nomi-skills`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(skills): nomi-skills, the SKILL.md format and its limits`.

### Task 2: Storage

**Files:**
- `backend/migrations/0167_user_agent_skills.sql`;
- `backend/crates/nomi-tools-skills/{Cargo.toml,src/lib.rs,src/store.rs}`;
- `nomi-tools-skills/tests/store.rs`.

Tables exactly as the spec's Storage section: `origin` has a CHECK constraint
(`typed`/`upload`/`link`), `user_id` references `users(id) ON DELETE CASCADE`, and references
cascade from their skill.

**Interfaces (produces), in `nomi_tools_skills::store`:**
```rust
pub enum Origin { Typed, Upload, Link }
pub struct SkillRow { pub id: Uuid, pub name: String, pub description: String, pub license: String, pub source: String,
    pub text: String, pub enabled: bool, pub origin: Origin, pub source_url: Option<String>, pub reference_names: Vec<String>, pub updated_at: DateTime<Utc> }
pub enum StoreError { NameTaken, TooMany, NotFound, Db(sqlx::Error) }
pub async fn list(conn: &mut PgConnection, user_id: Uuid, agent_type: &str) -> Result<Vec<SkillRow>, sqlx::Error>;          // by name
pub async fn get(conn, user_id, agent_type, id: Uuid) -> Result<Option<(SkillRow, Vec<Reference>)>, sqlx::Error>;
pub async fn find_enabled(conn, user_id, agent_type, name: &str) -> Result<Option<(SkillRow, Vec<Reference>)>, sqlx::Error>;
pub async fn index(conn, user_id, agent_type) -> Result<Vec<SkillEntry>, sqlx::Error>;   // enabled only, by name
pub async fn insert(conn, user_id, agent_type, skill: &ParsedSkill, origin: Origin, source_url: Option<&str>) -> Result<SkillRow, StoreError>;
pub async fn replace(conn, user_id, agent_type, id, skill: &ParsedSkill, source_url: Option<&str>) -> Result<SkillRow, StoreError>; // rename allowed if free
pub async fn set_enabled(conn, user_id, agent_type, id, enabled: bool) -> Result<bool, sqlx::Error>;
pub async fn delete(conn, user_id, agent_type, id) -> Result<bool, sqlx::Error>;
pub async fn delete_member(conn, user_id, agent_type) -> Result<u64, sqlx::Error>;
```
`insert` counts and inserts inside one transaction under
`pg_advisory_xact_lock(hashtext(user_id::text || agent_type))`. A link import's empty `source`
is set to `source_url`.

- [ ] **Step 1: Tests** (`#[sqlx::test]`):
  - `insert_then_list_and_get` (references round-trip);
  - `name_taken_for_same_member_only`: the same name is allowed on another member and for
    another user;
  - `thirty_first_skill_is_too_many`: a switched-off one counts;
  - `parallel_inserts_at_the_cap_land_one`: two tasks at 29, exactly one `TooMany`;
  - `another_users_skills_are_invisible` (list, get, find_enabled, index, delete);
  - `index_skips_disabled`;
  - `replace_renames_when_free_else_name_taken`;
  - `delete_member_removes_all`;
  - `link_source_defaults_to_url`.
- [ ] **Step 2:** Run. Expected: FAIL.
- [ ] **Step 3:** Implement the migration and `store.rs`.
- [ ] **Step 4:** `cargo test -p nomi-tools-skills`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(skills): user_agent_skills storage`.

### Task 3: `read_skill` and engine wiring

**Files:**
- `nomi-tool/src/context.rs`:
  - `OfferContext` gains `pub skills: &'a [nomi_skills::SkillEntry]`;
  - `ToolContext` gains `pub builtin_skills: &'a [nomi_skills::BuiltinSkill]`;
  - `nomi-tool` depends on `nomi-skills`.
- `nomi-tools-skills/src/tools.rs`.
- `nomi-agent-core`:
  - `subagent.rs`: `fn builtin_skills(&self) -> &'static [BuiltinSkill] { &[] }`.
  - `prompts.rs`: `READ_SKILL_TOOL_DESCRIPTION`.
  - `engine/offer.rs`: `OfferInputs.skills: Vec<SkillEntry>`, and register the tool in
    `engine_catalog_tools()` after `nomi_tools_attachments::tools()`.
  - `engine.rs`: load the index once per turn.
  - `engine/tool_batch.rs`: pass `builtin_skills`.
  - `engine/tool_definitions.rs`: set `skills: &[]`.
- Tests:
  - `nomi-tools-skills/tests/read_skill.rs`;
  - `nomi-agent-core/tests/engine_catalog.rs`, which gains tests.

**Interfaces:**
- Consumes Task 1's types and Task 2's `store::{index, find_enabled}`.
- Produces:
  ```rust
  pub const GROUP: &str = "skills";
  pub struct ReadSkillCopy { pub description: &'static str }   // description holds "{index}"
  pub fn tools(copy: ReadSkillCopy) -> Vec<Box<dyn Tool>>;
  pub fn merged_index(builtin: &[BuiltinSkill], mine: Vec<SkillEntry>) -> Vec<SkillEntry>; // mine first; a built-in shadowed by name is dropped
  ```
- The tool:
  - `auto_offer() == Everyone`, `approval() == Never`, `posts_activity() == true`;
  - `offer()` returns `None` when `ctx.skills` is empty, otherwise the description with the index
    lines filled in;
  - input `{name, reference?}`;
  - reads the person's enabled skill first, then `ctx.builtin_skills`;
  - returns the body plus a "More in this skill: a, b" line, or one reference (case-insensitive);
  - errors list the available names.
- The engine passes `merged_index(agent.builtin_skills(), store::index(..))` as
  `OfferInputs.skills`. On a DB error it logs and uses the built-ins only.

- [ ] **Step 1: Tests.**
  - `read_skill.rs` (`#[sqlx::test]`):
    - `not_offered_without_skills`;
    - `offer_lists_mine_before_builtin`;
    - `reads_body_and_names_references`;
    - `reads_one_reference_case_insensitively`;
    - `mine_shadows_builtin_of_same_name`;
    - `unknown_name_lists_available`;
    - `disabled_skill_is_not_read`;
    - `other_users_skill_is_not_read`.
  - `engine_catalog.rs`:
    - `read_skill_offered_when_member_has_a_skill`;
    - `read_skill_not_offered_without_one`.
  - The existing `tool_offer_snapshot` passes unchanged.
- [ ] **Step 2:** Run. Expected: FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test -p nomi-tools-skills -p nomi-agent-core && cargo test -p nomi-server --test tool_offer_snapshot`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(skills): read_skill, offered to a member that has skills`.

### Task 4: Zip and link import

**Files:**
- `nomi-tools-skills/src/import.rs`. Add dependencies on `zip` 2 (deflate), `reqwest` (the
  workspace's features) and `nomi-tools-web`, for `ensure_public_host`.
- `nomi-tools-skills/tests/import.rs`.

**Interfaces (produces):**
```rust
pub const MAX_ZIP_ENTRIES: usize = 64;
pub const MAX_ZIP_BYTES: u64 = 512 * 1024;
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_FETCHES: usize = 21;
pub enum ImportError { UnsupportedFile, NotASkill(String), Invalid(SkillError), TooLarge, FetchFailed(String), PrivateAddress, RateLimited }
pub struct Imported { pub skill: ParsedSkill, pub skipped: Vec<String> }
pub fn from_markdown(bytes: &[u8]) -> Result<Imported, ImportError>;
pub fn from_zip(bytes: &[u8]) -> Result<Imported, ImportError>;
pub async fn from_link(client: &reqwest::Client, url: &str, github_token: Option<&str>) -> Result<Imported, ImportError>;
```
- **Zip:**
  - `SKILL.md` at the root or under exactly one top folder; `references/*.md` beside it.
  - Every other entry goes in `skipped`.
  - Entries with `..`, a leading `/` or a drive letter are refused as `NotASkill`.
  - Bytes are read through `take(remaining budget)`.
- **Link:** parse per the spec's three URL shapes.
  - A folder lists via `api.github.com/repos/{o}/{r}/contents/{path}?ref={ref}` and `.../references`.
  - Files come from `raw.githubusercontent.com`.
  - The token goes only to `api.github.com`.
  - GitHub 403/429 → `RateLimited`.
  - Redirects are followed manually (at most 3), with `ensure_public_host` before each request.
  - Each body is read up to `MAX_SKILL_BYTES + 1`.

- [ ] **Step 1: Tests.**
  - Zip, built in the test with `zip::ZipWriter`:
    - `zip_root_layout`;
    - `zip_one_folder_layout`;
    - `zip_extra_files_skipped`;
    - `zip_two_skill_md_is_not_a_skill`;
    - `zip_parent_path_refused`;
    - `zip_over_512kb_uncompressed_too_large`, a highly compressible entry;
    - `zip_65_entries_too_large`;
    - `markdown_upload`.
  - Link, against a local axum stub with `NOMI_WEB_ALLOW_PRIVATE=1` set only inside these tests.
    Host rewriting is a test-only base-URL override: add
    `from_link_with_bases(client, url, token, api_base, raw_base)` and have `from_link` call it
    with the real hosts.
    - `github_folder_import`;
    - `github_blob_import`;
    - `raw_md_import`;
    - `github_403_is_rate_limited`;
    - `token_sent_only_to_api`;
    - `oversized_file_too_large`.
  - `private_address_refused`, without the flag, on `http://127.0.0.1`.
  - `http_scheme_refused`.
- [ ] **Step 2:** Run. Expected: FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test -p nomi-tools-skills --test import`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(skills): import skills from a zip, a file or a link`.

### Task 5: API routes

**Files:**
- `nomi-server/src/routes/crew_skills.rs` (new);
- `routes/mod.rs` and `app.rs`, for the routes;
- `routes/crew_settings.rs`: `member` becomes `pub(crate)`, and `delete_member` calls
  `store::delete_member` in its transaction;
- `nomi-server/tests/crew_skills_routes.rs`.

Routes and bodies exactly as the spec's API table.
- Errors are `400 {"error": code, "detail": text}`, or `404` for an unknown member or skill.
- `GET .../skills` gives `builtin` from `registry.find(type).builtin_skills()` (empty for dynamic
  members) and `mine` from `store::list`.
- `NOMI_GITHUB_TOKEN` is read from the env at call time.

Ruling: **upload takes the raw body**, like `project_import`, not multipart. nomi-server has no
multipart feature, and one file per request needs none.
- `POST .../skills/upload?filename=<name>`, where `.md` → `from_markdown` and `.zip` →
  `from_zip`. Anything else is `unsupported_file`.
- Body cap: `MAX_ZIP_BYTES`, enforced with `DefaultBodyLimit` on that route.

- [ ] **Step 1: Tests** (`#[sqlx::test]`, using the server test support):
  - `typed_create_list_get_update_delete`;
  - `each_error_code`: one case per code the routes can return;
  - `upload_md_and_zip`, with `skipped` in the reply;
  - `import_and_refresh_link_skill`, via a stub and the env flag;
  - `refresh_on_typed_is_not_a_link_skill`;
  - `unknown_member_404`;
  - `other_users_skill_404`;
  - `builtin_member_lists_empty_builtin`;
  - `deleting_own_member_deletes_its_skills`;
  - `enabled_toggle_hides_from_index`.
- [ ] **Step 2:** Run. Expected: FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test -p nomi-server --test crew_skills_routes`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(skills): crew member skill routes`.

### Task 6: Verify and open the PR

- [ ] **Step 1:** `cargo build --workspace && cargo test --workspace`. Expected: PASS.
- [ ] **Step 2:** `cargo llvm-cov --workspace --no-report`, then the floor check
  (`.github/scripts/llvm-cov-check.py` with `backend/coverage-floors.json`).
  - Add floors for `nomi-skills` and `nomi-tools-skills` one point under what they reach, if
    under the default.
  - Expected: PASS.
- [ ] **Step 3:** Commit `test(skills): coverage floors for the skill crates`.
- [ ] **Step 4:** Push `feat/crew-skills` and open a PR against main.
  - The body lists the error codes for ux-lead, and notes that the frontend-new screens follow
    through `/ship`.
