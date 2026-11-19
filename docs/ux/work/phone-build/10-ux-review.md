---
work: phone-build
gate: 1
round: 1
result: pass
date: 17 Nov 2026
---

# Gate 1 — phone-build

Scope: the six Mobile Build flows at 390, dark and light. Read from `Nomi.pen` (frame trees and
captions, screenshots of `ALFZb`, `wbexi`, `fH3bj`, `bANOq`), the built desktop workspace
(`frontend-new/src/routes/(app)/projects/[id]/`, `src/lib/projects/`, `src/lib/ds/{build,mobile,projects}`),
`frontend-new/AGENTS.md` §9 ("A project's workspace"), `frontend-new/BACKEND_TODO.md` and
`backend/crates/nomi-server/src/app.rs`.

**Verdict: no P0 gaps.** Every mobile step has a dark and a light frame, and the light flows match
the dark ones step for step and string for string (checked on all 52 step pairs). Wherever the
pen leaves a state undrawn, the desktop already has that state, with its copy and logic, and the
phone can reuse it inside a phone sheet or tab. Steps that need data the backend doesn't have are
left out on the phone for the same reason the desktop leaves them out. The contract is in
`40-ux-contract.md`. P1 and P2 gaps are listed below and don't block the build.

## Flows touched
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Steps |
|---|---|---|---|---|---|
| Sign in → live URL | o882RC | hifej | zM0xh | cyXXE | 10 |
| Workspace tools | U16wU | omR64 | tJh5G | zdXZh | 18 |
| Landing page | WUiji | pcnAT | yY8qf | oLJ3q | 5 |
| Roll back, limits & hand-over | pwkEe | Qt5C0 | Wv14s | yYAZV | 4 |
| Code, terminal & phone preview | uR6dq | kKWtu | r3IvZk | f48QD7 | 5 |
| Invite & build together | TnwQT | O0wCuQ | L8dgcS | ZTTgV | 10 |

Light step ids mirror dark in order (for example `ScfY0`↔`l5eYUi`, `ALFZb`↔`LuQRY`, `grv51`↔`AFhli`).
The full map is in the contract.

