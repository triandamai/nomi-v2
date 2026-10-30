# Design-system parity: frontend ↔ frontend-new ↔ backend

Goal: `frontend/` (Material 3) gets the same components as `frontend-new/` (the pen design system).
The two apps behave the same and look different. Where a component needs something the API doesn't
have, the backend changes so both apps can use it.

Approach (agreed 2026-10-30): audit first, then **M3 twins**.
- For every frontend-new component, `frontend/` gets a twin with the same name, the same props and
  the same events.
- The twin is drawn with the M3 tokens in `src/lib/styles/material3.css` and the rules in
  `frontend/CLAUDE.md`.
- The old pages then move onto the twins one section at a time.

## What exists today

| | frontend-new | frontend |
|---|---|---|
| Components | ~183 in `src/lib/ds/<section>/` | ~150 across `components/`, `m3/`, `blocks/`, `views/`, `project/`, … |
| App screens | none yet: only the `/design-system` gallery | 46 pages (chat, projects, money, reminders, memory, admin, …) |
| Styling | Tailwind v4 + pen tokens, class lint | Scoped CSS + M3 tokens (`material3.css`) |
| Component tests | Vitest browser mode (Chromium) | Vitest (node/jsdom), a few component tests |

frontend-new has no flows wired to the API yet, so "same behaviour" means: both apps render the same
backend data through components with the same contract.

## Section map

Status per section:
- **Has**: frontend already does this.
- **Partial**: it does some of it, in a different shape.
- **Missing**: it doesn't do it at all.
- **API gap**: the backend has to change first.

