---
work: phone-chat-flows
status: verified
verified_by: ux-lead
verified_on: 17 Nov 2026
proposal: UXP-002 (P1/P2 only, non-blocking)
---

# UX contract — phone-chat-flows

The engineer builds exactly this, on a phone (`phoneView`, below 768), at 390 in dark and light.
Anything not here is out of scope or a question. The desktop at 768 and above doesn't change.

Ground rules for every step:
- **Pen 1:1.** Layout, spacing and tokens come from the frame ids below. Copy comes from the copy table. Where the
  pen's example text is data (names, amounts, times), render the real data in the same slot.
- **Targets.** 44 pt minimum on every tappable element, including the sheet Close button. The pen draws it 30×30, so pad the hit area.
- **Sheets.** They use `BottomSheet` with `SheetHeader`. Each one traps focus, Escape (on a keyboard) and Close dismiss it unless
  it's marked non-dismissable, and focus goes back to the element that opened it.
- **Steps marked "Exists? no".** The phone ships them the way the desktop does today, by leaving the control out, unless the step
  gives an interim. Each one is listed under "BACKEND_TODO entries" for the engineer to add to
  `frontend-new/BACKEND_TODO.md`.
- **Excluded.** Errors scenario 6 steps 4–6 (S61SO / g2x0P, U6S0r / WUqGg, K2r8A / E1NuW): UXP-001.

## Scenarios

### S1 · Clarify → plan · mobile GCvfD / tq7Ht · desktop S3yL5 / g11Ew
| Step | Mobile dark / light | Desktop | What happens |
|---|---|---|---|
| 1 Question sheet | qfsQ7 / Vds7a | vnXns | Nomi's latest reply carries a `questions` block, so the sheet opens over the thread: the eyebrow is `Before I plan`, the progress line reads `{chat title} · {n} of {total}` with segmented progress, the question in 20 pt, the helper line (when the backend sends one) and options as 64-pt rows numbered 1…n. If an option is `recommended`, it's preselected and tagged `RECOMMENDED`. Otherwise nothing is selected and Next stays disabled until something is picked. Every single-choice question ends with `Something else…` (D4). Buttons: `Skip` and `Next →`. |
| 2 Your own answer | L42Ej / Bbhp1 | qPEH6 | Tapping `Something else…` opens the keyboard. The other options collapse to the `Show the {n} suggestions` link, so the field and Next stay above the keyboard. Buttons: `Back` and `Next →`. Next is disabled while the field is empty. |
| 3 Skip uses the default | U58RFC / aQV02 | WeYJX | `Skip` moves on. With a recommendation, the answer becomes that option, marked as a default. Without one, the answer is "skipped". The left button reads **`Skip`**, not the pen's "Don't send", which is a slip (UXP-002 G5). |
| 4 Scope, many of many | U8a64 / RNCFx | s7dRje | When `multiple` is true, the rows become checkboxes. The primary button reads `Review {count}`, where count is the number ticked, and is disabled at 0. Buttons: `Back` and `Review {count}`. |
| 5 Review | PJcN8 / WUZef | e5dOL | The sheet titled `Your answers` has the sub `Nothing runs yet`. It shows one row per question (its short topic and the answer), and tapping a row goes back to that question. Defaults get the `DEFAULT` tag and the note `You skipped {topic}, so I picked {answer}.`. A skip with no recommendation reads `Skipped` and gets no note. Buttons: `Back` and `Make the plan`. Make the plan sends the answers as one message (desktop `answersText`), closes the sheet and shows the live reply (FvlVs). |
| 6 Plan in the thread | WcVCK / v7CKnN | JONMS | The questions fold to one line: `Answered {n} questions` or `Answered 1 question`, plus ` · {d} default` / ` · {d} defaults` when any were defaulted. The plan card shows the title, a `DRAFT` note line, the numbered steps and the owner (when known; see the data table). While the plan's `state` is `proposed`, a sticky bar sits above the composer: `Plan ready · nothing runs yet` with `Review & run`, and the bar opens step 7. |
| 7 How should it run? | KCocj / HCwQ8 | JONMS | The run sheet is titled `Run this plan?` with the sub `{n} steps · {owners}`. It has three `PhoneOption` rows, the recommended one first and preselected: `Run it, ask before anything leaves Nomi`, `Research only` and `Keep planning`. Buttons: `Not now`, which closes the sheet and keeps the bar, and `Run plan`. Run plan sends the chosen option's title as the person's message (D1), closes the sheet and removes the bar. |
| 8 Keep planning | eSUqU / HWkle | JONMS opt 3 | Choosing option 3 closes the sheet and focuses the composer, which shows the hint chip `Changing the plan · v{version}`. The person's reply is sent as a normal message. |
| 9 Revised, ready | nyN4S / J5QRb | l66hAZ | A new plan version arrives with `state` set to `updated` on the old one. The card shows the `V{n}` tag, and new steps are marked `+` with a highlight. A step is "changed" when its text isn't in the previous version's checklist, which the client works out by comparing the two. The bar comes back as `Plan v{n} ready` with `Review & run`. |

