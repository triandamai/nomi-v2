---
work: phone-build
status: built
contract: 40-ux-contract.md (verified 17 Nov 2026)
branch: feat/frontend-new-v2-phone-build
---

# Build notes: phone-build

## What was built
Below 768, `/projects/:id` renders the phone's workspace. At 768 and up it renders the desktop as
before. `phoneView` picks one layout, and only that one is in the DOM.

- **One state, two layouts.** `projects/[id]/workspace.svelte.ts` holds the `Workspace` class. It has
  everything the desktop page script had: the deploy overview and its 3 s poll, `settled()` toasts,
  `SandboxRuntime`, `ProjectLive` (presence, follow, `onDeployChange`), the queue and requests, and
  every tool's state and action. The desktop page (`+page.svelte`) reads `ws.*`. Its markup didn't
  change.
- **The phone layout:** `projects/[id]/PhoneWorkspace.svelte`, plus its own logic in `projects/[id]/phone.ts`
  (the publish pill per role, the failed publish's card, the publish that went live while the
  workspace was open, and Code's tree as rows).
- **Adapter:** `PublishRow.failure` (step, phase, message, tail) and the invite's `added`, `path` and
  `expires_at`, passed through the `invite` action (G10).
- **F1 on the desktop:** a viewer sees the publish state only (Up to date or Publishing), never
  Publish or Retry.
- **ChatView (minimal change):** in the workspace's pane at phone width it renders in phone mode
  without its own NavBar. The code quote sits above `PhoneComposer` and is sent ahead of the message.
- **Toasts:** on the phone workspace they show at the top through `PhoneToastRegion`. The layout's
  `ToastRegion` is skipped there, so a toast never shows twice.
- **New ds components** (outside the frozen twin folders). Each has a `ComponentDoc` and an example:
  - `projects/`: `PhoneWorkspaceNav`, `PhoneAskBar`, `PhoneList`, `PhoneListRow`, `PhoneTile`,
    `PhoneNote`, `PresenceStrip`, `PhoneFollowBar`, `PhoneCodeLines`, `PhoneTerminal`,
    `PhoneBuildShareSheet`
  - `overlays/`: `PhoneToastRegion`
- **Twins used as they are:** `mobile/WorkspaceSwitch`, `NavBar`, `SheetRow`, `PhoneButton`,
  `PhoneToast`, `ErrorCard`, `LiveCard`, `DnsRecord`, `AskQueue`, `ChangeReady`.
- **Overlays and project sheets reused:** `BottomSheet`, `ActionSheet`, `SheetOption` and
  `DeleteProjectSheet`.

## How to reach each scenario (running app, a window under 768 or a phone)
| Flow · step | How |
|---|---|
| S1·1 first run | Sign up on a phone: `(auth)/first-run`, unchanged |
| S1·2 start (D2) | Projects › New: the built New project sheet |
| S1·3–5 questions, plan, building | The project's chat on the Chat tab: the phone chat's own cards (the phone-chat-flows branch) |
| S1·7 live card | A publish that goes live while the workspace is open shows `LiveCard` at the end of the thread |
| S1·8 preview | The Preview tab: the sandbox preview, the live app or the static preview, with the Ask bar |
| S1·9 share | ··· › Share (only once live). Live card › Share |
| S1·10 export | ··· › Export, or Delete › Export a copy first. Push to GitHub opens its sheet |
| S2·1–2 switch, changes waiting | Chat / Preview. The Preview tab counts waiting changes. Pill `Publish · n` |
| S2·3 publish | Pill › the Publish sheet (`Publish n changes?`, `as deploy #n · replaces deploy #m`) |
| G1 publishing | Pill `Publishing… m:ss` while a publish runs (3 s poll) |
| S2·4 failed (G3, G8) | The newest production publish failed: pill `Failed · Retry`, the card at the end of the chat, Fix it, View log |
| S2·6 published toast | The toast at the top: `Deploy #n is live` with Roll back |
| S2·7 ··· | Project options. Rows follow the desktop's rules |
| S2·8 deployments | ··· › Deployments. Roll back (admins) › action sheet |
| S2·9–10 domain (G9) | ··· › Domain: records, Check records / Make … primary, Remove (action sheet), Change address, Add |
| S2·11 monitor (D5) | ··· › Monitor: today, read again every 30 s while open |
| S2·12 database (D4) | ··· › Database: read only. The live database once published |
| S2·13–18 settings | ··· › Settings › People / Environment / Visibility / Transfer / Take down / Delete. Closing a sub-sheet returns to Settings |
| S3 site | A static project: ··· has no Database row and says why. Share leads with the card and visibility. Monitor shows visits, form sends and sources |
| S4·1–2 roll back | Deployments › Roll back › `Roll back to deploy #n?` › the toast `Rolled back · deploy #n is live` with Restore |
| S5·1–2 code | ··· › Code: the tree, folders open in place, a file. Long-press a line or press Enter, then Shift+arrows (or tap a second line) › Ask Koda about lines a–b |
| S5·3–4 terminal | ··· › Terminal (sandbox projects only). A failed start shows Ask Koda to fix |
| S5·5 desktop preview | Preview › Desktop: 1440 wide, scaled to fit |
| S6·2–3 invite (G10) | Settings › People › email › Invite › role › Send invite › `Invite sent · to … · n days` · Copy link |
| S6·5–8 together | With a teammate here: the strip, then tap it for In this project, Follow, the follow bar and Stop |
| S6·9 asks | Two asks waiting: `AskQueue` at the end of the thread |
| S6·10 change ready | An admin sees `ChangeReady` with Decline under it. A member's pill is the Ask / Asked icon, and Asked opens the sheet with Take it back |
| G15 deep links | `?open=settings`, `export`, `share`, `takedown`, `code&file=`. The notification's `/projects/:id/app` opens on Chat |

## Tests
- **Unit:** `projects/[id]/phone.test.ts`, 8 tests: the pill for admin, member and viewer; the failed
  publish with and without a live deploy; went live; Code's tree. In `page.server.test.ts`: the
  invite returns its link.
- **Component:** `ds/projects/phoneBuild.svelte.test.ts`, 14 tests: nav geometry and every pill state,
  Ask bar, list rows, tiles never showing 0 for "not measured", strip and follow bar, keyboard line
  selection and wrapping at 390, terminal failure, share targets and visibility, and the phone toast
  region.
- **Real backend:** `e2e/backend/phone-build.e2e.ts`, 8 tests:
  - only the phone layout at 390, the preview at full width, Desktop at 1440, the Ask bar focuses
    the composer, no sideways scroll, Back
  - the rows on ···, Deployments not set up, Export, GitHub
  - Code's tree, a file, and asking about lines picked by keyboard
  - People's invite with Copy link checked on the clipboard, Environment's reason, Transfer's empty
    state, Delete
  - a seeded failed first publish: the card, View log, Fix it
  - live with a later failure: the nav says the live deploy, the card says what's still live, and
    the Share targets
  - `?open=` deep links
  - two phones: the strip, In this project, Follow into Code, Stop
- **Publishing:** Shipyard isn't in e2e, so publishing itself (the pill states and the publish and
  roll back toasts) is covered by the unit and component tests.

