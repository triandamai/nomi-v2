---
work: phone-chat-flows
gate: 1
round: 1
result: pass
date: 17 Nov 2026
---

# Gate 1 — phone-chat-flows

Pass. Every step on the main path of the four phone flows is drawn at 390 in dark and light.
The light twins match their dark frames step for step and word for word: I compared the text of
all 47 step pairs and found no differences. No P0 gaps. The P1 and P2 gaps are in
[UXP-002](../../proposals/UXP-002-phone-chat-flows.md). They don't block the contract, which gives
an interim for each one. Scenario 6's file steps (S61SO, U6S0r, K2r8A and their light twins) are
out of scope and belong to UXP-001.

## Flows touched
| Flow | Desktop dark | Desktop light | Mobile dark | Mobile light | Mobile steps |
|---|---|---|---|---|---|
| Clarify → plan | S3yL5 | g11Ew | GCvfD | tq7Ht | 9 |
| Plan → approve → act | HU6nc | mjZPs | G1tK6L | EI3k6 | 6 |
| Approvals & background work | bWiWT | t16cU | xvntx | qhcNA | 6 |
| Errors & recovery (all but scenario 6 steps 4–6) | TpuRc | Nud6J | AyrHG | uuCam | 23 of 26 |

## Completeness walk (phone, dark · light)
| Moment | Covered by (frame id · step) | Gap id |
|---|---|---|
| Entry | The questions sheet opens on Nomi's ask (GCvfD qfsQ7 · 1). Draft plan in the thread with Run plan (G1tK6L jBK6g · 1). Plan bar to the run sheet (GCvfD WcVCK · 6 → KCocj · 7). Approval card to the full preview (G1tK6L XnozL · 4 → Yi1GU · 5). Form opens as a sheet (MKUyQ · 3). | Way back into a closed questions or form sheet: **G1 (P1)**. Opening a chat from Home's "needs you" with questions waiting is part of G1. |
| Before commit | Recommended answer preselected and Skip goes to the default (qfsQ7 · 1, U58RFC · 3). Review, where nothing runs yet (PJcN8 · 5). Run choice with the recommended option first (KCocj · 7). Full email preview before sending (Yi1GU · 5). Form validation (MKUyQ · 3). | "Don't ask again for Bu Sri" says more than the backend does: **G4 (P1)**. The plan copy offers "Reorder, reassign or remove steps", which nothing on the phone can do: **G3 (P2 + copy)**. |
| In progress | Plan running with steps and Stop plan (gQ44i · 2). Background banner with Stop all (xvntx HsE8N · 4). Retry countdown (AyrHG Obkjq). The reply's live step is reused from Ask → think → answer (FvlVs). | The "~3 min" estimate has no data behind it, so it's dropped (honesty, see contract). |
| Failure | Errors scenarios 1–5, 7, 8 and 6.1–6.3 (AyrHG). Each has a recovery action. A tool fails mid-plan (JHxIs). Form error (MKUyQ). | The approval call fails, or was already decided on another device: **G7 (P2)**. |
| Result | Plan v1 (WcVCK · 6), plan v2 (nyN4S · 9), plan finished (hgXxG · 6), approval folded to its outcome (hgXxG), Don't send (upMaX · 3), Supervisor report (nQiZd · 6), recovered and continued notes (kFkym, ABfCk, M7u2KP, M75Qz). | — |
| After | Keep planning, then v2 (eSUqU · 8 → nyN4S · 9). Resume after Stop all (nQiZd). Retry and Continue (AyrHG). | The "Answered 4 questions" fold does nothing on desktop or phone: **G6 (P2)**. Edit email, Undo on Don't send, Discard, Summarise & continue and View in Gmail need backend: **G8**. |
| Interrupts | Offline and back (x8dgV, HF7vy). Stream cut off (UTrnr). Signed out, then back and sent (hAhHg, M75Qz). Credits low or out (QMGjy, WVxJ4). Leaving the app while crew work (HsE8N). | Lock-screen push (hcv7x · 5) can't reach a phone browser, and "I'll notify you" over-promises: **G2 (P1)**. Half-answered questions lost when leaving: **G9 (P2)**. |
| Access | Not drawable. The contract sets focus order, focus traps, 44-pt targets and live announcements. | The sheet's Close is 30×30 (Iig1H/rQ4w3); it needs a 44-pt hit area: **G5 (P2)**. |

### Phone steps that lean on desktop frames
- **Live reply while the plan is made** (between PJcN8 · 5 and WcVCK · 6). Reuse the phone's existing
  reasoning and live step from Ask → think → answer (FvlVs), which already ships. No new layout.
