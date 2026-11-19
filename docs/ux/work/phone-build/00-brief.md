---
slug: phone-build
opened: 17 Nov 2026
owner: Mario
scope: a project's workspace (Build) on a phone
---

# Brief: Build on a phone

## What
A project's workspace already works on a desktop (AGENTS.md §9, "A project's workspace").
On a phone it is still the desktop layout squeezed. Build the pen's Mobile Build flows.

| Flow | Mobile dark | Mobile light | Desktop (built, for reference) |
|---|---|---|---|
| Sign in → live URL | zM0xh | cyXXE | o882RC / hifej |
| Workspace tools | tJh5G | zdXZh | U16wU / omR64 |
| Landing page | yY8qf | oLJ3q | WUiji / pcnAT |
| Roll back, limits & hand-over | Wv14s | yYAZV | pwkEe / Qt5C0 |
| Code, terminal & phone preview | r3IvZk | f48QD7 | uR6dq / kKWtu |
| Invite & build together | L8dgcS | ZTTgV | TnwQT / O0wCuQ |

## Already on a phone
Projects (list, sheets, detail), the chat (thread, composer, sheets), and the phone chat flows
(being built on `feat/frontend-new-v2-phone-chat-flows`). The phone components include
`AJXze` Mobile / Project Nav and `LX2ZI` Mobile / Workspace Switch.

## Constraints
- The backend API as it is. What it lacks goes to `frontend-new/BACKEND_TODO.md`.
- 390 wide, dark and light, 44-pt targets, sheets instead of popovers.

## Gate 1
ux-lead: run the completeness test on the six mobile flows. Name the desktop parts the phone reuses
as they are. Return a verified contract if there are no P0 gaps; otherwise write a proposal.
