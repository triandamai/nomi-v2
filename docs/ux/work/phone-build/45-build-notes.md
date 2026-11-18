---
work: phone-build
status: blocked (pen unreadable)
contract: 40-ux-contract.md (verified 17 Nov 2026)
---

# Build notes: phone-build

## Status
Blocked before step 2. On 18 Nov 2026 every Pencil MCP call (`read_skill`, `get_app_state`) answers
that the pen.dev plan's usage limit for this billing period has been reached. The contract's frames
can't be read (`Get`, `ctx.bounds`, `TakeScreenshot`), and AGENTS.md §0/§6 says nothing is built
from the pen without reading it first. No UI code has been written on this branch yet. The plan
below is ready to run as soon as the pen can be read again.

## Build plan

### 0. Shared state (no visual change on the desktop)
- `src/lib/projects/workspace.svelte.ts`: a `Workspace` class built during the page's init with
  `() => data`. It holds everything the desktop page script holds today: the deploy overview and
  its 3 s poll + `settled()` toasts, `SandboxRuntime`, `ProjectLive` (presence, follow,
  `onDeployChange`), the queue and the requests, publish / roll back, domain (rename, own domains),
  monitor (30 s poll while open), database (read), settings (transfer, take down, delete), people,
  environment, showcase, export + GitHub, code (tree, file, quote), the `?open=` handling.
- `projects/[id]/+page.svelte` keeps the desktop markup and reads `ws.*`. It picks the layout with
  `phoneView(page.data.phone)`: `<PhoneWorkspace {ws} />` or the desktop, never both.
- Desktop regression guard: `page.svelte.test.ts`, `workspace.svelte.test.ts` and the existing
  stub/real e2e (`workspace`, `together`, `asks`, `projects-cards`) stay green, unchanged.
- F1: the desktop's publish snippet gives a `viewer` a status-only pill (test first).

### 1. Adapter
- `PublishRow.failure?: { service?, deployment_id?, step?, phase?, message?, tail?: string[], runtime_logs?: string[] } | null`
  (backend `routes/deploy.rs` already sends `pp.failure`).
- `tools.invitePerson` returns `{ added, email, role, path, expires_at }` from the JSON body. The
  `invite` action passes them through (G10). Unit tests in `page.server.test.ts`.

### 2. Phone layout (new, outside the frozen twin folders)
Frozen and used as they are: `ds/mobile/*` (WorkspaceSwitch, AskBar, BuildProgress, LiveCard,
DnsRecord, TeammateBubble, AskQueue, ChangeReady, SheetHeader, SheetRow, PhoneToast, PhoneButton,
ErrorCard). `mobile/ProjectNav` only knows `idle | ready`, and its props can't grow (twin). So G1
gets a new presentational component that draws the same AJXze nav with the full pill machine:
- `ds/projects/PhoneWorkspaceNav.svelte`: back, name (mono) + `live · deploy #N`, pill
  `idle | ready(n) | publishing(elapsed) | failed | ask | asked | status-only`, ···.
- `ds/projects/PublishFailedCard.svelte` (ALFZb): Koda's line, `Build failed · exit {code}`,
  message (mono), `deploy #{n} · {step} step · {elapsed}`, Fix it, View log. G3 body when nothing is live.
- `ds/projects/PresenceStrip.svelte` (Qw22t) and `ds/projects/PhoneFollowBar.svelte` (evgdu).
- `ds/projects/PhoneCodeLines.svelte` (G87UAZ): numbered, wrapped lines; long-press / Enter +
  Shift+arrows to select; the floating `Ask Koda about lines {a}–{b}`.
- `ds/projects/PushedScreen.svelte`: back + title + subtitle for Code, file, Terminal, log.
- `ds/projects/InviteSentBanner.svelte` (E8mu9): `Invite sent · to {email} · {days} days · Copy link`.
- Each gets a `ComponentDoc` in `src/lib/docs/entries/projects.ts` and an example in
  `src/lib/docs/examples/<slug>/01-*.svelte`.