## Left out, and why
- **The project's `project_event` chat block:** the live and failed cards come from the deploy
  overview instead. The failed card shows for the newest production publish that failed. The live
  card shows for a publish that went live while the workspace was open. Rendering the block means
  editing the chat's block renderer (ReplyBlocks), which two other branches are changing. See the
  question below.
- **The pen's slips, and anything with nothing behind it** (AGENTS.md §9 and BACKEND_TODO.md):
  - no ETA or plan estimate (G4, G5)
  - no commit hashes or RECENTLY CHANGED
  - no line tag when following
  - no running time on asks
  - no PRE-ORDERS tile
  - no QR or embed
  - no build allowance (`AqQ3n`) and no schema-change review (`CY7CO`)
  - no web lock-screen push (D1 keeps "We'll notify you")
  - no Stop on a running publish
- **Undrawn steps:** typing and the joined note (G16), and the Database dump on the phone (G17).
- **`AskQueue`'s button:** the twin puts it on the ask that's next. The phone offers Let … go first
  only when that ask is a teammate's behind yours. When the movable ask is further back, the phone
  doesn't offer the button and the desktop does.
- **Usage (tokens & cost):** it landed on the desktop's tools from main while this was being built.
  The phone's ··· has no row for it, because the pen draws none.
- **Line selection:** the pen's drag handles are a second tap here, because a native selection
  inside a button list isn't reliable on iOS.
- **Focus:** focus moving to a sheet's title on open, and back to the opener on close, relies on the
  native modal dialog (`showModal`). Each sheet is removed from the page when it closes, so focus
  doesn't always go back to the control that opened it.
- **People:** the pen's inline role per row is a role sheet opened from the role text. Remove is in
  that sheet and asks first.

## Question for ux-lead
1. **The chat's deploy event (G8):** should the live and failed cards render in the thread, where
   the backend's `project_event` message sits, on both desktop and phone? Today they're the thread's
   last item, read from the deploy overview. That needs a small change to the chat's block renderer,
   to coordinate with the phone-chat-flows and chat-files branches.
2. **Usage:** should the phone's ··· get a Usage (tokens & cost) row? Main added the sheet on the
   desktop after the contract, and the pen draws no row for it.
