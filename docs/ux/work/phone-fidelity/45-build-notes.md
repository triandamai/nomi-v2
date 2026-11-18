---
work: phone-fidelity
gate: build
date: 18 Nov 2026
source: 50-build-verify.md (gate 3, round 1, chat flows only)
---

# Build notes: phone chat screens vs the pen

Built from the pen values recorded in `50-build-verify.md` only: pen.dev was unavailable (the plan's
usage limit), so no frame was opened again. Where the audit doesn't give a value the fix needs, the
item is marked **needs the pen** and left as it is. Projects wasn't audited and isn't touched.

Twin files (`src/lib/ds/{build,chat,conversation,mobile}`) are unchanged: where the fix lands in a
twin (Mobile / Nav Bar, Mobile / User Message, Suggestion Row), the chat's screens use a new
component of their own (`thread/PhoneChatNavBar`, `thread/PhoneMessageBubble`,
`thread/PhoneStarters`), each with its docs entry and live example.

## Tally

| | Count | Items |
|---|---|---|
| Fixed | 28 | 3, 4, 5, 6, 7, 8, 10, 12, 14a, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25, 26, 27, 28, 29, 30, 32, 33, 34, 35 |
| Left out (with the reason, in AGENTS.md §9) | 7 | 1, 9, 2, 11, 14b, 24, Chats "Name:" prefix |
| Needs the pen | 2 | 14c, 31 |
| Not a deviation | 1 | 13 |

Item 14 is three rows: Try another model (14a, fixed), Save to memory (14b, left out), Delete
reply (14c, needs the pen).

## Per deviation

### A chat: thread, nav bar, composer
1. **Composer attach (+) and mic: left out.** The backend takes files and voice notes
   (`POST /api/attachments`, `X-Attachment-Kind: voice` with the browser's transcript), but no v2
   composer sends attachments: the desktop's `ChatView` doesn't pass `onattach`/`onrecord` to
   `ChatComposer` either (only the old app does). Drawing the buttons without the upload, the
   attach sheet, the chips and the recorder would draw them dead, and the audit has no values for
   those states (they need the pen too). `PhoneComposer` already draws both at the pen's sizes once
   it gets the handlers (38 round `surface-2` Plus at 20 in `text-2`, 8 to the pill, the 20 mic at
   its end). Listed in §9 (UXP-001: both composers together).
2. **"Thinking" on a finished reply: left out.** The app says "Thought for Ns" for a reply it timed
   live. After a reload the data has no duration: the `reasoning` block is `{ kind, text }` and
   nothing else. BACKEND_TODO "How long a reply thought" proposes `duration_ms`; §9 says so.
3. **Date divider: fixed.** On a phone a thread of one day draws no divider; two days or more keep
   them. `ChatView`; backend e2e asserts no "Today" in a one-day thread.
4. **Nav bar mascot 20: fixed.** `thread/PhoneChatNavBar` (the twin `mobile/NavBar` keeps 16), used by
   a chat and New chat. Test: the who row is 20 tall, the mascot 20.
5. **Composer focus border: fixed.** The pill keeps `border-strong` while typing
   (`PhoneComposer`). Test: focused, the border is `border-strong`.
6. **User bubble width: fixed.** `thread/PhoneMessageBubble`: text at most 250, bubble 278 (the twin
   keeps 280). Test: a wrapped bubble is 278 wide.

### New chat
7. **Starters: fixed.** The pen's four (`phoneStarters` in `adapters/chat.ts`): "Build a menu page
   for my warung → koda", "Turn a photo of my price list into an order form → maya", "What can I
   build in 10 minutes? → nomi", "Remind me to check pre-orders at 4pm → rena", each left out when
   its owner isn't on the crew, as the desktop's are. en + id. Unit test, and the backend e2e sees
   Nomi's.
8. **Starter label 15, wrapping: fixed.** `thread/PhoneStarters` (Suggestion Row is the old app's
   twin at 13). Test: the label is 15 in `text`, a two-line row is 58.
9. Same as 1: left out.

### Chat sheet and the Chats sheet
10. **"14 messages · Dana": fixed.** The chat's sheet says "N messages · Name" (who answers). The
    count shows only once every message is loaded; a long chat with older pages says the name only,
    so the count is never short. Backend e2e asserts `N messages · Nomi`.
11. **Chats long-press subtitle: left out.** `GET /api/sessions` gives neither a message count nor
    who last answered. BACKEND_TODO "How many messages each chat holds" (new) and "Who last answered
    in each chat" (existing); §9 says the subtitle is when the chat last moved.
12. **Export chevron: fixed.** The Export row has "md" and the chevron (`SheetRow` `more`).
13. **Delete chat icon: not a deviation.** In `@lucide/svelte` 1.63 `trash-2` is an alias of
    `trash` (`trash-2.js` re-exports `trash.svelte`, whose glyph has the two inner lines). The build
    draws the pen's glyph; only the class name the audit read says "trash".

### Reply sheet
14a. **Try another model: fixed.** Under Regenerate, while the latest reply can run again and the
     chat has another model: cpu, the first other model as its meta (mono), and the chevron; it
     opens Regenerate's sheet, whose last group is the other models. The order of the rows (after
     Regenerate, before Edit my question) isn't in the audit: confirm with the pen.
14b. **Save to memory: left out.** Nothing in the backend creates a memory (`/api/memory` lists,
     edits, forgets, confirms). BACKEND_TODO "Save a reply to memory"; §9.
14c. **Delete reply: needs the pen.** The backend has it (`GET …/messages/:id/delete` for what goes
     with it, then `DELETE …/messages/:id`); the desktop doesn't use it yet, and the chat's API proxy
     (`lib/server/chatRoutes.ts`) would need both routes. The audit gives the row (divider, trash-2,
     danger) but not what follows the tap (a confirmation naming the cards and records that go, or
     an undo): the pen's frame for that is needed before building it.
