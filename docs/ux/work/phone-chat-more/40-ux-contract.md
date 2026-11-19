---
work: phone-chat-more
status: verified
gate: 1
date: 19 Nov 2026
---

# UX contract: the rest of a chat on a phone

Gate 1 was run by ux-lead (Claude) against nomi-ux-rules, from the pen frames below. Every step
is drawn at 390 in dark and light, and the light flows repeat the dark ones' copy. There are no
P0 gaps: each flow has its entry, result, failure or limit, and the way back. The phone reuses
the desktop's logic (adapters/blocks.ts, ReplyBlocks, ChatView) and swaps in phone presentation.

## Rich replies (XO7DI / OkCH1)

| # | Step | Frame (dark) | What the phone shows | Exists? |
|---|---|---|---|---|
| 1 | Cart | uk8og | The list's title and count; each row with name, size, a stepper 26 tall with a 44pt hit area, and the price in mono. An out-of-stock row says so and offers its swap. Subtotal, delivery and total (mono), then Save as list and "Checkout on {store}". Quick replies follow. | Yes, the `cart` block. Checkout follows the block's link. |
| 2 | Selection | t1pFD0 | A checkbox row per item with its detail and amount on the right, a footer with "{n} selected · Rp {total} / month", then Skip and "Track {n}". | Yes, the `selection` block. |
| 3 | Link preview + mentions | Y7zTEB | The pasted link unfurls (host, title). Mentions sit on their own row as chips: a crew member hands off, a file opens. | Link: yes (`link`). Mention chips come from the reply's `@` and file names. |
| 4 | Code + diff | VL7b8 | The file, +/− counts, unwrapped lines that scroll sideways, then Reject and Apply full width at the bottom. | Code: yes (`code`). **The diff with Apply / Reject has no block: left out (BACKEND_TODO.md).** |
| 5 | Reminder and table | fbRRc | Crew-owned cards keep their tint. A wide table always switches to the stacked list. | Yes (`reminder`, `table`). |

## Manage chat & long threads (W3a8j / N2N1d)

| # | Step | Frame | What the phone shows | Exists? |
|---|---|---|---|---|
| 1 | Chat sheet | wayGm | Already built: the chat's sheet, Delete last and red. The subtitle is the pen's "14 messages · Dana" only once both are known (phone-fidelity §9). | Built. |
| 2 | Rename | sAwKS | An alert with the title selected, above the keyboard. | Built as the sheet's rename. |
| 3 | Move to a project | x73ZO | A sheet with Search projects, the projects to pick, and who gains access before you confirm. | Yes: the move endpoint and the project list (the desktop's MoveDialog). |
| 4 | Moved | NCfNM | The project under the title, and a banner: "Moved to {project}", "{workspace} members can see it now", Undo (moves it back). | Yes: move again. |
| 5 | Delete chat | zobzL | Already built (ActionSheet). | Built. |
| 6 | Deleted, with Undo | ZV5d5 | Back on Chats; the banner "Chat deleted", "{n} messages · undo within 10 s", Undo. | Yes: the held delete (`later.ts`). |
| 7 | Scrolled up | P1LyI | New replies don't pull you down. A pill above the composer counts them ("2 new replies"). | Yes (JumpToLatest). |
| 8 | New replies | juEHj | Jumping down lands on a NEW · {n} replies divider and what arrived. | Yes (NewDivider). |

## Hand-offs, limits & memory (YuWzO / M9B8RO)

| # | Step | Frame | What the phone shows | Exists? |
|---|---|---|---|---|
| 1 | Hand-off | zph7D | "Nomi handed this to Dana · Ask Nomi" above the reply, then FOLLOW UP replies. | Follow-ups: yes. **The hand-off line has no event (the reply only carries who answered): left out (BACKEND_TODO.md).** |
| 2 | Can't do → alternatives | agprG | One line on the limit ("I can't cancel subscriptions"), the reason, then each alternative as a row with who does it and when, or a link. | Yes (`boundary` with alternatives). |
| 3 | Alternative picked | rDKtH | The chosen row is highlighted, then the crew's confirmation follows. | Yes: an alternative sends its message. |
| 4 | Memory used and saved | crHO0 | "From memory: {fact}" under the byline with Manage, and "Saved to memory: {fact}" with Edit and Undo. | Used: yes. Saved: yes (`memory_saved`); Edit and Undo through the memory routes (`PUT`, `DELETE /api/memory/:id`). |
| 5 | Connect a tool | SYohQ | A sheet: "Connect {service}", why the member needs it, what access means (three lines), Connect with Google, and "I'll tell you my free weekends" (answer yourself). | Yes (`connect`). The answer-yourself line sends nothing: it focuses the composer. |
| 6 | Connected, answered | mfedO | After Google's screen the sheet is gone; "{service} connected · read-only", then the answer. | The connected line shows on return from the connect flow. |

## Rules for the build

- Sizes, copy and order follow the frames above. On a phone every 44pt target is a hit area around the control, never a bigger control.
- Only one layout is mounted (`phoneView`).
- Every string is in en and id.
- What's left out is listed in AGENTS.md §9 and BACKEND_TODO.md.

## Acceptance

1. On a phone each rich card is the phone's (stepper 26 with a 44 hit area, selection footer total, stacked table, code scrolling sideways).
2. Moving a chat from its sheet asks in a sheet, then shows the moved banner with Undo, which moves it back.
3. Deleting goes back to Chats with "Chat deleted" and Undo for 10 s.
4. Scrolled up, new replies show the pill above the composer; jumping lands on NEW · {n} replies.
5. A boundary reply lists its alternatives; picking one highlights it and sends it.
6. Memory used and saved show under the byline; Undo forgets a saved memory.
7. A connect block opens the phone's sheet; "answer it myself" focuses the composer.
