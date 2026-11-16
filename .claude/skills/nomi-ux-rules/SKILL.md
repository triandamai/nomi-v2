---
name: nomi-ux-rules
description: Nomi's product, design and honesty rules for any screen, flow, state or UI copy. Preloaded into ux-lead, pen-designer and ui-engineer. Use whenever reviewing, designing or building anything a Nomi user can see.
user-invocable: false
---

# Nomi UX rules

## Who we design for
Indonesian builders, freelancers, UMKM owners and non-coders. Technical users read
English UI without friction and distrust marketing polish. Non-coders need every
error to say what happened and what to do next, in plain words.

## The promise
A serious tool, and your work belongs to you. Never "magic".

## Visual rules (checkable)
- One accent colour, under 10% of any screen. Neutrals do the work.
- No gradients, no glassmorphism, no glow, no 3D blobs, no emoji in UI chrome.
- Two type families max: sans for interface, mono for anything machine-produced
  (paths, IDs, tokens, costs, file names, sizes, logs). Max 4 sizes per screen.
- Sentence case for labels, buttons, headings. Tabular figures for scanned numbers.
- Every colour, spacing, type and radius value is a variable in `Nomi.pen`.
- Lucide icons only. Borders over shadows. Max two radius values.
- No bento grids, no three-card triplets, no centred hero in the app.

## Content rules
- No lorem ipsum. Indonesian context: `Rp 249.000`, `26 Jul 2026`, `bca-sep-2026.pdf`, Siti Rahayu.
- Numbers are consistent across frames of the same flow.
- Empty states say what to do next.

## Honesty rules (product-specific, never negotiable)
- Cost is always reachable. Estimate before any expensive action (big files, long runs).
- No fake progress bars. Unknown duration = elapsed time + current real step.
- Agent activity names the real tool and target: "Reading bca-sep-2026.pdf · page 4 of 12".
- The user always knows which provider and model will receive their data.
- Export/eject is never harder to find than upgrade.
- Banned in UI copy: magic, magical, effortless, "AI-powered" badge, sparkles.

## Every surface ships these states
1. Empty  2. Loading (layout skeleton)  3. Error (what, why, what to do)
4. Populated (realistic density)  5. Mobile (390 px)
Interactive elements also need: hover, focus-visible, active, disabled.
Desktop and mobile, light and dark. A flow is not done until all four exist.

## Flow completeness — the "missing part" test
For every flow, walk these and name the frame that covers each, or call it a gap:
- Entry: every way in (button, shortcut, drag, paste, deep link, mobile sheet).
- Before commit: validation, limits stated up front, cost/permission/provider shown.
- In progress: real progress, cancel, what happens if the user leaves.
- Failure: network, server, provider, permission, quota, unsupported input,
  partial success. Each with a recovery action.
- Result: success, partial result, and what the agent says about it.
- After: edit, undo, retry, delete, share, find it again later.
- Interrupts: offline, signed out, credits out, tab closed, second device.
- Access: keyboard path, focus order, screen-reader announcement for async changes.

## Pen file conventions (Nomi.pen)
- Screens: `App / <Area> / <Screen> — Dark|Light`.
- Flows: `Flow / <Area> — <Name>` (+ ` — Light`), mobile `Flow / Mobile <Area> — <Name>`.
- Each flow frame has a `Flow Header` (eyebrow, title, description) and numbered
  `Step N` frames with a caption: number / title / one-sentence rule.
- Reuse components (`Chat / *`, `Mobile / *`, `Workspace / *`, `Project / *`).
  New component only when no variant fits; name it `<Area> / <Thing> / <State>`.
- Never restyle Mario's reference frames. Extend; don't reinterpret.
