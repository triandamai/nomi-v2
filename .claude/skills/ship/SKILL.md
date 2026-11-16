---
name: ship
description: Run Nomi's UX-gated pipeline for any user-facing change - ux-lead review, pen-designer for missing UX, ux-lead verification, ui-engineer build, ux-lead build check.
argument-hint: "[what to build, e.g. retry a failed attachment upload]"
disable-model-invocation: true
---

# /ship — UX-gated delivery

Request: $ARGUMENTS

You are the orchestrator. You don't review, design or code yourself; you run the
three agents in order, pass files between them, and stop at every gate that fails.

## 0. Open the work item
1. Make a slug from the request (`chat-attachment-retry`). Create
   `docs/ux/work/<slug>/` and write `00-brief.md`: the request in the user's words,
   who it's for, why now, what's out of scope, linked issue if any.
2. Write the slug to `.claude/state/active-work`.
3. Tell the user the slug and that gate 1 is starting.

## 1. Gate 1 — UX review
Delegate to `ux-lead`: "Gate 1 for <slug>."
- Returns a verified contract → go to step 3.
- Returns `NEEDS_DESIGN UXP-<nnn>` → show the user the gap list (ids, priorities,
  one line each) and ask: design all, P0 only, or adjust. Wait for the answer.

## 2. Design loop
1. Delegate to `pen-designer`: "Design UXP-<nnn> for <slug>, scope: <answer>."
2. Delegate to `ux-lead`: "Gate 2 for <slug>."
3. `DESIGN_CHANGES_REQUIRED` → back to 2.1 with the listed fixes. After three
   rounds, stop and show Mario the open items.
4. Verified → continue.

## 3. Build
Delegate to `ui-engineer`: "Build <slug> from the verified contract."
- `BLOCKED` or open `Question for ux-lead` → route the question to `ux-lead`
  (it may update the contract or open a new proposal), then resume.

## 4. Gate 3 — build check
Ask the user to start the local dev server if it isn't running, then delegate to
`ux-lead`: "Gate 3 for <slug> at <local url>."
- Deviations → back to `ui-engineer` with the list. Repeat until pass.

## 5. Close
1. Update `docs/ux/flow-registry.md` status to `shipped`.
2. Clear `.claude/state/active-work`.
3. Commit and open the PR as the human author: Conventional Commits, PR body
   lists the contract path and frame ids. No AI attribution in any commit,
   trailer, PR title, PR body or comment.