15. **Quick tiles: fixed.** `SheetTiles`: 68 tall, icon 6 over the label, the label `text-2` (accent
    when on), the row [0,16,10,16]. Width stays 83.5 at 390: four tiles 8 apart in 358 can't be 84
    each (the audit's 84 would need 360). Test.
16. **Regenerate chevron: fixed.** Removed.

### Regenerate sheet
17. **Subtitle: fixed.** "Each result is kept as a version" (en + id).
18. **With another model: fixed.** The mono label "WITH ANOTHER MODEL" (`regen_models`, upper case in
    CSS, styled as the model sheet's group labels: the audit gives no label metrics), then each other
    model as "claude-sonnet · your key" / "deepseek-flash · Nomi credits" with the cpu icon. Picking
    one sets the chat's model (`PUT …/model`) and then regenerates (`again`): the backend runs the
    turn on the chat's model, as BACKEND_TODO's regenerate entry says. The chat stays on that
    model afterwards, and the composer's chip shows it. Backend e2e asserts the group shows exactly
    when the account has another enabled model.

### Dislike sheet
19. **Six reasons: fixed.** "Unsafe or harmful" is gone from the phone's sheet. Checked first: the
    backend's `ALLOWED_REASONS` (`routes/sessions.rs`) is an allow-list, so any subset is accepted,
    and nothing requires `unsafe`; the desktop's `DislikeFeedback` keeps it. Test and backend e2e:
    the six, in the pen's order.
20. **Chips 15, unselected `text-2`: fixed.** Test.
21. **Note field: fixed.** `bg`, radius 12, 12 in (11 + the 1px border), 84 tall. Test.
22. **Gap 14, label 15: fixed.** Test.

### Share sheet
23. **App row: fixed.** Tiles 65 wide and 8 apart, icon 22, label 11. More (ellipsis) is the fifth
    tile wherever the browser has a share sheet (`navigator.share`: every phone browser). Headless
    Chromium has none, which is why the audit saw four. Test (with `navigator.share` present).
24. **Footnote: left out (honest copy), lower case matched.** The pen's "· revoke in Settings ›
    Data" promises a page v2 doesn't have, so the footnote says only when it expires, in lower case
    as drawn ("expires in 7 days"). §9.
25. **Helper: fixed.** "Read-only, no sign-in" (`share_anyone_help_phone`; the desktop's
    SharePanel keeps its own line). Test and backend e2e.
26. **Preview card: fixed.** `bg`, a 1px `border` stroke (outline, so it takes no room), radius 12,
    padding 12, 14 / 1.45 (20) in `text-2`. Not checked, as in the audit: the masked "Rp •••"
    (the fake reply has no amounts; the backend masks the excerpt). Test.
27. **Segment: fixed.** 40 tall, 3 in, radius 10, gap 3; the chosen half `bg`, 34 tall, 600. Test.
28. **Gap 12, labels 15: fixed.** Test.

### Delete chat
29. **Copy: fixed.** The phone's ActionSheet says "{count} messages and their cards are removed.
    Reminders, money records and sent emails stay." (`chat_delete_body_phone`, en + id). The
    desktop's dialog keeps its own wording. Backend e2e.
30. **Line height 1.4: fixed.** `ActionSheet`'s message is 13 / 18. Test.

### Your message and editing
31. **Lifted bubble: needs the pen.** The audit says the pen lifts the bubble above the scrim,
    right-aligned just over the sheet, with a `border-strong` stroke and a shadow, but not how far
    above the sheet, how wide, or which shadow (the design system has one, `shadow-float`). Left as
    it is until those are read; §9 says so.
32. **Select text: fixed.** The third row (text-select icon, "Select text", en + id) selects your
    message where it stands, once the sheet is gone, so the phone's own handles and menu take over
    (the long press itself can't, it opens the sheet). Backend e2e checks the selection.
33. **No ring on Close: fixed.** `BottomSheet` opens with the sheet itself focused (not its close),
    unless something in it is marked autofocus; Tab goes on to the close and the rows. Test and
    backend e2e.
34. **Editor buttons: fixed.** Cancel and Send 34 tall, 14 in, fully round (17); Cancel in a
    `border-strong` stroke at 15 / 400 in `text`; Send 15 / 500 with a 14 arrow. Test.
35. **Editor radius 16, reply at 0.35: fixed.** The phone's reply dims to 0.35 (the desktop keeps
    0.4). Tests.

### Chats list
- **"Tara:" before a row's line: left out.** The list doesn't say who answered last
  (BACKEND_TODO "Who last answered in each chat"); now named in §9.

## Needs the pen
- 14c: Delete reply: what follows the tap (confirmation or undo), its copy and layout.
- 31: The lifted bubble: its gap above the sheet, its width and position, the shadow.
- To confirm when pen.dev is back: the reply sheet's row order with Try another model (14a), the
  "WITH ANOTHER MODEL" label's paddings (18), and the attach sheet, chips and recorder states for
  item 1.

## How to reach each fix in the running app (390 wide)
- New chat: Chats → New chat (starters, nav bar mascot, composer).
- A chat: open any chat; More in the nav bar is the chat's sheet (subtitle, Export).
- A reply's sheet: a reply's More, or a long press on it (tiles, Regenerate, Try another model);
  Regenerate opens its sheet; Bad opens the dislike sheet; Share opens the share sheet.
- Your message: a long press on it (Select text, Edit and resend → the editor).
- Delete chat: the chat's sheet → Delete chat.
