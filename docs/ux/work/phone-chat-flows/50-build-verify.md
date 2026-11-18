---
work: phone-chat-flows
gate: 3 (build verify)
verified_by: ux-lead
verified_on: 18 Nov 2026
build: feat/frontend-new-v2-phone-chat-flows (PR #306) at 464ad1cf, `npm run build` + `node server.js` on :4191, backend on :8080 (fake model)
contract: 40-ux-contract.md (verified 17 Nov 2026)
verdict: CHANGES REQUIRED (3 high · 6 medium · 3 low)
---

# Build verify: phone-chat-flows

## How it was checked

- **Viewport.** Phone 390×844 with an iPhone user agent and touch, dark and light (`colorScheme`), Chromium with `--font-render-hinting=none`. The contract leaves the desktop as it is, so the desktop wasn't captured.
- **Accounts.** Made as `e2e/backend/support/accounts.ts` makes them.
- **Seeding.** Blocks were seeded with `seedReply`, `seedPlan`, `seedPendingApproval` and `seedUsage`, the way `e2e/backend/phone-flows.e2e.ts` seeds them:
  - questions with three questions (two single, one many-of-many);
  - a plan with one version, and one with two;
  - a decision, a form, `gmail.send` approvals, quick replies and a `stopped` block;
  - one `run_failed` per failure kind, a `quota_notice` and a `connector_connect`.
- **Crew working (S3.4).** Reached by inserting an `active` `agent_sessions` row, which `GET …/agent-status` returns on open.
- **Offline (S4 5.1/5.2).** Reached with `context.setOffline`.
- **Signed out (S4 7.1).** Reached by clearing the cookies and sending.
- **What was measured.** `getBoundingClientRect` and computed styles were checked against `Get(…, {resolveVariables, resolveInstances})` of:
  - S1: qfsQ7, L42Ej, U8a64, PJcN8, WcVCK, KCocj, eSUqU, nyN4S;
  - S2: gQ44i, MKUyQ, XnozL, Yi1GU, hgXxG, jBK6g;
  - S3: upMaX, HsE8N, nQiZd;
  - S4: iXX9T, x8dgV, QMGjy.
- **Where the files are.** Screenshots and JSON are in `/private/tmp/claude-501/-Users-marioprasetyo-Documents-Nomi/7ff24e31-b37d-4e91-90b3-5ff5b1682437/scratchpad/gate3/shots/pf/`.

## What matches (no action)

- **S1.1 question sheet (qfsQ7).**
  - Header 75: title 17/600, sub `{chat title} · 1 of 3` 13 `#64646C`, Close 30.
  - Progress 358×4, gap 6. The question is 20/600 at 438.
  - Options 70 high, r14, padding 14, 26 key, 16/500 title, 13 detail, 22 radio at x 338, gap 8. `Something else…` is 54 high, 16 `#64646C`.
  - Skip and Next are 175×46 r12. Next is off until a pick; with no `recommended`, nothing is picked.
  - Focus starts on the first option.
- **S1.2 own answer (L42Ej).**
  - The other options fold to `Show the 3 suggestions`, and the field row is 54 high with the accent border.
  - Back and Next stay in view, and the field takes focus.
- **S1 interim.** Close parks the questions. The bar reads `2 questions waiting` · `Answer`, and Answer reopens the sheet on the same question.
- **S1.5 review (PJcN8).**
  - Rows 68: Q 13 `#64646C`, value 16, edit icon 16. A skip with no recommendation reads `Skipped` with no note.
  - Make the plan sends one message, and the skipped question is left out of it.
- **S1.6 fold and draft (WcVCK / jBK6g).**
  - The fold reads `Answered 3 questions`.
  - Card: 358 wide, header 38, DRAFT tag, and the intro copy `Tell Nomi what to change. Nothing runs until you say so.` (G3).
  - Steps: 11 mono index, 13 label. The owner chip is bordered, r6, with no chevron.
  - No estimate, and no reorder or remove (AC9).
- **S1.7 run sheet (KCocj).**
  - Title and `{n} steps · {owners}`. Three rows, with the first preselected and tagged `RECOMMENDED`; focus is on it.
  - `Not now` and `Run plan`. The choice goes as the person's message (D1, e2e).
- **S1.8 (eSUqU).**
  - Keep planning closes the sheet and focuses the field.
  - Notice `Changing the plan · v1`: 11 mono `#64646C`, 12 icon at x 16, text at 34. The bar goes.
- **S1.9 (nyN4S).**
  - V2 tag. The added step is marked `+` in accent, with an 8% accent row and a label gap of 10.
  - The unchanged steps aren't marked. The bar reads `Plan v2 ready` · `Review & run`.
- **S2.2 decision (gQ44i).**
  - The recommended option comes first, 91 high, with a `RECOMMENDED` tag. Option gap 12.
  - `Use 3 days before` is 358×46 r12, 12 below the options.
- **S2.3 form sheet (MKUyQ).**
  - Title `New reminder`, sub `Kala · prefilled from chat`. Labels 13/500, fields 44 r6, label gap 6, field gap 14.
  - The past date has a danger border, an icon, and `That date has passed.` in 12 danger with `role=alert`.
  - Segment 40, padding 3, gap 3, active `#0A0A0B` 14/600. Save is off. Focus is on the first field.
- **S2.4 card (MOzBb).**
  - Region `Tara needs your OK`. Header 44 (15/600 title, chevron). `to …` 11 mono, subject 15/500, two-line excerpt 14 `#A1A1A9`.
  - Buttons 46: `Don't send` (D5) and `Approve`.
- **S2.5.** No Edit and no `Don't ask again` switch (D3 and G8, stated).
- **S2.6.** The card folds to `Tara sent the email to bu.sri@example.com` and `Approved by you · sent 05:41` (the decision time). View in Gmail and `Finished in` are left out (stated).
- **S3.3.** `Don't send` folds to `Tara's email to sari@example.com` · `Not sent · nothing left Gmail`. There is no Undo. The quick replies follow as rows.
- **S3.6 supervisor (nQiZd).**
  - Header 40, title 13/500. Task rows 36 with done, `stopped · 2 of 3` and `not started` in 11 mono.
  - Footer `Nothing sent was undone.` and Resume. No Discard (stated).
- **S3.4 note.** `You can leave. Tara's request will wait here and on Home.` (G2).
- **S4 error card (DSGWn).** The card is 358 wide, header 39 in the danger tint, body padding 14. Meta is the model in 11 mono; buttons are 46 and stacked. It's 193 high without the "SAID" block, as expected. Titles and actions by kind:

  | Kind | Title | Actions |
  |---|---|---|
  | No credits | `Out of credits` | `Switch to Nomi credits` · `Retry` |
  | Rate limited | `Rate limited` | `Retry` · `Try another model` |
  | Too long | `Chat too long` | `Start a new chat` · `Choose a model` |
  | Bad key | `Key rejected` | `Update key` → `/settings/models` · `Use Nomi credits` |
  | Provider down | `Provider down` | `Retry` · `Choose a model` |
  | Timed out | `Timed out` | `Retry` · `Try a faster model` |
  | Other | `Couldn't reply` | `Retry` |
  | Quota | `Out of Nomi credits` | `Upgrade plan` · `Use my own key` |

  The quota card also shows the `Free plan` tag and `… tokens used · resets 1 Dec`. No code, "SAID" block or cost appears anywhere.
- **S4 4.1.** `Gmail signed out` with `Reconnect Gmail` and `Skip this step`.
- **S4 5.1, 5.2, 6.2, 7.1.**
  - Offline banner, and the bubble reads `queued · sends when you're back` with Cancel.
  - `Back online. 1 queued message sent.` hides within 5 s.
  - `85% of Nomi credits used · resets 1 Dec` with Usage.
  - The signed-out alertdialog shows the kept draft and `kept`; Escape doesn't close it, and focus is on `Sign in again`.
- **Light theme.** Every captured step was checked in light; the tokens flip correctly.
- **Left out because the backend lacks them.** Each is in BACKEND_TODO ("Found while building the phone's chat flows") and AGENTS.md §9, so they aren't failures:
  - recommended answers and helper lines;
  - each step's owner and timing, the estimate and `Finished in`;
  - a run mode;
  - Edit, a scoped remember, View in Gmail, Undo and Discard;
  - structured failures and the content-filter kind;
  - Summarise & continue;
  - Web Push.
- **Wired later, though the backend has them.** The model sheet's key health, `{n}% left` and the masked key are stated in §9 as not wired yet. That's accepted as stated, but it's the next follow-up, because the backend already has them.

## Deviations

| # | Frame | What the build does (measured) | Contract / pen | Severity |
|---|---|---|---|---|
| 1 | S1.6 WcVCK (Run Bar Y7Nxi) | A proposed v1 plan's bar reads `4 steps · Rena` with `Run plan` (S2.1's bar, JMude), and `Plan ready · nothing runs yet` never appears. | S1.6: while `proposed`, `Plan ready · nothing runs yet` with `Review & run`. Ruled in answer 1 below. | high |
| 2 | S1.7 KCocj (Option 2) | `Research only` has no detail line. | run.option2: `{research steps}. Nothing shared.` Until steps have kinds, show `Nothing shared.` alone. The honesty line must be there. | high |
| 3 | S2.5 Yi1GU (preview) | The fields show in the order **Subject, To** (Subject at y 644, To at 689). | To first, then Subject (O3l98 at 0, BzXqT at 45). Who receives it is what the person approves. | high |
| 4 | S1.1–1.4 (Progress P2RJL) | Answered and current segments are all `#EDEDEF`. At 3 of 3, all three are `#EDEDEF`. | Done segments are `$nm-accent` `#7BE3A4`, the current one `#EDEDEF`, the rest `#2C2C31` (L42Ej, U8a64). | medium |
| 5 | S1.4 U8a64 (many of many) | Ticked rows get the accent fill (8%) and an accent border. The boxes are native checkboxes drawn in accent green, and unticked boxes are filled grey. | Rows stay `#2C2C31`-stroked with no fill. The box is 22, r6: ticked `#EDEDEF` with a 12 `#0A0A0B` check, unticked `#111113` with a `#2C2C31` stroke. Accent is reserved for the single choice. | medium |
| 6 | S3.4 HsE8N; S4 x8dgV, QMGjy (banners) | `PhoneCrewWorking`, the offline banner and the credits banner sit at the **top of the thread's scroll**. In a chat longer than a screen, they're off-screen: the offline banner was at y −350, and `1 crew working · Stop all` was only visible after scrolling up. | Contract S3.4: "a banner **under the nav bar**". The person must see that the crew is working and reach Stop all, and must see that they're offline, wherever they are in the thread. Pin these under the nav bar (sticky) on the phone. | medium |
| 7 | S3.4 HsE8N (nav sub) | With one active session, the nav sub shows the last replier (`Supervisor`). It switches to `{n} crew working` only when n > 1. | The nav-bar sub reads `{n} crew working` whenever a session is active (`1 crew working`). | medium |
| 8 | S3.6 nQiZd (footer); also S1–S2 sheets' 44 pt rule | `Resume` is drawn 92×**44**, so the footer is **64** high. | `Resume` is 93×30 and the footer 50. The contract says 44 pt targets **pad the hit area**; the visible control keeps the pen's size. | medium |
| 9 | S2.3 MKUyQ (form summary and sheet) | The thread's summary card shows the date as `2026-01-25 · 09:00` (ISO). In the sheet, the date and time values are Geist sans 16. | Dates are `25 Jan 2026` (content rules). Date and time values are **Geist Mono** 16 (vKSyn, JCtTB). | medium |
| 10 | PJcN8, gQ44i, MKUyQ, Yi1GU (button icons) | `Make the plan`, `Use {option}` and `Save` have no trailing icon. | Each primary has a 16 trailing icon in accent-ink (QFQVM). | low |
| 11 | U8a64 (spacing) | Many-of-many option gap is 8, and the question block starts 2 px lower. | Gap 6, body gap 6. | low |
| 12 | PJcN8 (accessibility) | The review rows are named `Change Question 1: Travel with Sari's car`. | `Change {topic}: {answer}`. With no short topic, use the question text, not "Question 1". | low |

