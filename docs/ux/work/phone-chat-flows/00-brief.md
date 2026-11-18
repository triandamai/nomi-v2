---
slug: phone-chat-flows
opened: 16 Nov 2026
owner: Mario
scope: the phone versions of four chat flows the desktop already ships
---

# Brief: a chat's remaining flows on a phone

## What
`frontend-new` already builds these flows on a desktop. On a phone (below 768, `phoneView`)
they still render the desktop components inside the phone chat. Build them as the pen's
Mobile Chat flows draw them.

| Flow | Mobile dark | Mobile light | Desktop (built, for reference) |
|---|---|---|---|
| Clarify → plan | GCvfD | tq7Ht | S3yL5 / g11Ew |
| Plan → approve → act | G1tK6L | EI3k6 | HU6nc / mjZPs |
| Approvals & background work | xvntx | qhcNA | bWiWT / t16cU |
| Errors & recovery (all but scenario 6's file steps, which are UXP-001) | AyrHG | uuCam | TpuRc / Nud6J |

## Already on a phone (for context)
The tab bar and Chats, a chat's thread and New chat, and the reply and message sheets
(long-press, edit, dislike, share). The phone components so far are `PhoneComposer`,
`BottomSheet`, `SheetRow`, `SheetTiles`, `ActionSheet`, `PhoneMessageEditor`,
`PhoneDislikeSheet` and `PhoneShareSheet`.

## Constraints
- The backend API as it is. What it lacks goes to `frontend-new/BACKEND_TODO.md`.
- 390 wide, dark and light, 44pt targets, sheets instead of popovers.

## Gate 1
ux-lead: run the completeness test on the four mobile flows. Name the desktop frames
where a phone step leans on them. Return a verified contract if there are no P0 gaps;
otherwise write a proposal.
