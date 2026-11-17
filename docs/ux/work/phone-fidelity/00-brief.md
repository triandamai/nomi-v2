---
slug: phone-fidelity
opened: 17 Nov 2026
owner: Mario
scope: the phone screens already on main, against the pen
---

# Brief: phone screens vs the pen

Mario: "there is many design not same dev with pen". Audit what's built and shipped on a phone
against its pen frames, then fix the deviations in code on this branch.

| Built | Pen flow (mobile dark / light) |
|---|---|
| Tab bar and Chats | Mobile Chat — Start & answer step 1 (FvlVs / r8Gz3) |
| A chat: nav bar, thread, bubbles, byline, action bar, composer; New chat | FvlVs / r8Gz3 steps 2–7 |
| Reply and message sheets, dislike, share, delete, edit | Like & dislike M0VaJC / mg56o, Share & delete yoVD6 / Pjc23, Message controls ctP7Q / m2cy3I |
| Projects: header, search, sort, filters, cards, sheets, detail, confirmations, move, loading and error | Mobile Projects flows: fGjF3, T9IbH, G4Mwvd, owsMq, H4ngT2, F2Tuh, EkEA9 and their light twins |

## How
Gate 3 style: the real build at 390 px in dark and light, step by step next to each pen frame.
Check copy, layout, sizes, spacing, type, icons, colours, states and order. Every deviation
gets a frame id, what the build does and what the pen draws. Deviations already listed in
AGENTS.md §9 are deliberate; check them, don't re-report them.