Interim (UXP-002 G1): when the questions sheet is closed with Close or by swiping down, the sticky
bar from step 6 appears with `{n} questions waiting` and `Answer`, and Answer reopens the sheet on the
question the person left. The sheet also opens on its own when the chat opens with an unanswered
`questions` block in the latest reply.

### S2 · Plan → approve → act · mobile G1tK6L / EI3k6 · desktop HU6nc / mjZPs
| Step | Mobile dark / light | Desktop | What happens |
|---|---|---|---|
| 1 Draft plan | jBK6g / x1FvZ | z7FTq | The plan card has the `DRAFT` tag, steps with their owner on the right, and the foot `~{n} min · {owners}` only if the backend sends an estimate (it doesn't today, so leave it out). The run bar reads `{n} steps · {owners}` with `Run plan`, and opens S1 step 7. There's no long-press on steps (G3). The intro line reads **`Tell Nomi what to change. Nothing runs until you say so.`** (G3 copy change). |
| 2 Running, then a decision | gQ44i / Xmh0l | VsyNZ | The plan card shows `Plan · {title}`, `{done} of {total} done` and the step states (done, running, waiting). A per-step detail line shows only when the backend sends one. `Stop plan` sends "stop". A `decision` block shows big radio rows (`PhoneOption`), the suggested one first and tagged `RECOMMENDED`. The primary button reads `Use {option title}` and sends that title. |
| 3 Form sheet, with an error | MKUyQ / Kkv8E | X1TbC | A `form` block shows in the thread as its summary card (title, the prefilled values and the agent). It opens as a sheet titled after the form (`New reminder`) with the sub `{Agent} · prefilled from chat`. Fields: text, date, time and the `Once / Weekly / Monthly / Yearly` segment. A date in the past shows under the field: `That date has passed.`, followed by the backend's own suggestion sentence if one comes back. Save stays disabled until it's fixed. Buttons: `Cancel` and `Save`. Save sends the form (desktop `formText`). |
| 4 Saved, then an approval | XnozL / gCOOd | OQySw | The saved reminder lands as a `Reminder` card. A pending `approval_request` shows as `Mobile / Approval Card`: the title (`{Agent} wants to {description}`), the first fields compact (to, subject, a two-line body preview, the attachment chip), and the buttons `Don't send` and `Approve` (D5). The nav-bar sub reads `{Agent} · needs you`. |
| 5 Full preview | Yi1GU / pLM5s | (inline on desktop) | Tapping the card opens the sheet `Send this email?` with the sub `{Agent} · via your Gmail`, showing every field in full. The `Don't ask again…` toggle is **left out** (D3 / G4). `Edit` is **left out** until the backend takes edited input (G8). The button is `Approve & send`. |
| 6 Done | hgXxG / jidyu | KIlcs | After 202, the card folds to the outcome line, `{Agent} sent the email to {recipient}`, with the meta `Approved by you · sent {HH:MM}` using the approval's real time (the pen's 14:06 is a slip). `View in Gmail` is **left out** (G8). The plan reads `{total} of {total}`, and the foot reads `Finished in {m}m {s}s` (computed from the plan message's `created_at` to the turn's end). The reply gets its `ActionBar`. |