| Section (count) | frontend-new | frontend today | Status | Backend |
|---|---|---|---|---|
| actions (5) | Button, IconButton, Link, Kbd, KeyHint | `m3/Button`, `m3/IconButton`, `m3/SplitButton` | Partial: no Kbd/KeyHint/Link | — |
| inputs (11) | TextField, Checkbox, CheckMark, Toggle, Chip, Search, Segmented/Segment, Tabs/Tab, Composer | `m3/TextField`, `m3/Checkbox`, `m3/Switch`, `m3/Chip`, `m3/ButtonGroup`, `composer/CompactChatInput` | Partial: no Search, Tabs, CheckMark | — |
| overlays (10) | Dialog/DialogPanel, Modal, Menu/MenuPanel/MenuRow, Toast/ToastRegion, Tooltip | `m3/Dialog`, `m3/BottomSheet`, `m3/SideSheet`, `m3/Menu`, `m3/Snackbar` | Partial: no Tooltip, no toast queue | — |
| status (4) | Status, Tag, Meter, SectionLabel | `project/StatusSquare`, `m3/WavyProgress` | Partial | — |
| foundations / agents / identity (7) | Icon, VisuallyHidden, GithubMark, Mascot, Avatar, AgentByline, CrewRow | `icons/`, `m3/Avatar`, `m3/AgentShape`, `CrewPanel` | Partial: no AgentByline/CrewRow shape | — |
| navigation / chrome / shells (12) | NavItem, NavGroupLabel, RecentRow, Sidebar, RecentsList, SettingsNav, UsageCard, UserRow, WorkspaceChip, AppShell, FocusShell, WorkspaceShell | `Sidebar`, `NavList`, `DrawerRecents`, `RecentChats`, `AdminSidebar`, `OrgPicker` | Partial: no SettingsNav, UsageCard, Focus/Workspace shells | — |
| lists (4) | EmptyState, SkeletonRow, ProjectRow, SuggestionRow | `ListToolbar`, `m3/ListItem`, `ProjectPreview`, `StatusScene` | Partial | — |
| conversation (8) | MessageUser, MessageAgent, ActivityBlock, LiveStep, EventRow, FileRow, KVRow, ErrorCallout | `MessageBubble`, `blocks/ActivityDisclosure`, `blocks/FileChangeBlock`, `blocks/ErrorBubble`, `blocks/RunFailedCard` | Partial: MessageUser edit/version needs the API | **Gap 3** |
| chat: rich replies (≈25) | ReplyText, CodeBlock/CodeToken, Diff, DataTable, Sources/Citation, Hyperlink/LinkPreview, Reasoning/Thought/ThoughtSteps, PlanCard/PlanApproval/DraftPlanCard, ApprovalCard, ClarifyCard/Review/Answered, SelectionCard, ChoiceOption, FormCard, DecisionCard, Cart, Reminder, MemorySaved/Used, ConnectPrompt, QuickReplies, FollowUps, Handoff, CrewWorking | `blocks/*` (19) and `views/*` (17) cover every `ContentBlock` and `View` the backend sends | Has, in another shape. The twins take the same data. | — (the block/view types already match) |
| chat: actions on a reply (≈10) | ResponseActions, CopiedNote, DislikeFeedback, FeedbackSent, RegenerateMenu, VersionSwitcher, DeleteReplyDialog, DeletedReply, StopHint, SupervisorStopped | like / dislike with one reason, copy, delete (`MessageDeleteSheet`), "stop" | Partial | **Gaps 1, 2** |
| chat: actions on a chat (≈8) | ChatMenu (rename, pin, move, export, share, delete), RenameField, MoveDialog, ShareDialog/SharePanel | rename + delete (`SessionActions`) | Partial | **Gaps 4, 5, 6** (export is client-side) |
| chat: composer extras (≈12) | AttachMenu, AttachmentChip, AttachmentImage, MentionPicker/MentionCrew/MentionFile, VoiceRecorder/VoiceTranscribed, ComposerLimit, RunOptions, MessageEditor, Banner, SessionExpired | `CompactChatInput`, `MicButton`, `VoiceNoteRecorder`, `MessageAttachments`, `EffortMenu`, `ModelSelector` | Partial: no mention picker | — (crew list from `/api/agents`, files from `/api/projects/:id/files`) |
| chat: thread chrome (≈6) | DateDivider, NewDivider, LoadingOlder, JumpToLatest, Boundary, BackgroundNote | inline in `ChatThread` | Partial | — |
| build (31) | BuildPlan, BuildRunning, BuildDone, CodeView, FileList, Terminal, PreviewBuilding, PhoneFrame, DeviceBar, PublishButton, PublishRequest, RequestQueue, Presence, PeerCursor, Typing, RoleMenu, BuildSharePanel, BuildInvite, ExportPanel, … | `CodeWorkbench`, `project/*` (Deploy, Publish, Members, Monitor, Relay, …) | Partial | **Gaps 7, 8** |
| mobile (25) | Phone variants of composer, cards, nav bar, sheets, toasts | M3 already reflows at 390px | Twins only where the phone layout differs | — |

## Backend gaps

Each gap was checked against the real routes in `backend/crates/nomi-server` on 2026-10-30. They're
also written into `frontend-new/BACKEND_TODO.md` in that file's format.

1. **Dislike reasons and note.**
   - The component sends `{ reasons[], note, includeChat }`, and its reasons are `inaccurate`,
     `ignored`, `wrong_crew`, `too_long`, `outdated`, `unsafe`, `other`.
   - `PUT /api/sessions/:id/messages/:mid/feedback` takes one `reason` from `wrong_memory`,
     `not_relevant`, `too_long`, `other`, and no note.
2. **Regenerate a reply, and its versions.**
   - The components are RegenerateMenu (again, or with another model) and VersionSwitcher (`n / total`).
   - Today `POST …/messages/:mid/retry` only re-runs *failed* runs, and a message has no versions.
3. **Edit a sent message.**
   - The components are MessageEditor and MessageUser (`edited`, `version`).
   - There is no `PATCH …/messages/:mid`.
