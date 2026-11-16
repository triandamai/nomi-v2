---
slug: audit-chat-files
proposal: UXP-001
opened: 16 Nov 2026
owner: Mario
scope: P0 gaps — G3, G4, G6, G7, G9, G11, G12, G16
---

# Brief — chat files end to end (P0)

## What
Design, then verify, the P0 gaps of [UXP-001](../../proposals/UXP-001-chat-files-end-to-end.md)
in `Nomi.pen`: desktop and mobile, dark and light.

| Gap | What |
|---|---|
| G3 | Upload failed: Retry / Remove, Send locked with a hint |
| G4 | File type not supported: rejected at the chip before upload |
| G6 | Model can't read this file: warning, Switch model / Send text only |
| G7 | Cost and payer shown before a big read |
| G9 | File can't be read: locked / scanned / damaged / partial zip |
| G11 | Maya hands back a file: Agent File card + Generating + Expired |
| G12 | Preview a file and jump to a citation: desktop panel, mobile sheet |
| G16 | Files in a shared chat link: names only, opt-in toggle |

P1/P2 gaps (G1, G2, G5, G8, G10, G13, G14, G15, G17) are out of this round.

## Decisions (16 Nov 2026)
- D1: keep files until the chat is deleted, with per-file delete.
- D2: convert HEIC on upload (never a G4 rejection).
- D3: `Read as images` shows the estimate first, then runs on the usual payer.

## Gate 1
ux-lead's audit of 16 Nov 2026 is the proposal itself (verdict: gaps → UXP-001).

## Flows touched
Attachments, voice & limits (Iyq9S / ORByl / FLwdb / Fpgnv), Errors & recovery
(TpuRc / Nud6J / AyrHG / uuCam), Shared link (b8Flw8 / Wgdu2 / Yyjkb / e0FI49).

## Gate 2 loop
pen-designer → ux-lead gate 2, at most three rounds, then escalate to Mario.