### Not reached in this run (no failure recorded)

- **S1.3 default.** No `recommended` in the backend yet.
- **S2.2 running plan.** Needs a coding build.
- **S3.5 toast.** Covered by the e2e "elsewhere in the app"; not captured here.
- **S4 1.2–1.3, 2.1–2.2, 4.2, 5.3–5.4, 7.2.** Need a failing or throttled provider, a dropped stream, or a sign-in round trip.
- **S4 6.1 (not sent).** My route on `/api/sessions/*/messages` didn't catch the app's send, so the refusal wasn't forced.
- **S2.6 / S3.3 after a reload.** On reload, the decided card showed as pending again. That's a seeding artifact: the seeded agent session can't resume, so the backend never updates the block. It isn't a build fault, but recheck with a real `gmail.send` before merge.

These have component or unit tests per the build notes. Check them on the next pass, with a real provider where one is needed.

### Pen note for pen-designer (not a build deviation)

DSGWn's primary button `Switch to Nomi credits` uses the Lucide **sparkles** icon (TsoFP/QFQVM), and the build copies it. nomi-ux-rules bans sparkles. Swap it in the pen, for example for `refresh-cw` or `arrow-right-left`, and the build follows.

**Verdict: changes required.** #1–#3 are high and #4–#9 medium. After the fixes, re-run S1.1–S1.7, S2.3, S2.5, S3.4 (long chat), S3.6 and S4 5.1 at 390, dark and light.