- **Approval preview** (Yi1GU · 5). Phone only: the desktop shows the full email inline (HU6nc OQySw).
  It's a new sheet, drawn.
- **Notified elsewhere** (desktop bWiWT Z2dLp). The phone draws a lock screen (hcv7x) instead. The
  desktop toast can't be reused as is: the phone needs the top-anchored `Mobile / Toast` SCzar
  (`PhoneToast` in the DS). See G2.
- **Model picker for "Switch where it runs"** (MLk2U). The phone's model sheet already ships
  (PhoneComposer's chip). The grouping by who pays is desktop `modelMenu`, reused as is.
- **Supervisor report** (nQiZd) and **connect prompt** (JHxIs). The desktop `SupervisorStopped` and
  `ConnectPrompt` data paths can be reused as is. Only the phone card layout is new.
- **Signed out** (hAhHg). The desktop's `SessionExpired` and outbox logic can be reused as is. The phone
  presents it as a bottom sheet that can't be swiped away, which is drawn.

## Three lenses
**Product.** The phone flows answer "where am I, what's next, how do I recover" at every step. The
weak spots are the sheets: the questions and form sheets can be closed with no drawn way back
(G1), and two lines promise things the phone can't do yet: notifying a locked phone (G2) and
reordering plan steps (G3). Each one gets honest interim copy or behaviour in the contract. Pen copy slips
found: Clarify step 3's left button reads "Don't send" (U58RFC; it should be "Skip", as in step 1),
the folded approval says "sent 14:06" in a 15:24 thread (hgXxG), and the known "Team and invites"
banner slip (HsE8N, AGENTS.md §9 already says "Stop all").

**Business.** Trust holds: nothing leaves Nomi without an approval, Don't send states that nothing left
Gmail, and Stop all says nothing sent was undone. Two trust risks:
(1) "Don't ask again for Bu Sri" would actually set an always-allow rule for every `gmail.send` to
anyone, because the backend's `remember` is per tool. That's a privacy and abuse risk, so the contract leaves it out, as the
desktop does (G4).
(2) "I'll notify you" on a phone browser that has no push: the plan stalls on an approval nobody
sees, and that turns into support load (G2).
Cost: failures show no cost, as the rule requires, and "Use Nomi credits and retry" says who pays.
Upgrade (WVxJ4) sits next to "Use my own key", so the upgrade path stays fair.

**Tech.** The phone reuses the desktop's data paths 1:1. What the UI depends on:
- `questions` blocks (`intro`, `question`, `options[{label, detail}]`, `multiple`) with no
  `recommended` flag and no helper line. Answers go back as one message.
- `plan` blocks plus `GET …/agent-plans/:agent_session_id`. That gives versions, `state` (proposed, active, completed, blocked or updated)
  and a markdown checklist. There are no per-step owners, details or timings, and no run-mode API.
- `approval_request` plus `PUT …/messages/:id/approval {decision: approve|deny, remember}`. That returns 202, or 409 when
  the approval is no longer pending. There's no edited input and no undo.
- `stopped` blocks (tasks done, stopped or waiting), and the word "stop" sent as a message. Resume goes as a message too.
- Failures: a `run_failed` block (with an optional retry target), a `quota_notice`, and one localised sentence. There's no
  status code, provider reason or model in the block. The model comes from `GET /api/sessions/:id/model`
  and the masked key from `GET /api/llm/connections` (`api_key_masked`).
- Live: `/chat/:id/ws` (Delta, TurnFailed, AgentPhaseChanged with the `retrying` detail, Agent session
  started and ended, MessageUpdated). Socket close 4401 means signed out.
- Push goes to native apps only (FCM and APNs, `/api/devices`). There's no Web Push, so nothing reaches a phone browser while it's closed.
- Home's `needs_you` items exist (`GET /api/home`), and so does `/api/me/activity/ws`.
Backend lacks (each step is left out or replaced by the desktop's own fallback; listed in the contract
for BACKEND_TODO.md): approval edit, undo after deny, structured provider errors, summarise &
continue, run mode, per-step owner and detail, `recommended` and helper on questions, recipient-scoped
remember, a link to the sent mail, Web Push, and a content-filter failure kind.

## Result
pass → [40-ux-contract.md](40-ux-contract.md) (status: verified). Non-blocking gaps G1–G9 →
[UXP-002](../../proposals/UXP-002-phone-chat-flows.md).
