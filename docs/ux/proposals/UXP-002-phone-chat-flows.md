---
id: UXP-002
title: Phone chat flows — sheets with a way back, honest background copy, pen slips
area: chat
author: ux-lead
status: proposed
flows: [GCvfD, tq7Ht, G1tK6L, EI3k6, xvntx, qhcNA, AyrHG, uuCam]
audited: 17 Nov 2026
---

# UXP-002 — Phone chat flows

## Why
The four phone chat flows are complete on their main paths in dark and light. The gate 1 contract
for `phone-chat-flows` is verified, and nothing here blocks it. What's left are the places where a
phone behaves differently from the desktop:
- Sheets can be closed and leave no way back.
- Two lines promise things a phone browser can't do yet: notifying a locked phone, and reordering
  plan steps.
- One toggle grants far more than it says.
- A few copy slips.

Each gap has an interim in the contract (`docs/ux/work/phone-chat-flows/40-ux-contract.md`). This
proposal is so pen-designer can draw the final version.

All gaps are P1 or P2. There are no P0s.

## Already covered (don't redesign)
- **GCvfD / tq7Ht, steps 1–9.** The questions sheet, your own answer, skip to the default, scope,
  review, the plan in the thread, the run sheet, keep planning, and v2.
- **G1tK6L / EI3k6, steps 1–6.** Draft, running and decision, the form sheet with its error, the
  approval card, the full preview, and done.
- **xvntx / qhcNA, steps 3, 4 and 6.** Don't send, the background banner, and Stop all.
- **AyrHG / uuCam.** Scenarios 1–5, 7 and 8, and scenario 6 steps 1–3.

## Gaps

### G1 · A closed questions or form sheet has no way back · P1
- **Scenario:** the person closes the questions sheet (qfsQ7) or the form sheet (MKUyQ) with Close
  or a swipe, or opens a chat later from Home's "needs you" while questions are still waiting.
- **Expected:** the questions get the sticky bar above the composer, the same bar as WcVCK step 6.
  It shows `{n} questions waiting` and `Answer`, and Answer reopens the sheet on the question the
  person left. A form stays in the thread as its summary card with `Open`. A chat that opens with
  unanswered questions opens the sheet on its own.
- **Copy:** `{n} questions waiting` · `Answer` · `Open`
- **States:** parked questions; parked form; reopened with earlier answers kept.
- **Reuse:** the run bar in WcVCK; `Mobile / Approval Card` MOzBb's compact pattern for the form
  card. New: `Mobile / Pending Bar / Questions`.
- **Accept when:** dark and light steps exist after step 1 (GCvfD) and step 3 (G1tK6L) showing the
  parked state, and Answer returns to the same question.

### G2 · Notifications on a phone browser · P1 (honesty)
- **Scenario:** crew work in the background. The person leaves the chat or closes the tab.
- **Problem:**
  - hcv7x (lock screen) needs a push. The backend pushes only to native apps (FCM and APNs), so a
    phone browser never gets one.
  - HsE8N's line `You can leave. I'll notify you when Tara needs you.` over-promises. The plan then
    stalls on an approval nobody sees.
- **Expected:**
  - The HsE8N line becomes `You can leave. {Agent}'s request will wait here and on Home.`
  - While the person is elsewhere in the app, a top toast says
    `{chat title}: {Agent} needs your OK`, with `Open`.
  - Keep hcv7x as the native and Web Push picture, labelled as such.
- **Reuse:** `Mobile / Toast` SCzar, which is `PhoneToast` in the DS. Desktop Z2dLp is the reference.
- **Tech needs:** Web Push (VAPID plus a service worker) for the phone browser. On iOS it works only
  for an installed PWA. See decision D2.
- **Accept when:** a new step 5a in xvntx / qhcNA shows the in-app toast over another screen, and
  step 4's line is updated in both themes.

### G3 · Plan steps can't be reordered, reassigned or removed on a phone · P2 (copy now)
- **Scenario:** jBK6g's caption says "Drag and remove move to long-press on mobile", and the plan
  cards say `Reorder, reassign or remove steps…`. Nothing in the pen draws that long-press sheet,
  and the backend only ticks steps off.