## Answers to the engineer's questions

1. **The plan bar's label: one bar for a proposed plan.**
   - Version 1 reads `Plan ready · nothing runs yet` with `Review & run`.
   - A later version reads `Plan v{n} ready` with `Review & run` (as built).
   - `{n} steps · {owners}` stays as the run sheet's sub, where the person decides.
   - Why: the bar opens a sheet; it doesn't run anything. `Run plan` on the bar promises an action it doesn't take, and "nothing runs yet" is the honesty line.
   - S2.1's bar (jBK6g / JMude) is superseded. The contract's S2.1 row is amended to point at S1.6's bar.
2. **A generic form summary: yes.** Use the twin `Reminder` card only when the form *is* a reminder: its date or time and repeat fields map onto a reminder, as `New reminder` does. Every other form gets a generic summary card:
   - the same layout as the reminder card (title 13/500, values in 11 mono, the agent chip, a 30 edit button);
   - a neutral Lucide `clipboard-list` icon instead of the bell;
   - its button named `Edit {form title}`.

   A non-reminder form must never read `Edit reminder`. Put it in `ds/thread` (not a twin), with its docs entry and example. Dates in its values follow #9.
3. **Where ` · 1 default` sits: accepted in `ClarifyAnswered`'s mono summary.**
   - The twin's props are frozen, and the count stays on the same line, next to `Answered {n} questions`.
   - Two conditions:
     - the fold's accessible name includes it (`Answered 3 questions, 1 default`);
     - it's singular or plural as in the copy table.
   - Today it can't appear: there's no `recommended` yet. Re-check when BACKEND_TODO "Questions: a recommended answer" lands.
