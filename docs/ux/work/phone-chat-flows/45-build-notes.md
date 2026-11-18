---
work: phone-chat-flows
status: built
contract: 40-ux-contract.md (verified 17 Nov 2026)
branch: feat/frontend-new-v2-phone-chat-flows
---

# Build notes — phone-chat-flows

## Build plan

Branch rebased onto `origin/main` first: it now has the phone chat controls (#283: long-press,
`PhoneMessageEditor`, `PhoneDislikeSheet`, `PhoneShareSheet`, `SheetTiles`, `ActionSheet`).

### Where things go

- Everything renders under `phone` (`phoneView` below 768, not the workspace pane) in `ChatView`;
  the desktop branch stays as it is. ChatView only gains a few `{#if phone}` swaps, so the merge with
  `feat/frontend-new-v2-chat-files` stays small.
- Logic stays where the desktop has it: `ReplyBlocks` (answers, approvals, forms, reminders) and
  `PlanBlock` (the plan's versions) take a `phone` prop and swap in phone presentation per block.
  Phone-only state that crosses blocks and the composer (questions parked, answers kept, the plan's
  run bar, crew sessions) lives in one class, `src/lib/chat/phoneFlows.svelte.ts`, which ChatView
  creates and hands down. Pure helpers in `src/lib/chat/phoneFlows.ts` (unit tested).
- The twins (`ds/{build,chat,conversation,mobile}`) are composed, never edited. New presentational
  components go in `src/lib/ds/thread/` (the phone thread's section, not copied by the old app), each
  with its docs entry in `src/lib/docs/entries/thread.ts` and a live example.

### Scenario map

| Contract step | Now / backend | Components | State | API / events | Tests |
|---|---|---|---|---|---|
| S1.1–1.4 questions sheet | now (no recommended, no helper) | `PhoneClarifySheet` (new), `PhoneOption`, `Progress`, `SheetHeader` via `BottomSheet` | `PhoneFlows.clarify` asking[i] / other / review / parked | `questions` block | component (single, other, multi, review, skip), e2e |
| S1 interim (parked) | now | `PhoneFlowBar` (new) | parked → Answer reopens at i | — | component, e2e |
| S1.5 review → Make the plan | now | `PhoneClarifySheet` review | answers in memory | `POST messages {text}` (`answersText`) | e2e |
| S1.6 answered fold | now | `ClarifyAnswered` (twin) | defaults counted this visit | — | e2e |
| S1.6/S2.1 draft plan + run bar | now (owner = plan's agent, no estimate) | `PhoneDraftPlan` (new), `PhoneFlowBar` | plan `proposed` in the latest reply | `GET agent-plans/:agent_session` | component, unit, e2e (seeded plan) |
| S1.7 run sheet | now (D1: sent as a message) | `PhoneRunSheet` (new) | closed ⇄ open → sent | `POST messages {text}` | component, e2e |
| S1.8 keep planning | now | `PhoneComposer` `notice` | hint until the next send | — | component, e2e |
| S1.9 revised | now | `PhoneDraftPlan` `V{n}` + `+` | diff of the two latest versions | agent-plans versions | unit (diff), e2e |
| S2.2 running plan, decision | now (no per-step detail) | `PlanCard` (twin), `PhoneDecision` (new) | — | `decision` block, "stop" | component, e2e |
| S2.3 form sheet | now | `PhoneFormCard` + `PhoneFormSheet` (new), `Segment` | values, past-date error | `formText` → `POST messages` | component, e2e |
| S2.4–2.6 approval | now (no Edit, no remember, no View in Gmail, D5) | mobile `ApprovalCard` (twin), `PhoneApprovalSheet` (new), `ApprovalCard` (twin) outcome | pending → sending → 202 / 409 reload / toast | `PUT messages/:id/approval {remember:false}` | component, unit (labels, outcome), e2e (409/err) |
| S3.3 don't send + replies | now (no Undo) | `PhoneQuickReplies` (new) | — | `replies` block | component, e2e |
| S3.4 crew working | now | `PhoneCrewWorking` (new) | sessions started/ended | `AgentSessionStarted/Ended`, `GET agent-status` (proxied) | component, unit |
| S3.5 toast elsewhere | interim (G2) | `PhoneToast` (twin) | — | activity feed + `GET /api/home` needs_you | e2e if reachable |
| S3.6 supervisor | now (no Discard) | `PhoneSupervisor` (new) | — | `stopped` block | component, e2e (seeded) |
| S4 error cards | now (no code, no SAID, no cost) | `PhoneErrorCard` (new) | failure kind | `run_failed`, `quota_notice` | component, unit (titles/actions), e2e (seeded) |
| S4 2.1/2.2 rate limit | now | `PhoneErrorCard` with a countdown | phase `retrying` | `AgentPhaseChanged` | unit |
| S4 5.1/5.2 offline | now | `Banner` (twin), `PhonePendingMessage` (new) | outbox | — | component, e2e |
| S4 5.3/5.4 cut off | now | `PhoneCutNotice` (new) | `turn.end` interrupted | — | component |
| S4 6.1 not sent | now | `PhonePendingMessage` | failed + reason | — | component |
| S4 6.2/6.3 credits | now | `Banner`, `PhoneErrorCard` | usage ≥ 80% | shell usage, `quota_notice` | unit (reset date) |
| S4 7.1/7.2 signed out | now | `PhoneSignedOutSheet` (new, not dismissable) | draft kept | 4401 / 401 | component, e2e |
| S4 8.x kinds | now (8.5 reads as `other`) | `PhoneErrorCard` | — | — | unit |
| S1/S2/S3 marked "no" | backend | — | — | — | BACKEND_TODO.md |

## Decisions applied (Mario hasn't answered D1–D5; the contract's defaults)

- **D1** Run plan sends the chosen option's title as the person's message.
- **D2** Honest copy ("You can leave. {Agent}'s request will wait here and on Home.") and the
  in-app toast; no Web Push (BACKEND_TODO).
- **D3** No "Don't ask again" anywhere on the phone; `remember` is always false.
- **D4** "Something else…" on every one-of-many question, phone and desktop (desktop: `other` set
  in the non-twin `adapters/blocks.ts`, so the twin `ClarifyCard` shows its own field).
- **D5** "Don't send" for a tool that sends (`/send|mail|message|reply/`), "Decline" for the rest,
  phone and desktop (desktop: the twin `ApprovalCard`'s existing `rejectLabel`, set in the
  non-twin `ReplyBlocks`).

## How to reach each scenario in the running app

Open any chat at a width under 768 (or with a phone's user agent). The fake model only writes
text, so the blocks come from the database, as `e2e/backend/phone-flows.e2e.ts` seeds them
(`seedReply`, `seedPlan`, `seedPendingApproval` in `e2e/backend/support/seed.ts`).

| Scenario | Reach it | Covered by |
|---|---|---|
| S1.1–1.5 questions, own answer, many of many, review | a reply with a `questions` block as the latest reply: the sheet opens by itself | e2e "Clarify → plan: the questions…", component tests |
| S1 interim (parked) | Close the sheet: the bar "{n} questions waiting · Answer" | same e2e |
| S1.6 answered fold | Make the plan: "Answered {n} questions" in the thread | same e2e |
| S1.6/S2.1 draft plan + bar | a reply with a `plan` block whose newest version is `proposed` | e2e "Clarify → plan: a proposed plan…" |
| S1.7 run sheet, S1.8 keep planning, S1.9 V2 + marks | the bar's Review & run (or Run plan); a plan with two versions | same e2e |
| S2.2 running plan | a plan the build runs (`active`): the desktop's PlanCard with Stop plan | — (needs a coding build) |
| S2.2 decision | a `view: decision` block | e2e "Plan → approve → act" |
| S2.3 form sheet, past date | a `view: form` block: Edit reminder on its summary card | same e2e |
| S2.4–2.6 approval, preview, folded outcome | an `approval_request` (gmail.send) paused in an agent session | same e2e (202); e2e "409" for no longer pending |
| S3.3 Don't send + quick replies | Don't send on the card; a `quick_replies` view | e2e "Approvals & background: quick replies…" |
| S3.4 crew working, leave note | while agent sessions run (AgentSessionStarted), or `agent-status` has one on open | unit (state), component |
| S3.5 toast elsewhere | a new needs-you approval in another chat while on /chats | e2e "elsewhere in the app…" |
| S3.6 supervisor | a `stopped` block | e2e "Supervisor's report" |
| S4 1.1, 3.1, 8.x error cards | a reply with `run_failed` and the backend's sentence | e2e "a failed turn…", unit (every kind) |
| S4 1.2 model sheet that retries | the card's Choose a model / Try another model | component + manual |
| S4 1.3 recovered on credits | Switch to Nomi credits on a retryable failure | manual (needs a failing provider) |
| S4 2.1/2.2 rate limit countdown, retried note | `AgentPhaseChanged` `retrying` during a turn | manual (needs a throttled provider) |
| S4 5.1/5.2 offline, back online | go offline and send | e2e "offline…" |
| S4 5.3/5.4 cut off, continued | a stream that drops part-way | component (PhoneCutNotice) |
| S4 6.1 not sent | a send the server refuses (500/413/429) | component |
| S4 6.2/6.3 credits low / used up | usage ≥ 80% / a `quota_notice` | unit (reset date), manual |
| S4 7.1/7.2 signed out, sent after | sign-in runs out (4401) or a send comes back signed out | e2e "signed out…" |

## Left out, and why

- **Backend first** (each in `frontend-new/BACKEND_TODO.md`, "Found while building the phone's
  chat flows"): recommended answers and helper lines; per-step owner, status line, timing, the
  estimate and "Finished in"; a run-mode API; approval with edited input and View in Gmail; Undo a
  deny and Discard after a stop; "Don't ask again" scoped to a recipient; structured failures
  (code, provider, provider's words) and a content-filter kind; Summarise & continue; Web Push.
- **Excluded by the contract:** Errors scenario 6 steps 4–6 (UXP-001); S3 steps 1–2 (edit the
  draft).
- **Not wired this round (the backend has it):** the model sheet's key health ("out of credits",
  "key rejected") and "{n}% left", and the masked key in the error card's meta: the chat page
  doesn't read `/api/llm/connections`. The error card's meta shows the chat's model only.
- **Not built (no source in the data):** the paused plan's failed step in mono, "waits for step
  {n}" and "{Agent} · blocked" (a plan's checklist doesn't say which step failed); "{Service}
  reconnected · resumed at step {n}" (no event says so). The connect block does show as the error
  card with Reconnect and Skip this step.
- **Rate limit "typing a new message cancels the retry":** the phone sends a new message the way
  the desktop does; whether that cancels the provider retry is the backend's queue behaviour, not
  something the client can force.
- **Twin limits** (no props added to twins): the answered line's defaults sit in
  `ClarifyAnswered`'s mono summary ("· 1 default"); the form summary is the twin `Reminder` card
  (its edit button reads "Edit reminder"); the sheet close's 44 target comes from `BottomSheet`'s
  scoped style around the twin `SheetHeader`'s 30 circle.

## Commits

See `git log origin/main..feat/frontend-new-v2-phone-chat-flows`.

## Gates (run 18 Nov 2026, in `frontend-new`)

- `npm ci` ok · `npm run i18n` ok · `npm run check`: 9497 files, 0 errors, 0 warnings.
- `npm run test:unit`: 167 files, 958 tests passed · `npm run test:components`: 108 files, 694 tests passed.
- `npm run build` ok · `npm run test:e2e` (stub backend, pen parity): 24 passed.
- `npm run test:twins`: 326 passed.
- `npm run test:e2e:backend` (E2E_APP_PORT=4184, the backend shared on 8080): 121 passed, 4 failed,
  1 did not run. `phone-flows.e2e.ts` 9/9 passed. The 4 failures are the shared backend's, not
  this branch's: it's the chat-files worktree's binary started with `APP_URL=http://localhost:4183`
  (the emailed reset and invite links go to that app: auth forgot-password, invites), whose crew
  has no Sora yet (#285 on main: app-shell's crew list), and whose sign-in code limit is shared with
  that run (auth "too many codes"). The backend binds 8080 fixed, so a second one couldn't be run.