- **Expected now:** the intro reads `Tell Nomi what to change. Nothing runs until you say so.` in
  WcVCK, KCocj, eSUqU, nyN4S and jBK6g, and their light twins.
- **Expected later:** once a backend plan-edit API exists, a long-press step sheet with
  `Move up` · `Move down` · `Give to…` · `Remove step`.
- **Reuse:** `Mobile / Action Sheet` WBVBC.
- **Accept when:** no phone frame promises step editing that isn't drawn.

### G4 · "Don't ask again for Bu Sri" grants more than it says · P1 (trust)
- **Scenario:** Yi1GU's toggle.
- **Problem:** the backend's `remember` sets an always-allow rule for the whole tool. That means
  every `gmail.send`, to anyone.
- **Expected:** leave the toggle out, as the desktop does, until the backend can scope it to the
  recipient. If Mario wants it now, the label must say what it does:
  `Don't ask before {Agent} sends email`, with the sub `You can change this in Crew → {Agent} → Permissions.`
- **Accept when:** Yi1GU / pLM5s show no toggle, or show the honest wording.

### G5 · Pen slips · P2
- **U58RFC / aQV02:** the left button says `Don't send`. It should say `Skip`, as in step 1.
- **hgXxG / jidyu:** `sent 14:06` in a 15:24 thread. It should be `sent 15:24`.
- **HsE8N / i3AxZ:** the banner button reads `Team and invites`. It should read `Stop all` (already
  in AGENTS.md §9).
- **Sheet Close (Iig1H/rQ4w3):** 30×30. Give it a 44-pt hit area. The visual can stay.
- **i3AxZ / HsE8N:** the plan counts `2 of 5` but draws 4 rows. Draw 5 rows to match S1's v2 plan.
- **HsE8N:** `~3 min` has no source. Remove it, under the honesty rule against guessed durations.

### G6 · Looking at answers after the plan · P2
- **Scenario:** the person taps `Answered 4 questions · 1 default` (WcVCK).
- **Expected:** a read-only sheet titled `Your answers`, with the same rows as PJcN8. Its foot says
  `To change an answer, tell Nomi in a message.`
- **Reuse:** PJcN8's sheet.

### G7 · Approval failed, or decided on another device · P2
- **Expected:** a failed call keeps the card pending and shows `PhoneToast` with
  `The decision couldn't be recorded` and `Try again`. A decision made elsewhere folds the card to
  its outcome, with the meta `Approved on another device · {HH:MM}`.
- **Reuse:** hgXxG's folded line.

### G8 · Steps drawn without backend · P2 (backend first, no design work)
These are left out of the build until the backend exists:
- mIUPn and tRxJl (Edit the email, Sent with your edits)
- upMaX's `Undo`
- nQiZd's `Discard`
- Vy4HC's `Summarise & continue`, and J08wT
- hgXxG's `View in Gmail`
- The error card's code badge and `{PROVIDER} SAID` block
- IUA3P's content-filter kind
- Per-step owner and detail in plans

The entries are listed in the contract's BACKEND_TODO section. No redesign is needed. Once the
backend lands, ux-lead re-runs gate 1 for those steps.

### G9 · Half-answered questions are lost on leaving · P2
- **Expected:** answers given so far are kept on this device per chat, like the outbox. Reopening
  shows them selected, and the sheet's progress shows where the person stopped.
- **Accept when:** leaving at question 3 of 4 and coming back opens question 3 with answers 1–2
  kept.

## Open decisions for Mario
- **D1 · The run choice.** It's sent as a message (the contract's default), or it waits for a
  run-mode API. ux-lead recommends sending it as a message.
- **D2 · Web Push.** Add it now (backend and service worker), or later. ux-lead recommends honest
  copy plus the in-app toast now, and Web Push as a backend item.
- **D3 · "Don't ask again".** Leave it out, use the tool-wide honest wording, or wait for recipient
  scope. ux-lead recommends leaving it out.
- **D4 · `Something else…` on every single-choice question.** ux-lead recommends yes, on desktop and
  phone alike.
- **D5 · The deny label.** ux-lead recommends `Don't send` for tools that send something and
  `Decline` for the rest.