4. **Pin a chat.** ChatMenu has pin and unpin, but `sessions` has no `pinned` column.
5. **Move a chat into a project.** MoveDialog needs this, but `PATCH /api/sessions/:id` only takes `title`.
6. **Share a chat by link.**
   - SharePanel needs a URL, an expiry and settings (`scope` reply or chat, `mask`, `anyone`).
   - No share routes exist.
7. **Export a project to GitHub.**
   - ExportPanel's push action needs this.
   - Only `POST /api/projects/:id/import/github` exists, and it reads a public zip without a token.
   - It needs a GitHub connection first.
   - Download already works through `GET /api/projects/:id/snapshot`.
8. **Publish requests.**
   - With PublishRequest and RequestQueue, a teammate asks and the owner approves.
   - `POST …/deploy/publish` exists, but there is no request/approve step.

Not gaps:
- Export a chat: built client-side from `GET …/messages`.
- Stop: the supervisor handles "stop" sent as a message.
- Sources: messages already carry `sources`.
- Reminders: done, snooze and cancel all exist.
- Memory used / saved: `…/messages/:mid/memories` and `/api/memory` exist.
- Link previews: `View::Link` is filled server-side.
- Presence and typing exist.

## Phased plan

Each phase is its own PR: twins + tests + the pages moved onto them. The old components a phase
replaces are deleted in that phase.

| Phase | Frontend (M3 twins in `frontend/src/lib/components/ds/<section>/`) | Backend |
|---|---|---|
| 1 ✅ | Primitives: actions, inputs (all but Composer), overlays, status, foundations, agents, identity. Gallery at `/design-system`, gated like frontend-new's. No page uses them yet: pages move onto the primitives as phases 2–6 rebuild their screens. | — |
| 2 | Shell: navigation, chrome, shells, lists. Sidebar, settings nav and the app frame move onto them. | Gaps 4, 5 (pin, move) ✅ |
| 3 (in progress) | Conversation + rich replies, copied from frontend-new as they are (`chat/`, `conversation/`): their token classes point at M3 through `src/lib/styles/ds-twins.css`. Blocks moved onto them so far: suggestions (FollowUps), run failed (ErrorCallout), reminder (Reminder), approval (ApprovalCard), reasoning (Reasoning). Still the old components: questions, plan, table, todo list, views, file changes, voice, connect, project link, quota; and `MessageBubble` itself. | Connect prompt needs `agent_type`, `why` and `reads` on its blocks (BACKEND_TODO). |
| 4 | Reply and chat actions: ResponseActions, DislikeFeedback, Regenerate/Version, MessageEditor, ChatMenu, Move, Share | Gaps 1, 2, 3, 6 |
| 5 | Composer (moved here from phase 1) and its extras: mentions, attach menu, voice, limits | — |
| 6 | Build + mobile: CodeWorkbench and `project/*` move onto the build twins | Gaps 7, 8 |

How a twin is built:
- **Copied as is (from phase 3).** Where a component is only frontend-new's token classes over the
  twins of phases 1–2, it's copied byte for byte and `ds-twins.css` maps the tokens to M3 (colours,
  type scale, shapes; spacing stays). `dsParity.test.ts` keeps `chat/` and `conversation/` identical.
- **Contract.** Copy the frontend-new component's `Props` type verbatim (names, optional
  flags, callback signatures). Its tests are ported too, with only the selectors that depend on styling changed.
- **Pure modules.** `clarify.ts`, `stepState.ts`, `table.ts`, `code.ts`, `mention.ts`,
  `toasts.svelte.ts` and the agent `crew.ts` are copied together with their tests.
  - `frontend/src/lib/dsParity.test.ts` fails when a copy drifts.
  - The same test compares every twin's props (its types and `$props()` defaults) with frontend-new's.
- **Copy.** Strings go through each app's own Paraglide messages, with the same keys where the text is the same.
- **Look.**
  - M3 tokens only, following `frontend/CLAUDE.md`.
  - Hover and `focus-visible` states, and 44px targets on phones.
  - Checked at 390 px and in dark mode.