### S3 · Approvals & background work · mobile xvntx / qhcNA · desktop bWiWT / t16cU
| Step | Mobile dark / light | Desktop | What happens |
|---|---|---|---|
| 1 Edit the draft | mIUPn / jdAny | M39Rx | **Not built.** There's no backend for edited input (G8), so Edit stays hidden. |
| 2 Sent with your edits | tRxJl / ZxNsS | gzagU | **Not built.** Depends on step 1. |
| 3 Don't send | upMaX / jwIL6 | IFTOC | `Don't send` sends `deny`, and the card folds to `{Agent}'s email to {recipient}` with the sub `Not sent · nothing left Gmail`. There's **no `Undo`** (G8). The agent's follow-up arrives as a normal reply with its `QuickReplies` (`replies` block). |
| 4 Runs in the background | HsE8N / i3AxZ | HcQAT | While one or more agent sessions are active (`AgentSessionStarted` / `Ended`, `GET …/agent-status`), a banner under the nav bar shows the crew mascots and `{n} crew working`, with the button **`Stop all`** (it sends "stop"). The nav-bar sub reads `{n} crew working`. The note under the plan reads **`You can leave. {Agent}'s request will wait here and on Home.`** (G2 copy change: there's no push to a phone browser). The `~3 min` estimate is left out. |
| 5 Lock screen | hcv7x / QIkI5 | Z2dLp | **Not on the web phone** (push is native-only). Interim (G2): if the person is elsewhere in the app when an approval or question arrives in another chat, show `PhoneToast` with `{chat title}: {Agent} needs your OK` and `Open`. Home lists it under "needs you". |
| 6 Stop all | nQiZd / ft0Qv | eCsdf | A `stopped` block shows the Supervisor card: `Supervisor stopped {n} tasks`, one row per task (`done`, `stopped · {detail}` or `not started`) and the line `Nothing sent was undone.`. `Resume` sends the resume message. `Discard` is **left out** (G8). |

### S4 · Errors & recovery · mobile AyrHG / uuCam · desktop TpuRc / Nud6J
The error card is `Mobile / Error Card` (`ErrorCard`). Above it goes one plain sentence: the backend's
`content`. The card shows the title (from the failure kind) and the actions from `failureView`. The pen's
`{code}` badge and the `{PROVIDER} SAID · {reason}` block are **left out** until the backend sends
structured errors (G8). The meta line `{model} · key ••{last4}` shows when the chat's model is known.
The key part appears only when it runs on a person's key (`api_key_masked`). No cost on any failure.

| Scenario · step | Mobile dark / light | Desktop | What happens |
|---|---|---|---|
| 1.1 Out of credits | iXX9T / D4yEn | r4QPG | Kind `no_credits`, title `Out of credits`. Primary `Switch to Nomi credits` (when credits can be used, else `Choose a model`). Secondary `Retry` when retry is allowed. The pen's `Add credits on DeepSeek` is left out: there's no provider billing link. |
| 1.2 Switch where it runs | MLk2U / kFsUB | M1zmIC | The model sheet is grouped `YOUR KEYS` / `NOMI CREDITS`. A key whose health is `balance_empty` shows `out of credits`, and one with `rejected` shows `key rejected`. The Nomi credits row shows `{n}% left`. The primary button reads `Use Nomi credits and retry`. |
| 1.3 Recovered | kFkym / Yhvo4 | N6glC | The byline meta reads `{HH:MM} · Nomi credits`. The note reads `First try failed: {reason} Retried on Nomi credits.`, where `{reason}` is the failure sentence. |
| 2.1 Rate limited, retrying | Obkjq / Qix0k | m9cSQ8 | The `retrying` phase brings the card titled `Rate limited` with the primary `Retrying in {m:ss}`, which counts down from the phase detail and can't be pressed, and the secondary `Try another model`. Typing a new message cancels the retry. |
| 2.2 Retried automatically | ABfCk / sARxr | ySqag | The meta reads `{HH:MM} · retried`. The note reads `Rate limited for {s}s, then retried automatically.` |
| 3.1 Context limit | Vy4HC / Thg9t | k7TBc | Title `Chat too long`. Actions as the desktop has them: `Start a new chat` and `Use a {n}K model` (the smallest enabled model with a bigger `context`, else `Choose a model`). The pen's `Summarise & continue` is left out (G8). |
| 3.2 Summarised | J08wT / FKsV9 | wr5K6 | **Not built** (G8). |
| 4.1 Step failed | JHxIs / HNwBO | Kz8QN | The plan card reads `paused`. The failed step shows `{tool}` in mono, and the steps after it show `waits for step {n}`. The nav-bar sub reads `{Agent} · blocked`. A `connect` block shows as the error card titled `{Service} signed out`, with `Reconnect {Service}` (OAuth, returning to this chat) and `Skip this step` (sent as a message). |
| 4.2 Resumed | xC1OJ / wSPjs | XcnaV | Back in the chat, the note reads `{Service} reconnected · resumed at step {n}`. Approvals still ask (S2 step 4). |
| 5.1 Offline, queued | x8dgV / EWjXb | was5c | Banner: `You're offline. Messages send when you're back.` Your bubble is marked `queued · sends when you're back` with `Cancel`. |
| 5.2 Back online | HF7vy / zmOYu | anQwN | Banner: `Back online. 1 queued message sent.` (or `Back online.` when nothing was queued). It hides after 4 s. |
| 5.3 Cut off | UTrnr / HVyK2 | FCOhA | The partial text stays. The note reads `Reply cut off after {s}s`, with `Start over` and `Continue`. |
| 5.4 Continued | M7u2KP / yW7KJ | Ks1u8 | The meta reads `{HH:MM} · continued` and the note `continued after the connection dropped`. |
| 6.1 Not sent | TJhp2 / c2WTE | A9dKD | Your bubble stays with `not sent · {reason}` and `Retry`. Edit comes through the existing long-press message sheet. |
| 6.2 Credits running low | QMGjy / dycE0 | N6bqZ | At 80% or more, a banner reads `{n}% of Nomi credits used · resets {1 Mmm}` (the first day of next month) with `Usage`, which goes to `/settings/plan`. |
| 6.3 Credits used up | WVxJ4 / QXoA9 | oNADF | A `quota_notice` gives the card `Out of Nomi credits`, with the meta `{used} of {allowance} tokens used · resets {1 Mmm}` and the plan tag. Actions: `Upgrade plan` and `Use my own key`. |
| 7.1 Signed out | hAhHg / lsBur | PIVgr | The socket closes with 4401, or a send comes back signed out. A sheet that **can't be dismissed** says `You've been signed out`, the body text, and the kept draft with the `kept` tag. The button is `Sign in again`. |
| 7.2 Back, and sent | M75Qz / gEmpO | t0jQ11 | Back in the same chat, the outbox sends, and the note under the bubble reads `sent after you signed in · {HH:MM}`. |
| 8.1 Key rejected | BdRr4 / tAJzS | rmLEk | `bad_key`, title `Key rejected`. Actions: `Update key` (`/settings/models`) and `Use Nomi credits`. |
| 8.2 Model retired | VWoAf / fbJcw | sjWbQ | `no_model`, title `Model retired`. Action: `Choose a model`. |
| 8.3 Provider down | Yw8Ai / X3lRG | UJGnO | `provider_down`, title `Provider down`. Actions: `Retry` and `Try {other model}` (or `Choose a model`). |
| 8.4 Timed out | NI5Su / ZlGO1 | k1TqkU | `timed_out`, title `Timed out`. Actions: `Retry` and `Try a faster model` (opens the model sheet). |
| 8.5 Blocked by provider | IUA3P / o6xwQH | B86hlc | There's no backend kind yet, so it reads as `other` (`Couldn't reply`, `Retry`). When the kind exists, the title becomes `Blocked by provider` with `Rephrase` (puts your message back in the composer) and `Try another model`. |

The phone shows the remaining failure kinds (`unreachable`, `unreadable`, `other`) in the same card, with
the desktop's titles and actions.

## State machine
- **Questions sheet:** `closed` → (latest reply has an unanswered `questions` block, on load or
  `MessageCreated`) → `asking[i]` → (Next / Skip) → `asking[i+1]` … → `review` → (Make the plan) →
  `sending` → `answered`. From any `asking` or `review` state, Close goes to `parked`, where the bar shows `{n} questions waiting` and Answer goes back to `asking[i]`.
  Answers are kept in memory while the chat stays open; leaving the chat drops them (G9).
- **Plan:** `proposed` (bar shown) → (Run plan sends a message) → `active` (Stop plan shown) →
  `completed`, or `blocked` (failed step). A new version moves the old one to `updated` (V tag, `+` marks).
- **Run sheet:** `closed` ⇄ `open` (Review & run / Not now) → `sent`.
- **Approval:** `pending` → (Approve, Don't send) → `sending` → 202 → `approved` | `rejected`. A 409 → reload
  the message, which shows the real status. Any other error → stays `pending` + `PhoneToast` error.
  A `MessageUpdated` from another device moves it to `approved`, `rejected` or `cancelled`.
- **Crew working:** `idle` → `AgentSessionStarted` → `working(n)` → `AgentSessionEnded` (n=0) → `idle`.
  Stop all → `stopping` → a `stopped` block arrives.
- **Turn failure:** `streaming` → `TurnFailed` / a `run_failed` message → `failed(kind)` → (action) →
  `retrying` → `recovered` (note) | `failed`. Phase `retrying` → `waiting(seconds)` → `streaming`.
- **Connection:** `online` → `offline` (queue) → `back-online` (4 s) → `online`. A stream drop gives `interrupted`, and Continue resumes it.
- **Session:** `signed-in` → 4401 → `signed-out` (sheet, draft kept) → sign in → `signed-in` → the outbox sends with the late note.

## Copy table (verbatim; `{}` = data)
| Key | Text | Where |
|---|---|---|
| clarify.eyebrow | `Before I plan` | S1.1 |
| clarify.progress | `{chat title} · {n} of {total}` | S1.1 |
| clarify.recommended | `RECOMMENDED` | S1.1, S1.3, S1.7, S2.2 |
| clarify.other | `Something else…` | S1.1 |
| clarify.show_suggestions | `Show the {n} suggestions` | S1.2 |
| clarify.skip / back / next | `Skip` · `Back` · `Next` | S1.1–S1.4 |
| clarify.review_count | `Review {count}` | S1.4 |
| clarify.review_title / sub | `Your answers` · `Nothing runs yet` | S1.5 |
| clarify.default_tag | `DEFAULT` | S1.5 |
| clarify.default_note | `You skipped {topic}, so I picked {answer}.` | S1.5 |
| clarify.skipped | `Skipped` | S1.5 |
| clarify.make_plan | `Make the plan` | S1.5 |
| clarify.answered | `Answered {n} questions` / `Answered 1 question` + ` · {d} default(s)` | S1.6 |
| clarify.waiting / answer | `{n} questions waiting` · `Answer` | S1 interim |
| plan.intro | `Tell Nomi what to change. Nothing runs until you say so.` | S1.6, S2.1 (replaces "Reorder, reassign or remove steps…") |
| plan.ready_bar | `Plan ready · nothing runs yet` · `Review & run` | S1.6 |
| plan.ready_bar_v | `Plan v{n} ready` · `Review & run` | S1.9 |
| plan.steps_owners | `{n} steps · {owners}` | S1.7, S2.1 |
| run.title | `Run this plan?` | S1.7 |
| run.option1 / detail | `Run it, ask before anything leaves Nomi` · `{Agent} asks before sharing; Dana only records` (agent-supplied detail; omit if none) | S1.7 |
| run.option2 / detail | `Research only` · `{research steps}. Nothing shared.` | S1.7 |
| run.option3 / detail | `Keep planning` · `Tell Nomi what to change` | S1.7 |
| run.not_now / run | `Not now` · `Run plan` | S1.7, S2.1 |
| plan.changing | `Changing the plan · v{n}` | S1.8 |
| plan.progress | `{done} of {total} done` | S2.2 |
| plan.stop | `Stop plan` | S2.2 |
| plan.waits | `waits for step {n}` | S4 4.1 |
| plan.paused | `paused` | S4 4.1 |
| plan.finished | `Finished in {m}m {s}s` | S2.6 |
| decision.use | `Use {option}` | S2.2 |
| form.sub | `{Agent} · prefilled from chat` | S2.3 |
| form.past_date | `That date has passed.` | S2.3 |
| form.cancel / save | `Cancel` · `Save` | S2.3 |
| approval.nav | `{Agent} · needs you` | S2.4 |
| approval.deny / approve | `Don't send` · `Approve` (D5) | S2.4 |
| approval.preview_title | `Send this email?` | S2.5 |
| approval.preview_sub | `{Agent} · via your Gmail` | S2.5 |
| approval.approve_send | `Approve & send` | S2.5 |
| approval.sent | `{Agent} sent the email to {recipient}` · `Approved by you · sent {HH:MM}` | S2.6 |
| approval.not_sent | `{Agent}'s email to {recipient}` · `Not sent · nothing left Gmail` | S3.3 |
| approval.failed | `The decision couldn't be recorded` | toast |
| crew.working | `{n} crew working` | S3.4 |
| crew.stop_all | `Stop all` | S3.4 |
| crew.leave_note | `You can leave. {Agent}'s request will wait here and on Home.` | S3.4 |
| notify.toast | `{chat title}: {Agent} needs your OK` · `Open` | S3.5 interim |
| supervisor.title | `Supervisor stopped {n} tasks` | S3.6 |
| supervisor.states | `done` · `stopped · {detail}` · `not started` | S3.6 |
| supervisor.note | `Nothing sent was undone.` | S3.6 |
| supervisor.resume | `Resume` | S3.6 |
| error.titles | `Out of credits` · `Rate limited` · `Chat too long` · `Key rejected` · `Model retired` · `Provider down` · `Timed out` · `Out of Nomi credits` · `Couldn't reply` | S4 |
| error.actions | `Switch to Nomi credits` · `Use Nomi credits and retry` · `Retry` · `Retrying in {m:ss}` · `Try another model` · `Start a new chat` · `Use a {n}K model` · `Choose a model` · `Update key` · `Use Nomi credits` · `Try {model}` · `Try a faster model` · `Upgrade plan` · `Use my own key` | S4 |
| error.recovered | `First try failed: {reason} Retried on Nomi credits.` | S4 1.3 |
| error.retried | `Rate limited for {s}s, then retried automatically.` | S4 2.2 |
| model.groups | `YOUR KEYS` · `NOMI CREDITS` · `out of credits` · `key rejected` · `{n}% left` | S4 1.2 |
| connect.title / actions | `{Service} signed out` · `Reconnect {Service}` · `Skip this step` | S4 4.1 |
| connect.resumed | `{Service} reconnected · resumed at step {n}` | S4 4.2 |
| net.offline | `You're offline. Messages send when you're back.` | S4 5.1 |
| net.queued | `queued · sends when you're back` · `Cancel` | S4 5.1 |
| net.back | `Back online. 1 queued message sent.` / `Back online.` | S4 5.2 |
| net.cut | `Reply cut off after {s}s` · `Start over` · `Continue` | S4 5.3 |
| net.continued | `continued after the connection dropped` | S4 5.4 |
| send.failed | `not sent · {reason}` · `Retry` | S4 6.1 |
| credits.low | `{n}% of Nomi credits used · resets {1 Mmm}` · `Usage` | S4 6.2 |
| credits.out | `{used} of {allowance} tokens used · resets {1 Mmm}` | S4 6.3 |
| auth.title | `You've been signed out` | S4 7.1 |
| auth.body | `Your session ended, so this message wasn't sent. It's saved on this phone and goes out after you sign in. Running crew tasks keep going.` | S4 7.1 |
| auth.kept / button | `kept` · `Sign in again` | S4 7.1 |
| auth.late | `sent after you signed in · {HH:MM}` | S4 7.2 |

Each string needs keys in both `en.json` and `id.json`. The Indonesian text goes through the usual
translation pass, not the pen.

## Data and events the UI depends on
| Need | Source (API / event / config) | Exists? |
|---|---|---|
| Questions, options, details, multiple | `questions` block (`intro`, `questions[{question, options[{label, detail}], multiple}]`) | yes |
| Recommended option per question | — | **no**: no preselect, Next is disabled until a pick |
| Helper line under a question ("You and Sari, from Jakarta.") | — | **no**: omitted |
| Send answers, run choice, keep planning, skip step, resume | `POST /api/sessions/:id/messages {text}` | yes |
| Plan steps, version, state | `plan` block + `GET /api/sessions/:id/agent-plans/:agent_session_id` | yes |
| Per-step owner, status detail, timing; plan estimate | — | **no**: owner = the plan's agent; detail and estimate omitted |
| Run mode (run / research only) as an API | — | **no**: sent as a message (D1) |
| Decision, form, reminder, quick replies, connect blocks | `decision` · `form` · `reminder` · `replies` · `connector_connect`/`workspace_connect` blocks | yes |
| Approve / deny | `PUT /api/sessions/:id/messages/:mid/approval {decision, remember:false}` → 202 / 409 | yes |
| Edit approval input before sending | — | **no**: Edit hidden |
| Undo a deny | — | **no**: Undo hidden |
| Remember scoped to a recipient | `remember` is per tool (+path) | **no**: toggle hidden (D3) |
| Link to the sent message | — | **no**: View in Gmail hidden |
| Crew working, start/end | `AgentSessionStarted` / `AgentSessionEnded` / `AgentPhaseChanged` on `/chat/:id/ws`; `GET …/agent-status` | yes |
| Stop all | message "stop" | yes |
| Supervisor report | `stopped` block (`tasks[{agent, title, detail, state}]`) | yes |
| Discard after stop | — | **no**: hidden |
| Needs-you elsewhere in the app | `/api/me/activity/ws`, `GET /api/home` (`needs_you`) | yes |
| Push to a closed phone browser | push is FCM / APNs (native) only | **no**: G2 |
| Failure kind, sentence, retry | `run_failed` (+`retry`), `quota_notice`; `failureKind` from the sentence | yes |
| HTTP code, provider name, provider reason | — | **no**: badge and "SAID" block omitted |
| Chat model; masked key | `GET /api/sessions/:id/model`; `GET /api/llm/connections` `api_key_masked` | yes |
| Key health (`balance_empty`, `rejected`) | connections health | yes |
| Model context sizes | `GET /api/llm/model-options` `context` | yes |
| Retry countdown | `AgentPhaseChanged` phase `retrying`, detail (attempt, seconds) | yes |
| Content-filter failure kind | — | **no**: falls to `other` |
| Summarise older messages | — | **no**: not built |
| Credits used, allowance | `GET /api/usage`, `GET /api/usage/brief` | yes |
| Offline queue, signed-out draft | `outbox.ts` (this device), socket close 4401 | yes |
| Continue a cut-off reply | `continueReply` (existing) | yes |
| Retry a failed run / send | `POST …/messages/:mid/retry`; resend | yes |

### BACKEND_TODO entries (engineer adds to `frontend-new/BACKEND_TODO.md`)
1. `recommended: bool` and `helper` on question options and questions.
2. Per-step `owner`, `status_detail`, `started_at` / `finished_at` in a plan, and an optional estimate.
3. Run mode for a proposed plan (`POST …/agent-plans/:id/run {mode: run|research}`).
4. Approval with edited input (`PUT …/approval {decision:"approve", input}`) and a sent-message link.
5. Undo a deny within a short window, and Discard after a stop.
6. `remember` scoped to a recipient (or the same rule as the tool), so the copy can say what it does.
7. Structured failure: `{status, provider, provider_message, model}` on `run_failed`, and a content-filter kind.
8. Summarise older messages and continue (context limit).
9. Web Push (VAPID) for the phone browser: needs-you, approvals, plan finished (D2).

## Accessibility
- **Questions sheet.** It's `role="dialog"` with an `aria-labelledby` pointing at the question. Opening it focuses the selected
  option, or the first one. Options are a `radiogroup`, or a group of checkboxes when `multiple` is true, and each option's
  accessible name is its title, then its detail, then "recommended". Focus order: options → Something else… → Skip / Back →
  Next → Close. Moving to the next question announces (polite) `Question {n} of {total}: {question}`.
  Closing returns focus to `Answer` on the bar.
- **Own answer.** Focus goes to the field. When the person leaves the field, `Show the {n} suggestions` brings back the list and focuses the
  first option.
- **Review.** Each answer row is a button named `Change {topic}: {answer}`. Make the plan is the last item.
- **Run sheet.** A radiogroup, the recommended option focused first. Not now and Run plan come last. After Run plan, focus moves
  to the composer and the screen reader announces `Plan sent to run.`
- **Plan card.** A list. Each step's state is in its name (`Step 2, done, Dana`). Progress changes are
  announced politely: `Plan: {done} of {total} done`. A plan paused on a failed step is announced assertively:
  `Plan paused: {Service} signed out`.
- **Form sheet.** Opening it focuses the first field. The error is tied to its field (`aria-describedby`, `aria-invalid`) and
  announced when it appears. Save stays disabled with `aria-disabled` and its reason in the field error.
- **Approval.** The card is a region named `{Agent} needs your OK`. The sheet traps focus on `Approve & send`
  last. The outcome is announced: `Email sent to {recipient}` / `Not sent. Nothing left Gmail.`
- **Banners.** Offline, back online, crew working and credits low are `role="status"` (polite). The offline banner
  doesn't take focus.
- **Errors.** The error card is announced assertively when it arrives: `{title}. {sentence}`. Focus doesn't move.
  The primary action comes first in the tab order after the sentence. The countdown is announced at the start only and
  at 0 (`Retrying now`), not every second.
- **Signed-out sheet.** It's `role="alertdialog"`, doesn't respond to Escape or swipe, and focus starts on `Sign in again`.
- **Toast.** `PhoneToast` is `role="status"`, and `Open` can be reached with the keyboard while it's shown (at least 6 s, and it pauses on focus).
- **Targets.** Every control is at least 44×44 pt, including the sheet Close.

## Acceptance criteria
- [ ] AC1 · All steps listed above render at 390 in dark and light, matching their frame ids (1:1 per AGENTS.md §6), except the documented copy changes and left-out controls.
- [ ] AC2 · A reply with a `questions` block opens the questions sheet. Close parks it, and `{n} questions waiting · Answer` reopens it on the same question with earlier answers kept.
- [ ] AC3 · With no `recommended` option, nothing is preselected and Next is disabled until a pick. With one, it's preselected, and Skip records it as `DEFAULT` in the review with the note.
- [ ] AC4 · `Something else…` opens the keyboard. The field and Next stay above it, and Next is disabled while it's empty.
- [ ] AC5 · Many-of-many shows checkboxes, and `Review {count}` matches the ticks and is disabled at 0.
- [ ] AC6 · Make the plan sends one message with every answer. The thread then shows `Answered {n} questions · {d} default`.
- [ ] AC7 · A `proposed` plan shows the run bar. Run plan sends the chosen option. Keep planning focuses the composer with `Changing the plan · v{n}`.
- [ ] AC8 · A new plan version shows `V{n}` and marks added steps with `+`. Steps that didn't change aren't marked.
- [ ] AC9 · No copy on the phone offers reordering, reassigning or removing plan steps, an estimate (`~n min`), or a lock-screen notification.
- [ ] AC10 · `Stop plan` and `Stop all` send "stop". The Supervisor card lists each task's real state and `Nothing sent was undone.`, and Resume sends the resume message.
- [ ] AC11 · Form: a past date shows `That date has passed.` under the field, and Save stays disabled until it's fixed.
- [ ] AC12 · Approval: Approve and Don't send call the approval API once each. 202 folds the card. 409 reloads and shows the real outcome. Any other error keeps the card pending and shows `The decision couldn't be recorded`.
- [ ] AC13 · The approval preview sheet shows no Edit, no `Don't ask again` toggle and no `View in Gmail`. `remember` is always false.
- [ ] AC14 · The crew-working banner appears on the first `AgentSessionStarted` and goes away when the last session ends.
- [ ] AC15 · Elsewhere in the app, a new needs-you item shows `PhoneToast` with Open, which leads to the chat.
- [ ] AC16 · Every failure kind shows its title and the desktop's actions in the phone error card. No HTTP code, provider "SAID" block or cost is shown. The model line appears when the model is known.
- [ ] AC17 · Rate limit: the countdown matches the `retrying` phase seconds, and typing a message cancels the retry.
- [ ] AC18 · `Switch to Nomi credits` / `Use Nomi credits and retry` switches the chat's model and retries. The recovered note and the `· Nomi credits` meta follow.
- [ ] AC19 · Offline: the message queues with Cancel and sends by itself when the connection is back, and the banner flips for 4 s.
- [ ] AC20 · Cut-off reply: the partial text is kept. Continue completes it in place with the continued note, and Start over discards it.
- [ ] AC21 · Signed out: a sheet that can't be dismissed, with the kept draft. After sign-in, the same chat sends it with `sent after you signed in · {HH:MM}`.
- [ ] AC22 · Credits: the banner shows at 80% or more with the reset date, and at 100% the card shows Upgrade plan and Use my own key.
- [ ] AC23 · Connect failure mid-plan: the plan shows `paused`, and later steps read `waits for step {n}`. Reconnect returns to this chat. Skip this step sends a message.
- [ ] AC24 · Every tappable element is at least 44 pt, and every sheet traps focus and returns it to whatever opened it. The live announcements are as listed under Accessibility.
- [ ] AC25 · New and changed DS components each get their docs entry and live example in the same PR. Twin props stay frozen (`test:twins`).

## Decisions for Mario
These are the defaults this contract builds unless Mario says otherwise.
- **D1 · Run choice as a message.** The run sheet sends the chosen option's title as the person's message. That's how decision and clarify answers already work. The alternative is to wait for a run-mode API. **Recommended: send it as a message.**
- **D2 · Lock-screen notifications.** A phone browser gets no push today. The options are to ship honest copy and an in-app toast now, then add Web Push later; or to hold the background step until push exists. **Recommended: honest copy now, Web Push as a backend item.**
- **D3 · "Don't ask again".** The backend's `remember` would allow every `gmail.send`, to anyone. The options are to leave it out (as the desktop does), to word it for the whole tool ("Don't ask before {Agent} sends email"), or to have the backend scope it to the recipient. **Recommended: leave it out until the backend can scope it to the recipient.**
- **D4 · `Something else…` on every single-choice question.** The pen draws it on every one. The desktop shows it only when the adapter sets `other`, and it never does. Answers go back as text, so no backend is needed. **Recommended: show it on every single-choice question, on both desktop and phone.**
- **D5 · Deny label.** The pen says `Don't send`; the desktop says `Decline`. **Recommended:** `Don't send` for tools that send something (`gmail.send` and other mail or message tools) and `Decline` for the rest. Use the same rule on desktop.
