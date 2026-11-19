---
work: phone-chat-more
gate: 3
verdict: pass, with one item waiting on #318
date: 19 Nov 2026
---

# Gate 3: the rest of a chat on a phone

Checked against the pen at 390 with the real backend. The steps are captured by
`e2e/backend/phone-chat-more.e2e.ts`.

## Rich replies (XO7DI)

- The pen draws these with the desktop's own chat cards: Cart `jGiqJ`, Selection `AXjHQ`, Link preview `Jrzfh`, Code `N2sVwP`, Reminder `kBrpD`, Boundary `l2wP5f` and Table.
- On a phone they render as they are. Nothing is wider than the phone; a long code line scrolls inside its card, as the step's caption says.
- The diff card (Apply / Reject) is left out. There's no diff block (BACKEND_TODO.md).

## Manage chat & long threads (W3a8j)

- **Move (x73ZO), now the phone's own `PhoneMoveSheet`:** the 40 search, 54 rows with the name in mono, the chosen one filled with its check, Move chat at 46, and off until another project is picked.
- **Moved (NCfNM):** the nav bar names the project under the title. The toast says the chat stays private, not the pen's "members can see it now": the backend keeps a filed chat private (§9).
- **Rename, Delete and the jump pill / NEW divider:** already built.
- **Waiting on #318:** the pen draws Moved and Deleted as Mobile / Toast `SCzar` at the top of the screen. #318 adds the phone's toast region. Until it lands, these toasts sit where every toast does today (bottom on a phone).

## Crew, memory & connections (YuWzO)

- **Connect (SYohQ, mfedO):**
  - `PhoneConnectSheet` opens on its own for the latest reply that asks for a first connection.
  - Connect with Google posts to Settings' own action and comes back to the chat, which says "Google connected" once.
  - "I'll tell Tara myself" closes the sheet and focuses the composer.
  - The lines say only what holds for every app, not the pen's Calendar-only "Read-only access".
- **Can't do → alternatives (agprG, rDKtH):** the desktop's Boundary card, as the pen draws it.
- **Memory (crHO0):** "Saved to memory" now has Edit (Settings › Memory) and Undo, which forgets it (`DELETE …/memory/:id`).
- **Left out:** the hand-off line ("Nomi handed this to Dana · Ask Nomi"). There's no hand-off event (BACKEND_TODO.md).

## Found and fixed during the build

- A new message key collided with an existing one (`phone_connect_title`). The later one silently replaced the error card's "{service} signed out".
- The key is renamed, and the messages test now fails on any key named twice.
