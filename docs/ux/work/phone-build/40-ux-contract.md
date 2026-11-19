---
work: phone-build
status: verified
verified_by: ux-lead
verified_on: 17 Nov 2026
proposal: none (no P0 gaps; P1/P2 in 10-ux-review.md)
---

# UX contract: phone-build

The engineer builds exactly this. Anything not here is out of scope or a question for ux-lead.
The pen is the source for every drawn step (layout, spacing, tokens, copy). This contract covers
four things the pen doesn't: which data feeds each element, which steps the phone leaves out,
what the undrawn states say (desktop copy, reused), and access.

## 0. Ground rules
- **Below 768 the workspace is the phone screen**, not the desktop squeezed. `phoneView` picks one
  layout, and only that one is mounted. There are no hidden copies.
- **One state, two layouts.** Move the desktop page's state and actions (`projects/[id]/+page.svelte`
  script) into a shared module. The desktop and the phone both read it. Don't fork logic into the
  phone. The list of what's reused as it is lives in `10-ux-review.md` ("What the phone reuses").
- **Screen anatomy (all steps except Code/Terminal screens and the email):** `Mobile / Status Bar`,
  `Mobile / Project Nav` AJXze (back · project in mono + `live · deploy #N` · publish pill · ···),
  `Mobile / Workspace Switch` LX2ZI (Chat | Preview, Preview carries the waiting-change count),
  then the Chat tab (the project's chat in phone mode with the phone composer) or the Preview tab
  (the frame, full width, `Mobile / Ask Bar` xWDVb floating). There's no tab bar under the workspace
  (`phoneScreen`).
- **Tools open as `BottomSheet`s** over the current tab, in the desktop's order. Confirms that can
  be undone are `ActionSheet`s. Code and Terminal are pushed screens with their own back.
- **Targets are 44 pt (`pointer-coarse:`) and phone body text is 16 pt.** Mono is used for project
  names, addresses, paths, deploy numbers, DNS values and logs.
- **Numbers are real.** The pen's example values (`deploy #2`, `08:42`, `128`) are placeholders
  for the API's values. Pen number slips (G18) aren't copied.

## 1. Scenarios
Columns: mobile dark / light step · desktop counterpart (dark step; desktop light flow in the
heading) · what the phone shows · source.

### S1 · Sign in → live URL: zM0xh / cyXXE · desktop o882RC / hifej
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | ScfY0 / l5eYUi | TUMox | First run: `What should we call you?`, the one-line promise, name field focused (keyboard up), `Continue` | Reuse `(auth)/first-run` with `FirstRun`. At 390 it must match ScfY0: one column, field above the keyboard |
| 2 | fH3bj / a4iT5 | Cs7I8 | Per **D2 (a)**: the build starts from Projects › New (built `New project` sheet, H4ngT2). `fH3bj` isn't a separate screen | If Mario picks D2 (b), build fH3bj with the `ProjectComposer` and the three starters |
| 3 | DWymB / uXXo1 | p7FEW | Koda's questions as the phone chat's question sheet: `Before Koda builds` · `1 of 3`, 64 pt rows, recommended preselected, `Skip` · `Next` | Rendered by the phone chat (GCvfD qfsQ7). Not reimplemented here |
| 4 | jXfT8 / J4mUx | qyp1R | Plan card (`Mobile / Build Plan` uYQxq): steps, outcome line, `Build it`, `Ask before it goes live`, `Keep planning` | Steps from the plan block. The cost/time line only from what the block carries (G5). Run choice behaviour is the phone chat's (GCvfD KCocj) |
| 5 | SaEJ7 / g61qB | r3Qmzd | `BuildProgress` cfa9j: `Building {project}`, elapsed `01:12` (mono, ticking), the real step `writing src/routes/menu/+page.svelte` (from the chat's live step), `Step {n} of {total} · {step label}` (from the plan checklist: done count + 1 of total), close-app line, `Stop` | No ETA (G4). Stop is the chat's Stop. Composer hint `Koda is building · you can close the app` |
| 6 | z0NpLc / YhPj0 | — | Not a screen in the web app. The backend's `published` notice (Inbox + email) links to the workspace | OS push only in native apps (G11, D1) |
| 7 | bANOq / vrcyK | gbVc6 | Workspace on Chat. The nav says `live · deploy #1` with the `Up to date` pill. `LiveCard` tJ85O after the deploy event: `deploy #1 · 08:18`, `Live`, the address (Copy), `Open`, `Share`, summary line | Card from the chat's `project_event` deploy `ok:true` + `url` (G8). Open opens the live address in a new tab. Share opens S1·9 |
| 8 | I6z3D6 / osMP2 | gbVc6 (preview pane) | Preview tab: the frame full width, Ask bar floating | Frame = sandbox preview (sandbox projects) else live URL else static preview, as on desktop. Ask bar copy per G14 |
| 9 | CFzPR / qI3bl | uDena | Share sheet: `Share {project}` · `anyone with the link can open it`, address + `Copy`, row `Build together` · `Invite people to prompt Koda with you.`, app row `WhatsApp` `Telegram` `Mail` `More` | Only once live (as desktop). Build together opens People (S6·2). App links: `https://wa.me/?text=`, `https://t.me/share/url?url=`, `mailto:?body=`. More calls `navigator.share`, and is hidden when it's not supported. No QR |
| 10 | bR1XV / K1dyPq | chXEk | Export sheet from ···: `Export {project}` · `included on Free · no fee`; Repo (`{n} files · git history`), Database dump (table names), Env var names (`names only`); `Push to GitHub` and `Download zip` equal weight | Reuse the desktop Export (`export/summary`, `github` action, `export/zip`). Push to GitHub opens a sheet with the desktop's repo fields |

### S2 · Workspace tools: tJh5G / zdXZh · desktop U16wU / omR64
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | yS6ev / QCKHX | aDwmt | The Chat/Preview switch replaces the split. Preview full screen, Ask bar floating | There's no resize on the phone. The switch is a radio group |
| 2 | fYIj3 / RbHR6 | NpRMd | After Koda changes the preview: pill `Publish · {n}` (mint), Preview tab badge `{n}`, Koda's reply in chat | `n` = `unpublished_changes` (else desktop fallback in `publishState`) |
| 3 | pvAvh / EqBYK | O3nr7x | Publish sheet: `Publish {n} changes?` · `as deploy #{next} · replaces deploy #{live}`, body `Customers see it on their next visit. Roll back from Deployments any time.`, `Publish deploy #{next}` | There's no file list (count only). First publish uses `deploy_confirm_first`. A retry uses `deploy_confirm_retry` |
| — | (undrawn) | mv7qO | Pill `Publishing… {elapsed}` while the publish runs. Polls every 3 s | G1 |
| 4 | ALFZb / LuQRY | n2bnhd | Pill `Failed · Retry`. Chat card: Koda's line, `Build failed` · `exit {code}`, the failing tool's label + message (mono), `deploy #{n} · {step} step · {elapsed}`, `Fix it`, `View log` | Card from `project_event` `ok:false` + `publishes[].failure` (G8). Fix it quotes `failure.message` into the composer (as the desktop's `askToFix`) and sends `Fix it`. View log opens a pushed screen with `failure.tail` in the Terminal component. With no live deploy: G3 copy |
| 5 | lqgZO / GM1a1 | cMHJu | Pill back to `Publish · {n}`. Koda's reply with the changed files | Chat as is |
| 6 | I8tuK0 / qkUHu | m3FsPu | Pill `Up to date`, nav `live · deploy #{n}`. `PhoneToast` success `Deploy #{n} is live` · detail `{files} files · {time}` · action `Roll back` | Toast from `settled()`. The action rolls back to the previous succeeded publish |
| 7 | wbexi / ogktj | (bar nav) | ··· sheet: header `{project}` · `{kind} · live · deploy #{n}`; rows `Deployments` (count), `Domain` (`free address` or the primary domain), `Monitor`, `Database` (`{n} tables`), `Settings`; `UNDER THE HOOD` `Code` (`read-only`), `Terminal` (`what Koda ran`); footer `Share`, `Export` | The rows appear exactly when the desktop shows that nav item: Deployments/Monitor when there's a deploy overview, Domain when there's an address, Database when the project has one, Terminal only where the sandbox runs, Share only once live |
| 8 | pefkG / ePIRn | w6Kep | Deployments sheet: `Deployments` · `{project} · {n} deploys`; rows from `deployRows` (number, title, when · files, state chip `Live` / `Failed` / `Publishing`), `Roll back` on a rollback-able row; foot note `Rolling back publishes the older build as a new deploy. Your database is not rolled back.` | Empty: `deploy_none_*`. Not set up: `deploy_unavailable_*` |
| 9 | py7VN / c3gGe9 | szaN7 | Domain sheet: `Domain` · `free address works now`; the app's address row `Live`; own domain card with `SET THESE AT YOUR REGISTRAR`, one `DnsRecord` per record (type, `Name: {name}`, `waiting`/`found`, value with Copy), `Check records` | Records, status and checks from the custom domains API. Plus G9: `Change address` (admins), add field, `Remove domain` |
| 10 | JpLGa / XUcRW | LmuGl | `{domain} connected`, records `found`, `HTTPS issued · both addresses serve deploy #{n}`, `Make {domain} primary` | HTTPS stays "on its way" until the hosting can serve the name (§9). Make primary only once every record is found, and only for admins |
| 11 | M4GVXb / XwQaO | B5zS8Z | Monitor sheet: `Monitor` · `today`; 2×2 tiles from `monitorStats` (Requests, Errors with share, p95, Visitors), each with what it counts; `LOG` with the app's last lines (time + line, mono) | D5 (a): fixed 24h, polls every 30 s while open. "Not measured" never shows as 0. No PRE-ORDERS tile |
| 12 | jyEKQ / hjtob | EkRm2 | Database sheet: `Database` · `postgres · {n} tables`; table chips; the first rows as phone rows; foot `Read-only here. Ask Koda to change columns.` | D4 (a): read only. Source Live once published, else Development (desktop rule) |
| 13 | tweH7 / UHyJI | xURyC | Settings sheet: `People` (`{n} people`), `Environment` (`{n} variables`); `OWNERSHIP`: `Transfer ownership`, `Take down`, `Delete project` (danger) | D3: no Name row. Visibility row → the desktop's showcase toggle (Public / Unlisted) |
| 14 | OgP6t / ctBsE | hRsW7 | People sheet: email field + role + `Invite`; rows (initials, name, `you`, role); note | Roles and note from the desktop (`people_role_*`, `people_note`). Admins change a role with a `SheetOption` sheet and remove with an `ActionSheet`. `Who can open it` (`people_access_*`) as a row |
| 15 | mAp2k / KBJj0 | NJtAe | Environment sheet: `Environment` · `{n} variables · applies on next Publish`; names only (`SecretRow`), `Add variable` | The add form opens in the sheet (name, value). Values are never shown. Remove asks first |
| 16 | jg2Am / KhzpB | fubF5 | Transfer: the built `ProjectMoveSheet` (workspace cards), the project's name typed to confirm, `Transfer` | The backend moves to a workspace. Copy from `transfer_*`. Empty: `transfer_none` |
| 17 | l94PT / GjY31 | a0Eys | `ActionSheet`: `Take {project} down?` · `Customers will see 'This page is not available'. Code, database and deploys stay. Put it back online any time.` · `Take down` (danger) · `Cancel` | Toast `{name} is offline`. Only while live |
| 18 | Fbg8O / LVHJc | c3DzFx | The built `DeleteProjectSheet`: `Delete {project}?` · `this can't be undone`, body, `Export a copy first`, name field, `Delete forever` (disabled until the name matches) | Mismatch: `delete_name_mismatch`. On success go to Projects with the deleted note |

### S3 · Landing page: yY8qf / oLJ3q · desktop WUiji / pcnAT
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | jd1aW / I5uWk | F61wR | Same start as S1·2 (D2) with a site starter | — |
| 2 | muFvH / bGpTX | fgTCS | Plan card, five steps, `A live site you can send to clients today.` | As S1·4 |
| 3 | W5Pdjg / caT3u | VcyLD | ··· sheet header `{project}` · `one-page site · live`; no Database row; note `No Database: this site doesn't store data.` | Database row absent when `data.database` is empty (desktop rule) |
| 4 | eMzUB / mrweT | gNIpy | Share sheet led by the link card (title, description, address), `public · shows in search` / unlisted, address + `Copy` | Card = the project's name and description (desktop). Visibility = showcase toggle. More = OS share. No QR or embed |
| 5 | vhsAL / ucTxi | YVZth | Monitor for a site: `VISITS` today, `FORM SENDS` with their share, `WHERE VISITS CAME FROM` rows with share | `siteStats` + `sourceRows`. "Not reported yet" when null |

### S4 · Roll back, limits & hand-over: Wv14s / yYAZV · desktop pwkEe / Qt5C0
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | grv51 / AFhli | E1Agxo | `ActionSheet` from Deployments › Roll back: `Roll back to deploy #{n}?` · `The {k} changes from deploy #{live} come off the live site; #{live} keeps them so you can restore it. Your database stays as it is.` · `Roll back to #{n}` · `Cancel` | When the change count isn't known, the body is `deploy #{live} comes off the live site; #{live} keeps its changes so you can restore it. Your database stays as it is.` |
| 2 | J1r3Q / m2f18 | M9rnhH | Nav `live · deploy #{new}`. Event line `deploy #{new} · rolled back to #{n} by {name}`. `PhoneToast` `Rolled back · deploy #{new} is live` · `same build as #{n} · {time}` · `Restore #{prev}` | Restore = roll back to the previous publish (same action) |
| 3 | AqQ3n / qw9nF | fKAEh | **Left out** (no build allowance in the API). A run that hits the monthly limit mid-chat shows the phone chat's quota card (AyrHG), with `Use my own key` | — |
| 4 | CY7CO / jEsMA | f9B9cZ | **Left out** as a dedicated step. A generic approval from Koda renders as the phone chat's `Mobile / Approval Card` MOzBb | — |

### S5 · Code, terminal & phone preview: r3IvZk / f48QD7 · desktop uR6dq / kKWtu
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | C0kOpT / DkhtA | v0CaT | Pushed screen `Code` (back), the file tree from `fileTree` as 44 pt rows (folders expand), foot `Read-only. Ask Koda to change anything.` | No commit and no RECENTLY CHANGED. Empty: `workspace_no_files_*` |
| 2 | G87UAZ / JH3mz | QMRI2 | File screen `{file}` · `read-only`, numbered lines (wrap, never scroll sideways). Long-press a line, then drag handles to extend. One floating button `Ask Koda about lines {a}–{b}` | `quoteLines` → quote above the phone composer, switches to Chat. Binary: `code_binary`. Loading: `code_reading` |
| 3 | K7RZh1 / gwCos | e8oM4 | Pushed screen `Terminal` · `dev server · sandbox`, the sandbox log (`terminalLines`), wrapped | Only where the sandbox runs. Empty: `sandbox_terminal_empty` |
| 4 | D11w0 / VNFFB | FXKgN | A failed start: red lines, foot summary `The preview didn't start` · `Terminal shows what went wrong.` · `Ask Koda to fix` | `askToFix()` as on desktop |
| 5 | PsC5F / aBElA | bvZyd | Preview tab `Phone` / `Desktop` segmented. Desktop renders the frame 1440 wide scaled to fit, note `1440 px wide, scaled to fit · pinch to zoom` | Phone is the default. Page zoom is allowed (viewport has no `user-scalable=no`) |

### S6 · Invite & build together: L8dgcS / ZTTgV · desktop TnwQT / O0wCuQ
| # | Mobile D / L | Desktop | Phone shows | Notes |
|---|---|---|---|---|
| 1 | W2Z4Q / MwgiT | FyNIH | Share sheet's `Build together` row | Opens step 2 |
| 2 | OvEGT / aiGhh | yl9G8 | Invite sheet: `Invite to {project}` · `they join this project's chat`; email field (keyboard up); role rows `Can build` · `Prompt Koda, edit and publish` / `Can prompt` · `Ask Koda for changes; you publish` / `Can view` · `Open the preview and code, no changes`; `Send invite` | Roles map to `build` / `prompt` / `view` (the desktop's invite action). Email error: `people_email_needed`. Admins only |
| 3 | E8mu9 / jRJNf | NOidU | People sheet with banner `Invite sent` · `to {email} · {days} days` · `Copy link` | G10: from the invite's `path` + `expires_at`. If `added:true`: `{email} invited` toast, no link. No INVITED row |
| 4 | d9xBO / F0SR5H | BLMmX | The backend's email. `Join and build` → `/invite/{code}` (built) | Not app UI |
| 5 | Qw22t / WQ0xV | lA0dI | Presence strip under the switch: teammate's initials in their colour + `{name} is here · {where}` | Shows only while someone else is here. `where` from `whereLabel`. Joined note left out (G16) |
| 6 | MkB5P / QpZ3B | OZ1rT | `TeammateBubble` w182X: initials, name, `{time} · to Koda`, their colour. Koda answers by name | From `ChatView` teammate messages. Typing left out (G16) |
| 7 | GnUU0 / vJsq4 | zx5a9 | Tap the strip → sheet `In this project` · `{n} people · live now`; each person with `Viewing {where}` and `Follow`; you last with `you · {where}` | `PresenceMenu` data |
| 8 | evgdu / dSnab | x4ZHo | Following: bar on top `Following {name} · {where}` · `Stop`, in their colour; your view goes where they go, file by file | Stops when you move on your own or they leave. No line tag |
| 9 | mtpwT / P8c40I | abcVH | `AskQueue` R2hTlH at the end of the thread when more than one ask is waiting or running: `{n} asks in line`, each ask with who and `running` / `next`, `One change at a time, so edits never collide.`, `Let {name} go first` | `askLine` / `movableAsk`. The button only for a teammate's ask behind yours. No running time |
| 10 | z1pQC / NGjzH | E1pnnH | Admin's phone: pill `Publish · {n}`; `ChangeReady` fZ4Cj `{name}'s change is ready` · meta from `requestMeta` · `Publish` · `Review in preview`; `Decline` under the card | Review in preview switches to Preview. Member: Ask pill per G1/G13 |

## 2. State machine
**Workspace view:** `chat` (default) ⇄ `preview`, by the switch, Ask bar (→ chat), `Review in
preview` (→ preview), `Ask Koda about lines` / `Fix it` / `Ask Koda to fix` (→ chat with quote).
Pushed screens `code` → `code/file` and `terminal` (and `log`) sit on top, and back returns to the
last tab. Following overrides the view until stopped.

**Publish pill** (from `publishState` + role):
| State | Admin pill | Member | Viewer | Trigger |
|---|---|---|---|---|
| off (no Shipyard) | none | Rocket `Ask to publish` | none | `available:false` |
| idle | `Up to date` | `Up to date` | `Up to date` | no waiting changes |
| ready | `Publish · {n}` (mint) → S2·3 | Rocket icon `Ask to publish` | none | `unpublished_changes > 0` |
| publishing | `Publishing… {mm:ss}` | same | same | newest production publish `running` or `backend_setup` |
| failed | `Failed · Retry` → S2·3 (retry copy) | Rocket icon | `Failed` (no action) | newest production publish `failed` |
| asked (member) | — | Hourglass `Asked to publish` → mine sheet | — | `myOpenRequest` |
Viewer: the pill is a status with no action. Publishing needs `require_admin` in
`routes/deploy.rs`. Today the desktop shows a viewer the admin's pill, and the backend refuses the
publish. The phone doesn't copy that (finding F1 in `10-ux-review.md`).
Polling: every 3 s while publishing. `onDeployChange` from the project link re-reads. When a
publish settles: success toast (with Roll back) or error toast with the reason, and the chat card (G8).

**Sandbox Preview:** `loading` → `waking` → `starting` → `running` (frame) | `stopped` | `failed`
(both with `Start again`). Started on open, kept awake while open, stopped on leave (`SandboxRuntime`).

**Sheets:** one at a time. Opening a sheet from inside a sheet replaces it (Settings → People /
Environment / Transfer / Take down / Delete, and back returns to Settings). Escape, the grabber
swipe and the scrim all close it.

**Follow:** `none` → `following(id)` by Follow; → `none` by Stop, your own move, or their leaving.

## 3. Copy table (verbatim)
Drawn steps use the pen's strings exactly as shown in section 1. Undrawn states reuse these
existing `messages/en.json` keys (Indonesian follows from `id.json`):
| Key | Text | Where |
|---|---|---|
| `ws_up_to_date` / `ws_publish` / `ws_publishing` | Up to date / Publish / Publishing… {elapsed} | Nav pill (phone form: `Publish · {n}`, `Failed · Retry` per pen) |
| `publish_ask` / `publish_asked` | Ask to publish / Asked to publish | Member pill (icon + aria-label) |
| `publish_request_mine_title` / `_body` / `publish_request_keep` / `publish_request_cancel` | You asked to publish / The project’s admins see your request in the chat. Publishing sends what’s in the preview live. / Keep waiting / Take it back | Member's request sheet |
| `deploy_confirm` / `deploy_confirm_one` / `deploy_confirm_retry` / `deploy_confirm_first` | Publish {count} changes? / Publish 1 change? / Publish again? / Your app goes live as deploy #{number}, at its own address. | Publish sheet variants |
| `deploy_publish_as` | Publish deploy #{number} | Publish sheet button |
| `deploy_live` / `deploy_roll_back_to` | Deploy #{number} is live / Roll back to #{number} | Toast + action |
| `deploy_failed` / `deploy_failed_body` / `deploy_couldnt_start` | Deploy #{number} failed / The live app keeps running. Ask Koda in the chat to fix it. / That didn’t start | Error toasts |
| (new, G3) | Publishing stopped at the {step} step. Nothing is live yet. | Failed card, first publish |
| `deploy_none_title` / `_body` | Not published yet / Publish sends what’s in the preview live, as deploy #1. Every deploy shows here. | Deployments empty |
| `deploy_unavailable_title` / `_body` | Publishing isn’t set up here / Whoever runs this Nomi hasn’t connected a place to publish apps yet. Your project keeps building in the preview. | Deployments, no Shipyard |
| `domain_none_title` / `_body` | No address yet / The app gets its own address on its first publish; you can change its name from here then. | Domain empty |
| `domain_change` / `domain_field` / `domain_checking` / `domain_available` / `domain_taken` / `domain_save` / `domain_rename_note` | Change address / Address / Checking… / Available / That name isn’t free. / Use this address / The old address stops working once the new one is live. Links you’ve shared will need the new one. | Domain › Change address (G9) |
| `monitor_unpublished_title` / `_body` | Nothing running yet / Once the app is published, its traffic, errors, speed and what it prints show here. | Monitor empty |
| `monitor_failed_title` / `_body` | The numbers couldn’t load / The app keeps running. They load again in 30 seconds. | Monitor error |
| `monitor_not_measured` / `monitor_log_empty` | Not measured / The app hasn’t printed anything yet. | Tiles, log |
| `database_loading` / `database_none_title` / `database_failed_title` / `database_failed_body` | Reading the tables… / No database here / The database couldn’t be read / Try again in a moment. | Database states |
| `database_read_only` | Read-only here. To change columns or fix rows, ask Koda in the chat. | Not used: phone uses the pen's `Read-only here. Ask Koda to change columns.` |
| `people_email_needed` / `people_invited` | Enter an email address. / {email} invited | People |
| `people_access_org` / `people_access_members` | Everyone in {workspace} / Only people added | Who can open it |
| `env_add` / `env_note` / `env_saved` | Add variable / Values are encrypted and never shown again, never exported, and never sent to the chat. They apply on the next publish. / {name} saved · applies on the next publish | Environment |
| `transfer_title` / `transfer_body` / `transfer_none` / `transfer_send` | Transfer {name}? / The workspace you pick gets the code, database, deploys and the address. Its people can work on it from then on. / No other workspace to move it to. Create or join one first. / Transfer | Transfer sheet |
| `takedown_done` | {name} is offline | Toast |
| `delete_name_mismatch` | Type the project’s name exactly to confirm. | Delete |
| `sandbox_loading_title` / `sandbox_waking_title` / `sandbox_waking_body` | Looking at the sandbox / Waking the sandbox / The sandbox runs the app as Koda builds it. Waking it takes a few seconds. | Preview |
| `sandbox_starting_title` / `_body` | Starting the app / The sandbox is installing packages and starting the app. Terminal shows each step. | Preview |
| `sandbox_stopped_title` / `_body` | The sandbox stopped / It stops when the workspace has been closed for a while. Start it again to see your changes. | Preview |
| `sandbox_failed_title` / `_body` / `sandbox_start_again` | The preview didn’t start / Terminal shows what went wrong. / Start again | Preview, Terminal foot |
| `sandbox_terminal_run` / `sandbox_terminal_empty` | dev server · sandbox / Nothing has run yet. | Terminal |
| `workspace_nothing_title` / `workspace_nothing_body` / `workspace_nothing_building` | Nothing to preview yet / The app shows here once Koda has built it and it’s published. / Koda is building it. The app shows here once it’s published. | Preview, no frame |
| `workspace_no_files_title` / `_body` | No files yet / Koda writes the project’s files as it builds. Ask in the chat to get it started. | Code |
| `code_binary` / `code_reading` | not text · it can’t be shown here / reading… | File screen |
| `publish_request_decline` / `publish_request_approved` / `publish_request_declined` | Decline / {name} published your change / {name} declined to publish your change | Requests |
| `follow_stop` | Stop following | Follow bar aria-label (visible `Stop`) |
| `presence_follow` / `presence_you` | Follow / {name} (you) | Who's here |
| (pen `Qw22t`) | Ask Koda for a change… | Ask bar (G14) |

## 4. Data and events the UI depends on
| Need | Source (API / event / config) | Exists? |
|---|---|---|
| Project, files, stack, status, org | `GET /api/projects/:id` via `+page.server.ts` load | Yes |
| Publish state, deploy rows, live number, unpublished count, role | `GET /api/projects/:id/deploy` (`publishes[]`, `unpublished_changes`, `role`, `available`, `backend_setup`) | Yes |
| Publish failure detail (step, message, log tail) | `publishes[].failure` `{service, deployment_id, step, phase, message, tail[], runtime_logs[]}` | Yes (backend). Adapter type to add |
| Publish / roll back | `POST …/deploy/publish`, `POST …/deploy/publishes/:id/rollback` | Yes |
| Live/failed card in chat | chat block `project_event` `{event:"deploy", ok, environment, text, url}` + Koda's chat text | Yes (backend). Not rendered on desktop (G8) |
| Changes to deploys, requests, domains | `GET /api/projects/:id/ws` (`onDeployChange`, `domain.*`) | Yes |
| App address + rename + check | `GET|PUT …/domain`, `GET …/domain/check` | Yes |
| Own domains, records, check, primary, remove | `…/domains`, `…/domains/:host/check`, `…/primary`, `DELETE` | Yes. HTTPS stays `pending` (Shipyard) |
| Monitor numbers, site stats, log | `…/deploy/metrics`, `…/deploy/usage`, `…/deploy/logs` | Yes |
| Database tables and rows (read) | `…/database`, `…/database/tables`, `…/database/query` | Yes |
| People, roles, access, invite (+ link) | `…/members`, `…/invites` (→ `path`, `expires_at`), `PUT …/access`, member PUT/DELETE | Yes. Link not used on desktop yet (G10) |
| Env names, set, remove | `…/secrets` | Yes |
| Transfer targets, transfer | `…/transfer/orgs`, `POST …/transfer` | Yes (immediate, to a workspace) |
| Take down / delete | `DELETE …/deploy/environments/:env`, `DELETE /api/projects/:id` | Yes |
| Export summary, zip, GitHub | `…/export`, `…/export/zip`, `…/export/github` | Yes |
| Showcase (Public/Unlisted) | `GET|PUT …/showcase` | Yes |
| Sandbox phase, preview URL, log | `…/sandbox`, `/start`, `/stop`, `/logs` | Yes |
| File content | frontend `projects/[id]/file` → backend files | Yes |
| Presence, where, follow | `…/presence` (GET/POST/DELETE), project stream | Yes |
| Teammate line position | — | **No** (left out) |
| Typing | `POST …/typing` | Yes (send). Not rendered anywhere yet (G16, P2) |
| Ask queue, let first | `GET /api/sessions/:id/queue`, `POST …/queue/:job/first` | Yes. **No** `started_at` |
| Publish requests | `GET|POST …/deploy/requests`, `POST …/requests/:id` | Yes |
| "Built in preview · tests pass" event | `AskBuilt` | **No** (BACKEND_TODO) |
| Plan steps, done count | `agent-plans` checklist (PlanBlock) | Yes |
| Real step / file Koda is on | chat live step (agent activity) | Yes |
| Build ETA, time/cost estimate on the plan | — | **No** (G4, G5) |
| Build allowance | `GET /api/me/plan` allowance | **No** (BACKEND_TODO) |
| Which file Koda changed last | — | **No** |
| Commit per publish | — | **No** |
| Live notification | `notify_admins("published")` → Inbox + email (pref, default on) + native push | Yes. **No** web push (G11) |
| Phone vs desktop layout | `phoneView` (`lib/phone.svelte.ts`), server UA guess | Yes |

## 5. Accessibility
- **Focus order on the workspace:** Back → project name (heading, not focusable) → publish pill →
  ··· → Chat/Preview radio group (arrow keys move it) → presence strip (when shown) → tab content →
  composer (Chat) or Ask bar (Preview).
- **Sheets:** on open, focus moves to the sheet's title. Focus is trapped while open, and Escape
  closes. On close, focus returns to the control that opened it. A sheet opened from a sheet
  returns focus to the row that opened it.
- **Action sheets:** focus starts on `Cancel`, never on the destructive action.
- **Pushed screens (Code, Terminal, log):** focus starts on the back button. Back returns focus to
  the ··· row or card that opened it.
- **Code lines:** each line is a button with `code_line` ("Line {n}"). Selection works without a long
  press too: Enter on a line starts it, Shift+arrows extend it, and the Ask button follows the
  selection in reading order.
- **Live regions (polite):** publish state changes (`Publishing…`, `Deploy #{n} is live`, `Deploy #{n}
  failed`), sandbox phase changes, `{name} is here` / `{name} left`, `Following {name} · {where}` /
  `Stopped following`, queue changes (`{n} asks in line`), request answers. **Assertive:** a
  failed publish and a failed sandbox start.
- **Labels:** the pill as an icon has `aria-label` (`Ask to publish`, `Asked to publish`). The switch
  says `Preview, {n} changes not yet published`. Copy buttons say `Copy {value}` (`dns_copy`) and
  announce `Copied`.
- **Targets and type:** every target is at least 44×44 pt, and phone text is at least 14 pt (body 16 pt).
  Respect `prefers-reduced-motion` for sheet slides and the building elapsed tick (it updates each
  second; announce only step changes, never the seconds).

## 6. Acceptance criteria
- [ ] AC1 At 390 (and anything under 768) `/projects/:id` renders the phone workspace and no desktop
      split. At 1512 it renders the desktop as before. Only one is in the DOM.
- [ ] AC2 Every drawn step in section 1 matches its mobile frame in dark and light (screenshots
      against `zM0xh`/`cyXXE`, `tJh5G`/`zdXZh`, `yY8qf`/`oLJ3q`, `Wv14s`/`yYAZV`,
      `r3IvZk`/`f48QD7`, `L8dgcS`/`ZTTgV`), except the items marked "left out".
- [ ] AC3 The pill follows the state machine for admin, member and viewer. Publishing shows
      elapsed time ticking. A publish that settles shows the toast within one poll (≤ 3 s).
- [ ] AC4 Publish → confirm → success shows `Deploy #{n} is live` with Roll back. Roll back →
      action sheet → deploy #{n+1} live, toast with Restore. The database isn't touched.
- [ ] AC5 A failed publish leaves the live deploy serving, shows `Failed · Retry`, and the chat
      card shows the backend's `failure.message`. Fix it puts the error in the composer. View log
      shows `failure.tail`. With no live deploy the card says `Nothing is live yet.`
- [ ] AC6 The ··· sheet lists only the tools the desktop would show for that project (no Database
      for a site, no Terminal without a sandbox, no Share before live).
- [ ] AC7 Each sheet shows its empty, loading and error states with the copy in section 3. No sheet
      ever shows a blank body or a 0 for something that wasn't measured.
- [ ] AC8 Domain: records copy one at a time. Check records re-checks. Make primary is shown only
      when every record is found and you're an admin. Change address checks availability as
      you type (400 ms debounce) and offers free names when taken.
- [ ] AC9 Code: the tree opens files. Long-press (or keyboard) selects lines. `Ask Koda about lines
      {a}–{b}` quotes them above the composer on the Chat tab. Long lines wrap with no
      horizontal scroll at 390.
- [ ] AC10 Terminal shows the sandbox log. A failed start shows the summary and `Ask Koda to fix`,
      which quotes the error.
- [ ] AC11 Preview: sandbox phases show their copy and `Start again` works. Phone/Desktop switches
      the frame between 390 and 1440 scaled to fit.
- [ ] AC12 Invite: an email + role sends. A new person gets `Invite sent · to {email} · {days}
      days · Copy link`, and Copy puts `{origin}{path}` on the clipboard. An existing person shows
      as added.
- [ ] AC13 With a teammate in the project, the strip shows them and where they are. Follow mirrors
      their view and file. Stop, your own move, or their leaving ends it, and each is announced.
- [ ] AC14 Two asks show `AskQueue`. `Let {name} go first` only on a teammate's ask behind yours,
      and it reorders. An admin sees the change-ready card with Publish, Review in preview and
      Decline. The member hears the answer in a toast.
- [ ] AC15 `?open=settings|export|share|takedown|code(&file=)` opens the matching sheet or screen
      on the phone and then tidies the URL. The notification's `/projects/:id/app` lands on the
      workspace's Chat tab.
- [ ] AC16 Nothing claims what the API doesn't give: no ETA, no commit hashes, no RECENTLY
      CHANGED, no line tag, no running time on asks, no PRE-ORDERS tile, no QR.
- [ ] AC17 Keyboard and screen reader: the whole path Publish → confirm → result, Roll back,
      Share → Copy, and Code → ask about lines works without touch, and the announcements in
      section 5 fire.
- [ ] AC18 Export is reachable from ··· with the same weight as Share, and from Delete's
      `Export a copy first`.

## 7. Pen slips not to copy (add to AGENTS.md §9 when built)
`bR1XV` · 10 deploy number mismatch. `I8tuK0` · 6 publish time before the fix. `z0NpLc` · 6
lock-screen time and date. `OgP6t` · 14 "Can edit" (use the desktop's role labels).
`I6z3D6` / `yS6ev` "Ask Koda about this screen" / "knows the screen" (the frame is sandboxed).
`SaEJ7` · 5 "~3 min left". Also the flow header's "ACCEPT TRANSFER → HANDED OVER" (`Wv14s`
`gndih`), which has no steps.