- Route-level composition: `src/lib/projects/PhoneWorkspace.svelte` (not ds; reads `ws`):
  nav, switch, presence strip, Chat tab (`ChatView` in phone mode, `after` = ChangeReady /
  AskQueue / Live and Failed cards), Preview tab (frame, sandbox phases, Phone/Desktop segmented,
  Ask bar), one `BottomSheet` at a time (`sheet` kind; sheet-from-sheet replaces, back returns),
  `ActionSheet`s for publish / roll back / take down / remove domain / remove person / remove
  variable, pushed screens for Code, file, Terminal, log.
- ChatView: one minimal change so the workspace's pane renders in phone mode below 768 without
  the chat's own NavBar (`phone = phoneWidth.current && (!pane || …)`), and the quote sits above
  `PhoneComposer`. Kept as small as possible for the phone-chat-flows and chat-files branches.

### 3. Scenarios → parts → tests
| Flow | Steps built | Parts | Tests |
|---|---|---|---|
| S1 zM0xh/cyXXE | 1 (first run at 390), 3–5 via the phone chat, 7 LiveCard from `project_event`, 8 Preview, 9 Share sheet (wa.me / t.me / mailto / `navigator.share`), 10 Export sheet + GitHub sheet | LiveCard, BuildProgress (eta empty, G4), Share/Export sheets | component: share targets, More hidden without `navigator.share`; e2e: Export from ···, Share hidden before live |
| S2 tJh5G/zdXZh | 1–18 + Publishing (G1) | nav pill, publish ActionSheet/sheet, failed card (G3/G8), toast with Roll back, ··· sheet rows by the desktop's rules (AC6), Deployments, Domain + G9, Monitor fixed today (D5), Database read-only (D4), Settings (no Name, D3), People, Environment, Transfer (ProjectMoveSheet), Take down, Delete (DeleteProjectSheet) | unit: pill state per role; component: every sheet's empty / loading / error copy (AC7); real e2e: ··· rows, Deployments "not set up", People invite + Copy link, Environment add/remove, Transfer, Delete, seeded failed publish → card + View log |
| S3 yY8qf/oLJ3q | 3 (no Database row + note), 4 Share with link card + visibility, 5 site Monitor | same sheets, `siteStats` / `sourceRows` | real e2e: static site → no Database row |
| S4 Wv14s/yYAZV | 1 roll back ActionSheet (k-known and unknown bodies), 2 toast with Restore; 3–4 left out | ActionSheet, PhoneToast | component + stub e2e (Shipyard is stubbed) |
| S5 r3IvZk/f48QD7 | 1 Code tree, 2 file + select lines → quote, 3–4 Terminal + failed start, 5 Phone/Desktop | PushedScreen, PhoneCodeLines, Terminal | component: keyboard selection; real e2e: import a file, ask about lines, quote above the phone composer |
| S6 L8dgcS/ZTTgV | 1–3 invite + banner (G10), 5 strip, 6 TeammateBubble, 7 who's-here sheet, 8 following, 9 AskQueue, 10 ChangeReady + Decline; member/viewer (G13) | PresenceStrip, PhoneFollowBar, InviteSentBanner | real e2e: two contexts at 390 (strip, follow, stop), asks in line, member's Ask / Asked sheet |

Deep links (G15, AC15): `?open=settings|export|share|takedown|code&file=` open the phone sheet or
screen, then the URL is tidied; `/projects/:id/app` lands on Chat.

### 4. Left out (to write into BACKEND_TODO.md and AGENTS.md §9 when built)
The contract's "left out" list: no ETA (G4), no plan cost line (G5), no commit, no RECENTLY
CHANGED, no line tag, no running time on asks, no PRE-ORDERS, no QR/embed, no build allowance,
no schema-change step, no hand-over, typing/joined note (G16), web push (G11). BACKEND_TODO
entries to add or extend: the line a teammate is on, cancelling a publish, web push.

## How to reach each scenario
Not yet: nothing is built.

## Question for ux-lead
The pen can't be read until the pen.dev plan is upgraded or its billing period renews
(Pencil MCP: "usage limit reached", upgrade at pen.dev → farianto231 → settings → billing).
Should the build wait for pen access, or may the engineer build the parts that need no new frame
(the shared state module, the adapter fields, F1 on the desktop, and the phone screens composed
only from the existing `ds/mobile` components) and leave the new sheets' geometry for a follow-up
once the frames can be read?