## Completeness walk (all six, at 390)
| Moment | Covered by (frame id · step) | Gap id |
|---|---|---|
| Entry | Projects › New sheet (built, H4ngT2) or `fH3bj` · 2 (Describe); `PhoneProjectDetail` › Open workspace; the notification `z0NpLc` · 6; `wbexi` · 7 (··· menu, the way into every tool); `yS6ev` · 1 (Chat/Preview switch); `CFzPR` · 9 and `W2Z4Q` · 1 (Share › Build together); `C0kOpT` · 1 and `K7RZh1` · 3 (··· › Code / Terminal); `pefkG` · 8 → `grv51` · 1 (Deployments › Roll back); `?open=` deep links from the card menu (desktop logic) | G2 (two Describe entries), G15 |
| Before commit | `DWymB` · 3 (questions sheet, Skip); `jXfT8` · 4 and `muFvH` · 2 (plan with cost line); `pvAvh` · 3 (publish confirm says what it replaces); `grv51` · 1 (roll back says what comes off); `l94PT` · 17 (take down); `Fbg8O` · 18 (type the name, export first); `OvEGT` · 2 (role in plain words); `jg2Am` · 16 (transfer) | G5 (estimate/allowance not in the API) |
| In progress | `SaEJ7` · 5 (real step and file, Stop, "you can close the app"); `mtpwT` · 9 (ask queue); presence strip `Qw22t` · 5 / `MkB5P` · 6 | G1 (Publishing pill not drawn), G4 (no ETA) |
| Failure | `ALFZb` · 4 (publish failed, live untouched, Fix it, View log) → `lqgZO` · 5; `D11w0` · 4 (failed test, Ask Koda to fix) | G6 (sheet-level errors not drawn on mobile; reuse desktop copy), G7 (sandbox phases on Preview), G3 (first publish fails, no live deploy to fall back on) |
| Result | `bANOq` · 7 (live card); `I8tuK0` · 6 (published toast with Roll back); `J1r3Q` · 2 (rolled back, Restore #2); `JpLGa` · 10 (domain connected); `z1pQC` · 10 (teammate's change ready) | G8 (live/failed card from the chat's deploy event) |
| After | `I6z3D6` · 8 (full-screen preview, Ask bar); `CFzPR` · 9 (share); `bR1XV` · 10 (export); `pefkG` · 8 (deployments, roll back); `M4GVXb` · 11 / `vhsAL` · 5 (monitor); `jyEKQ` · 12 (database); `tweH7`–`Fbg8O` · 13–18 (settings, people, env, transfer, take down, delete); `PsC5F` · 5 (desktop-width preview) | G9 (Change address and add/remove own domain not drawn on mobile), G10 (invite link) |
| Interrupts | "You can close the app" (`SaEJ7` · 5) + notification (`z0NpLc` · 6); a teammate's live presence (`Qw22t`, `GnUU0`, `evgdu`) | G11 (the web gets no lock-screen push), G12 (offline: none on desktop either), G13 (member and viewer bars) |
| Access | Not specified in the pen | Specified in the contract (focus, live regions, 44 pt targets) |

## What the phone reuses from the desktop, as it is
Lift these out of the desktop page script into one shared module (for example
`lib/projects/workspace.svelte.ts`). The desktop and phone layouts should both render from it, and
only one of them is mounted at a time (`phoneView`, AGENTS.md §2 "Layout and screens").
- **Server:** `projects/[id]/+page.server.ts` load and all 20 form actions (`publish`, `rollback`,
  `domain`, `domainAdd/Check/Primary/Remove`, `transfer`, `takedown`, `delete`, `invite`, `role`,
  `remove`, `access`, `secret`, `unsecret`, `github`, `showcase`, `ask`, `answer`), and the `+server`
  endpoints `deploy`, `monitor`, `database(+rows, dump)`, `file`, `export(+summary)`, `transfer`,
  `domain/check`, `queue(+[job])`, `requests`, `presence`, `sandbox/[action]`, `preview/[...path]`.
- **Pure logic, as it is:** `lib/projects/deploy.ts` (`publishState`, `publishing`, `deployRows`),
  `domains.ts` (`domainView`), `monitor.ts` (`monitorStats`, `siteStats`, `sourceRows`, `trafficBars`),
  `database.ts` (`cellText`), `code.ts` (`fileTree`, `firstFile`, `quoteLines`), `terminal.ts`
  (`terminalLines`), `peers.ts` (`distinctInitials`, `peerFor`, `whereLabel`), `asks.ts`
  (`askLine`, `movableAsk`, `openRequests`, `myOpenRequest`, `answeredRequests`, `requestMeta`).
- **Live state, as it is:** `SandboxRuntime` (`sandbox.svelte.ts`), `ProjectLive` (`live.svelte.ts`,
  presence, follow, `onDeployChange`), the 3 s publish poll, the 30 s monitor poll, the queue read
  on chat activity, `settled()` toasts, `?open=` handling.
- **Content, as it is (re-hosted in phone sheets):** `ChatView` (in its phone mode, from the phone
  chat work), `CodeView` lines, `Terminal`, `DnsRecords` values, `SubdomainField`, `SecretRow`,
  `PersonRow`, `EmptyState` with every desktop empty and error state, and all the desktop's
  `m.*` copy for states the mobile pen doesn't draw.
- **Phone components that already exist and match the pen:** `mobile/ProjectNav` (AJXze),
  `WorkspaceSwitch` (LX2ZI), `AskBar` (xWDVb), `BuildPlanCard` (uYQxq), `BuildProgress` (cfa9j),
  `LiveCard` (tJ85O), `DnsRecord` (XuqeO), `TeammateBubble` (w182X), `AskQueue` (R2hTlH),
  `ChangeReady` (fZ4Cj), `SheetHeader`, `SheetRow`, `PhoneToast`, `PhoneButton`, `ErrorCard`;
  `overlays/BottomSheet`, `ActionSheet`, `SheetOption`; `projects/ProjectMoveSheet`,
  `DeleteProjectSheet`. Today these are only in the gallery and docs. This work puts them in the app.

## Steps that need a phone layout (the pen draws them)
Chat/Preview switch in place of the split (`yS6ev`); the ··· tools sheet in place of the bar's
nav (`wbexi`); each tool as a bottom sheet (`pefkG`, `py7VN`, `JpLGa`, `M4GVXb`, `jyEKQ`,
`tweH7`, `OgP6t`, `mAp2k`, `jg2Am`, `Fbg8O`); confirms as action sheets (`pvAvh`, `grv51`,
`l94PT`); Code and Terminal as pushed screens (`C0kOpT`, `G87UAZ`, `K7RZh1`, `D11w0`); the
preview full screen with the Ask bar (`I6z3D6`); Phone/Desktop inside Preview (`PsC5F`); the
presence strip, the who's-here sheet and the follow bar (`Qw22t`, `GnUU0`, `evgdu`); share as a
sheet with app targets (`CFzPR`, `eMzUB`); the in-chat cards: plan, building, live, publish
failed, change ready and asks in line (`jXfT8`, `SaEJ7`, `bANOq`, `ALFZb`, `z1pQC`, `mtpwT`).

## Left out on the phone, as on the desktop (same reason)
| Pen (mobile) | Desktop ref | Why (AGENTS.md §9 / BACKEND_TODO.md) |
|---|---|---|
| `main@9d2e41a` on event lines and Code (`I8tuK0`, `C0kOpT`) | gbVc6 / v0CaT | A publish has no commit. Rows say when and who |
| Publish confirm's file list with +/− (`pvAvh`) | O3nr7x | The backend counts the changes but doesn't list them. The confirm names the count |
| Deploy titles like "Best seller tag + Roti bakar" (`pefkG`) | w6Kep | Not given. `deployRows` title and meta |
| Monitor's PRE-ORDERS Rp and the parsed log columns (`M4GVXb`) | B5zS8Z | The app's own data isn't measured. The log is the app's own lines |
| Database edit in place (desktop has it); the pen says read-only on the phone (`jyEKQ`) | EkRm2 | The phone follows its pen: read only. See D4 |
| Settings › Name (`tweH7`) | xURyC | §9: "no rename". **Stale reason:** `PATCH /api/projects/:id` now exists. See D3 |
| Transfer to a person by email (`jg2Am`) | fubF5 | The backend moves a project to a workspace, not to a person |
| People's INVITED row (`OgP6t`, `E8mu9`) | hRsW7 / NOidU | The project's members don't list open invites |
| Terminal's "run #2 · deploy #2" and per-command times (`K7RZh1`, `D11w0`) | e8oM4 / FXKgN | The terminal is the sandbox's own log. Header: "dev server · sandbox" |
| Code's RECENTLY CHANGED (`C0kOpT`) | v0CaT | The backend doesn't say which file Koda changed last. Tree only |
| Follow's line tag (`evgdu`) | x4ZHo | Presence gives the view and the file, not the line |
| Asks queue "running · 00:21" (`mtpwT`) | abcVH | There's no `started_at` on a running ask. Shows "running" |
| "Sari's ask built · 2 files · tests pass" event (`z1pQC`) | E1pnnH | There's no `AskBuilt` event (BACKEND_TODO) |
| Out of build allowance (`AqQ3n`) | fKAEh | No build allowance in the API (BACKEND_TODO "a build's allowance") |
| Schema change review (`CY7CO`) | f9B9cZ | Not built on the desktop. The backend has no schema-change approval. If Koda asks for a generic approval, the phone chat's approval card shows it |
| Hand-over (the flow header's "ACCEPT TRANSFER → HANDED OVER") | pwkEe header | Neither the desktop nor the mobile pen draws these steps. Transfer is immediate |
| QR and embed under More (`CFzPR`, `eMzUB`) | uDena / gNIpy | No backend. More is the OS share sheet |
| Pointers over the preview | zx5a9 | Touch has no hover pointer, and the preview frame is sandboxed |

## Three lenses
**Product.** The phone story is complete. You can always see where you are: the nav shows the
project, live or not, and the publish state, and the switch shows Chat or Preview. You can see
what comes next: the plan before Build it, and the confirm before Publish, Roll back, Take down
and Delete. You can recover: Publish failed keeps live up and offers Fix it, Roll back offers
Restore, a failed test or start offers Ask Koda to fix. Honesty: "~3 min left" (`SaEJ7`) and
"About 5 min" (`jXfT8`) have no source, so they're left out (G4, G5). "Ask Koda about this
screen" (`I6z3D6`) promises context the app can't read from a sandboxed frame, so it uses the
pen's own "Ask Koda for a change…" from `Qw22t` (G14). "We'll notify you" is true on the web
through the Inbox and email, but not as the lock-screen push the pen draws (G11, D1).

**Business.** Export sits next to Share in the ··· sheet, with the same weight, and is one tap
from Delete (`Fbg8O` "Export a copy first"). It's never harder to reach than upgrading. The cost
line under Build it stays quiet and the pen leads with the plan. Roll back and Take down are
reversible and say so, which keeps support load low. Role labels are plain words, and someone who
can only prompt can't publish (`OvEGT`, `z1pQC`), so a teammate can't put something live by
accident. No new unbounded cost: the phone polls on the desktop's timers (3 s only while a publish
runs, 30 s only while Monitor is open).

**Tech.** Every step maps to an existing endpoint (see the contract's "Exists?" column). What the
backend lacks, already filed in BACKEND_TODO.md: a build's step/ETA on the summary, the build
allowance, a running ask's `started_at`, an `AskBuilt` event, which file Koda changed last, the
line a teammate is on, cancelling a publish, and web push (the backend pushes only to native
iOS/Android through FCM/APNs, `push.rs`). The UI also depends on two things the backend already
exposes that the frontend adapter doesn't read yet: `publishes[].failure`
`{ step, phase, message, tail[] }` on `GET /api/projects/:id/deploy` (for `ALFZb`'s card and View
log), and a project invite's `{ path, expires_at }` from `POST /api/projects/:id/invites` (for
`E8mu9`'s Copy link). The deploy event the chat already gets (`project_event`
`{event:"deploy", ok, environment, text, url}`) can drive the live and failed cards (G8).

## Gaps (none P0)
### P1: build them in this work item, reusing what exists
- **G1 · Publishing and ask states on the nav pill.** `mobile/ProjectNav` has `idle | ready`, and
  the pen also draws `Failed · Retry` (`ALFZb`). Publishing isn't drawn on mobile, and neither are
  the member's Ask/Asked states. Use the desktop's states (`publishState`) and copy:
  `Publishing… 00:24` (`ws_publishing`), `Failed · Retry`, and for a member the icon-only Rocket
  `Ask to publish` / Hourglass `Asked to publish`. AGENTS.md §9 already decided "on a phone the
  bar's ask is its icon".
- **G2 · Two Describe entries.** `fH3bj` ("New build" screen) and the built Projects › New sheet
  (H4ngT2 `dDACS`) both start a build, and the phone has no Build tab. See D2.
- **G3 · The first publish fails.** `ALFZb` assumes deploy #1 is live. With no live deploy the card
  body reads "Publishing stopped at the {step} step. Nothing is live yet." Fix it and View log stay.
  The pill shows `Failed · Retry`.
- **G4 · No ETA.** `SaEJ7`'s "~3 min left" is left out. The line reads `Step 2 of 6 · menu page`.
  `BuildProgress`'s `eta` is passed empty and the separator is dropped.
- **G5 · Plan cost line.** `jXfT8`'s "About 5 min · included in your free plan" shows only what the
  block carries. With no estimate or allowance, the line is left out (same as desktop Deploy's "Uses").
- **G6 · Sheet-level errors and empties (undrawn on mobile).** Monitor couldn't load, Database
  couldn't be read or isn't there, a domain's add/check refused, an invite refused, env save
  refused, transfer with no other workspace, rollback/take down/publish that didn't start,
  Deployments with publishing not set up or not published yet, Domain with no address yet. Use the
  desktop's `m.*` copy (contract copy table) with `EmptyState` or `mobile/ErrorCard` inside the
  sheet. Field errors sit under their field, and action errors go in a `PhoneToast`.
- **G7 · Preview tab states.** Sandbox loading, waking, starting, stopped and failed (with
  `Start again`), "Nothing to preview yet", and "No files yet" in Code. Use the desktop's
  `EmptyState` and copy, centred in the Preview tab. The Ask bar stays.
- **G8 · Live and failed cards from the chat's deploy event.** The desktop doesn't render
  `project_event`. The phone renders `LiveCard` (`bANOq`) for `ok:true` and the Build failed card
  (`ALFZb`) for `ok:false`, joined with `publishes[].failure`. Recommend the desktop follows (Event /
  Deploy `JzFOy`).
- **G9 · Domain actions the desktop has, undrawn on mobile.** Change address (admins), "Use your own
  domain" add field, Check now, Remove domain (asked first). Put them in the Domain sheet with
  `SubdomainField` / `TextField` at full width, `Check records` as drawn, and Remove in an
  `ActionSheet`. Copy from the desktop.
- **G10 · Invite link after sending.** `E8mu9` draws `Invite sent · to {email} · 7 days · Copy link`.
  The backend returns `path` and `expires_at` for someone new. Build it from those. When the person
  was added straight away (`added:true`), the banner reads `{name} added · can prompt` with no link.
- **G13 · Member and viewer on the phone.** A member gets the Ask pill (G1) and Asked opens a sheet
  with `publish_request_mine_*` and `Take it back`. A viewer gets no pill, no Let … go first, and
  read-only sheets. This is the desktop's role logic.
- **G14 · Ask bar copy.** Placeholder `Ask Koda for a change…` (pen `Qw22t`), not "about this
  screen". Tapping it switches to Chat with the composer focused. The mic is left out until the
  phone composer records voice (UXP-001 territory).
- **G15 · Deep links.** `?open=settings|export|share|takedown|code&file=` from the phone card
  sheet opens the same sheet or screen on the phone. The notification link
  (`/projects/:id/app?sheet=deploy` → workspace) opens on Chat, scrolled to the end.

### P2: don't block
- **G11 · Lock-screen push on the web.** `z0NpLc` is only true in the native apps (FCM/APNs). On the
  web the backend's "published" notice goes to the Inbox, and to email unless the person turned
  deploy emails off. See D1. Web Push isn't in the backend (BACKEND_TODO candidate).
- **G12 · Offline in the workspace.** There's none on the desktop either. The phone chat's offline
  banner (`tkzUt` / `i9fRJB`) covers the Chat tab once the phone chat work lands.
- **G16 · Typing and joined note** (`MkB5P` "Sari is typing", `Qw22t` "You joined · you can prompt ·
  Mario publishes"). `POST /api/projects/:id/typing` exists, but the desktop renders neither
  (`Typing`, `JoinedNote` are unused). Build both together on desktop and phone later.
- **G17 · Database dump on the phone.** It isn't drawn. Desktop has `Download dump`. See D4.
- **G18 · Pen slips (numbers).** `bR1XV` · 10: bar "deploy #2" over card "deploy #1". `I8tuK0` · 6:
  "live at 08:42" after the fix at 08:45. `z0NpLc` · 6: lock screen 10:31 for a deploy at 08:18.
  Not copied: the app shows real values.

## Findings on the built desktop (outside this item)
- **F1 · Viewers see Publish.** `projects/[id]/+page.svelte` branches only on `role === 'member'`.
  A `viewer` falls through to the admin's `PublishButton` (Ready / Retry), and the backend refuses
  the publish (`require_admin`, `routes/deploy.rs`). The phone contract gives viewers a status-only
  pill. Recommend the same fix on the desktop.
- **F2 · Stale §9 line.** "The pen's project-name field is left out (no rename: BACKEND_TODO.md)".
  Rename exists now. See D3.

## Decisions for Mario
- **D1 · "We'll notify you" on the web.** Options: (a) keep the pen's copy (true through
  Inbox + email); (b) `You can close the app. We'll email you and put it in your Inbox when it's
  live.`; (c) add Web Push in the backend. **ux-lead recommends (a) now**, and (c) as a
  BACKEND_TODO entry. "Notify" stays true without naming a channel the person may have turned off.
- **D2 · Where a phone build starts.** (a) Projects › New keeps opening the built New project
  sheet, and `fH3bj` isn't built as its own screen; (b) New opens `fH3bj` full screen. **ux-lead
  recommends (a).** The sheet is built, gated and drawn (H4ngT2), and the phone has no Build tab to
  host `fH3bj`. Steps 3 onwards are the same either way.
- **D3 · Rename in Settings.** §9 left the project-name field out for "no rename", but
  `PATCH /api/projects/:id {name}` exists now (BACKEND_TODO "renaming — done"). **ux-lead recommends
  keeping it out of this phone work** for parity with the desktop. Open a small item to add the
  Name row to both, and fix the stale §9 line.
- **D4 · Database on the phone.** (a) Read only, as the phone pen says (`jyEKQ` "Read-only here.
  Ask Koda to change columns."); (b) the desktop's Edit rows and Download dump too. **ux-lead
  recommends (a).** Editing cells at 390 is error-prone, and the pen chose read only. Download dump
  can come later as a sheet row (G17).
- **D5 · Monitor range on the phone.** The pen draws one window, with no range control. (a) Fixed
  `today` (24h, the desktop's default); (b) the desktop's 1h / Today / 7 days as a segmented
  control under the title. **ux-lead recommends (a)**, as drawn, with the subtitle `today`.

## Result
pass. `40-ux-contract.md` is written with `status: verified`, and `flow-registry.md` is updated for the six rows.
Dependency: Chat-tab steps (`DWymB` questions sheet, `jXfT8` plan, approvals) render through the
phone chat work on `feat/frontend-new-v2-phone-chat-flows`. If that hasn't merged, this item shows
the chat in its current phone mode and the cards below still apply.
