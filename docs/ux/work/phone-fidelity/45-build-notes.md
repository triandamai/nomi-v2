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

Update, 18 Nov: pen.dev is back. 14c and 31 are built from their frames, 14a and 18 confirmed, and
the Projects part of the audit is fixed (the second half of this file).

Twin files (`src/lib/ds/{build,chat,conversation,mobile}`) are unchanged: where the fix lands in a
twin (Mobile / Nav Bar, Mobile / User Message, Suggestion Row), the chat's screens use a new
component of their own (`thread/PhoneChatNavBar`, `thread/PhoneMessageBubble`,
`thread/PhoneStarters`), each with its docs entry and live example.

## Tally

| | Count | Items |
|---|---|---|
| Fixed | 30 | 3, 4, 5, 6, 7, 8, 10, 12, 14a, 14c, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35 |
| Left out (with the reason, in AGENTS.md §9) | 7 | 1, 9, 2, 11, 14b, 24, Chats "Name:" prefix |
| Not a deviation | 1 | 13 |

Item 14 is three rows: Try another model (14a, fixed), Save to memory (14b, left out), Delete
reply (14c, fixed once the pen was back).

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
     opens Regenerate's sheet, whose last group is the other models. Confirmed with the pen (SwByh):
     Regenerate, Try another model, Edit my question (Save to memory, left out), then Delete reply.
14b. **Save to memory: left out.** Nothing in the backend creates a memory (`/api/memory` lists,
     edits, forgets, confirms). BACKEND_TODO "Save a reply to memory"; §9.
14c. **Delete reply: fixed** (pen read 18 Nov: SwByh XPHtc, Share & delete EQUZo and y6rgt). The
     row sits last under a 1px line (trash-2, danger), for any reply except the one being written.
     The tap reads the backend's preview (`GET …/messages/:id/delete`), then asks in an
     `ActionSheet`: "Delete this reply?", the pen's "Your message stays so you can ask again.
     Things the crew already did aren't undone." where the reply's cards made nothing, else "What
     its cards made goes with it: {items}" (the backend deletes those reminders and plans), Delete
     reply, Cancel. Deleting puts `DeletedReply` in the reply's place with Undo, and the toast
     "Reply deleted · Undo within 10 seconds"; the `DELETE` goes after ten seconds (or at once on
     `pagehide`), as a deleted chat's does, since the backend can't restore one. The placeholder
     has no Regenerate (a deleted reply can't be asked again in its place). §9 and BACKEND_TODO
     "Bring a deleted reply back". Proxy routes in `chatRoutes.ts` (unit test), backend e2e: asked,
     undone, then deleted for good.
15. **Quick tiles: fixed.** `SheetTiles`: 68 tall, icon 6 over the label, the label `text-2` (accent
    when on), the row [0,16,10,16]. Width stays 83.5 at 390: four tiles 8 apart in 358 can't be 84
    each (the audit's 84 would need 360). Test.
16. **Regenerate chevron: fixed.** Removed.

### Regenerate sheet
17. **Subtitle: fixed.** "Each result is kept as a version" (en + id).
18. **With another model: fixed.** The mono label "WITH ANOTHER MODEL" (`regen_models`, upper case in
    CSS, styled as the model sheet's group labels; confirmed with the pen, lVDqs: [12,20,4,20], 30 tall), then each other
    model as the pen draws it (lVDqs): its name as the label, "your key" / "Nomi credits" as the mono
    meta, cpu for a key and sparkles for Nomi credits (was one label "name · source" in cpu). Picking
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
31. **Lifted bubble: fixed** (pen read 18 Nov: nzT7f). While your message's sheet is open the bubble
    is held over the scrim, right-aligned, 12 above the sheet and 16 from the edge
    ([0,16,12,16]), 278 wide at most, surface-2 with a 1px border-strong stroke, 18 corners and the
    shadow 0 6 18 in `shadow` (new token `shadow-lift`, allowed in the class lint). `BottomSheet`
    takes a `lifted` snippet (a tap beside it closes the sheet like the scrim); `PhoneMessageBubble`
    takes `lifted`; the bubble's place in the thread stays, empty, until the sheet closes. Component
    test (12 / 16 / 278, stroke, shadow) and backend e2e.
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
Nothing left: 14c and 31 are built from the pen, and 14a's row order and 18's label paddings are
confirmed. Item 1's attach sheet, chips and recorder wait for PR #305 (attach) and the voice work.

## How to reach each fix in the running app (390 wide)
- New chat: Chats → New chat (starters, nav bar mascot, composer).
- A chat: open any chat; More in the nav bar is the chat's sheet (subtitle, Export).
- A reply's sheet: a reply's More, or a long press on it (tiles, Regenerate, Try another model);
  Regenerate opens its sheet; Bad opens the dislike sheet; Share opens the share sheet.
- Your message: a long press on it (Select text, Edit and resend → the editor).
- Delete chat: the chat's sheet → Delete chat.
- Delete reply: a reply's sheet → Delete reply → the action sheet → the placeholder with Undo.
- The lifted bubble: a long press on your message.

# Projects (gate 3, the Projects part of 50-build-verify.md)

Built 18 Nov 2026 with the pen back: every frame below was read with `Get`/`ctx.bounds` before
coding. 29 deviations: **27 fixed**, **2 fixed in part** (P6, M12: the layout fixed, a line of the
pen's copy left out as untrue, §9). Nothing touches the twins: new phone parts are
`projects/PhoneProjectEmpty`, `PhoneProjectMenu`, `PhoneWorkspaceList`, `PhoneAskBar`, each with its
docs entry and live example; changed ones had theirs updated. Tests: `projects/phoneFidelity.svelte.test.ts`
(one per frame), `projects/list.test.ts`, `projects/about.test.ts`, `server/projects.test.ts`, and
the backend phone e2e (sort sheet, search, no match, Workspaces sheet, ask bar).

| # | Frame | Deviation | Status |
|---|---|---|---|
| P1 | oFdFf | Searching doesn't change the bar | Fixed: 1px `focus` stroke (no glow), 20 round clear (text-3, x 12 in surface-2) once typed, a text Cancel (16, text) in Sort's place that empties and blurs; the field is 299 beside it. |
| P2 | OC2PS | No match is the desktop's | Fixed: `PhoneProjectEmpty`, no box, [40,24] gap 10, 48 tile r14 surface-2 with the 22 glyph in text-2, 17/600, 14/1.45, the 40 pill with x. Also used for an empty filter (uDnnd, circle-check for Needs you) and couldn't-load. |
| P3 | M7SCRS | Sort sheet order and labels | Fixed: Updated, "Status · needs you first", "Name · A–Z", Created (phone-only labels; the desktop menu keeps its own). |
| P4 | rbozn | Move title repeats the subtitle | Fixed: "Move {name}" over "To another workspace". |
| P5 | KEIJW | "current · current · only you" | Fixed: the meta is said once ("current · 3 members"). |
| P6 | AHDzi | Delete sheet copy | Fixed in part: "The live URL stops working and the database is removed." as drawn; the pen's "Your code stays in your account for 30 days" is untrue (the backend's `DELETE /api/projects/:id` removes the files, every publish and the database at once), so: "Its code goes too, for good: export it first if you want a copy. Its chat stays." §9. |
| P7 | a9NrP | Workspaces are rows, not cards | Fixed: `PhoneWorkspaceList`, cards 64 [12,14] r14 in [4,16] gap 8, tile 36 r9 on bg, name 16/500, the chosen one surface-2 with the 20 accent check. PERSONAL and Join stay out (§9). |
| P8 | C8evkZ | Boundary note missing | Fixed: bg box, 14 in, r12, gap 10; users / lock at 16, 14/500 titles over 13 lines. "Shared crew templates" left out, as the desktop switcher (§9). |
| P9 | kB2Dt | Ask bar | Fixed: `PhoneAskBar`, wrap [10,16,4,16] on bg under a top stroke (66), the 52 round bar (surface, border-strong, float shadow, [0,6,0,14] gap 10), Koda's 20 mascot, placeholder 15 in text-3, the 40 surface-2 send (arrow-up 18) once typed (eiQ7z). The mic is left out until voice notes (§9); typed text 16pt so iOS doesn't zoom (§9). |
| P10 | AL711 | Status line is the footer | Fixed: `detailLine`: "deploy #6 failed · #5 still live" (the latest production publish failed: its number, and the live one's), "deploy #6 failed" with nothing live, "Koda is changing it · #15 stays live" with the warning dot, the live line once live, else the footer. The failed number comes from the deploy overview (`failedNumber` added to `ProjectAbout`). "1 of 3" isn't given (§9, BACKEND_TODO "a card's progress"). |
| P11 | — | "deploy #0" | Fixed: a live number of 0 or none falls back to the card's count, and with neither the part is left out ("live · 2d ago"). The seeded publish had `files_version` 0. |
| M1 | wDi32 | Filter counts ignore the search | Fixed: `filterCounts(cards, query)` for the phone's chips (desktop toolbar unchanged). |
| M2 | AXKpW | Pill initial in mono | Fixed: sans 12/600. |
| M3 | G8cFz | Card sheet subtitle | Fixed: the project's state (`detailLine`), its deploys read when the sheet opens so "live · deploy #15 · 2d ago" gets its age. |
| M4 | R4a7xS | Icons | Fixed: app-window, arrow-up-right, power (`PhoneProjectMenu`). Delete's `trash-2` was already right: in @lucide/svelte 1.63 `trash-2` is an alias of `trash` (as chat item 13). |
| M5 | XDW6C, ZRj78 | Separators | Fixed: 9 tall, [4,20], the line inset to 350; the phone's groups (open · change and share · remove). |
| M6 | VvKNR, gvrNK, nEjEw | No chevrons | Fixed: Move to workspace…, Share… and Settings end in the 16 chevron. |
| M7 | I5UHw | Move cards | Fixed: 68 tall, 14 in, tile 36 r9 (`radius-workspace`, new token), name 16/500, the radio drawn as the pen's (1.5 stroke; chosen, an accent ring 7 thick), body pt 4; the note at 14/1.45. |
| M8 | eTxpl | Rows under the workspaces | Fixed: no divider, 8 above, "Create workspace" without a chevron; Workspace settings keeps its chevron as drawn. |
| M9 | CvjGu | Chip labels text-2 | Fixed: text-3. The accent database chip still needs Koda to say what the database is for. |
| M10 | x7xv6 | Alert title without the number | Fixed: "Deploy #6 failed" once the overview is read ("Deploy failed" before). |
| M11 | V6bMR | Loading cards | Fixed: 194, border-strong, bars 12 and 10 r4 3 apart, the 46 state box holding 90 × 10 and 220 × 10, footer bar 10. |
| M12 | tDuEM | Couldn't-load spacing and copy | Fixed in part: top padding 40. The code line keeps "503 from Nomi's server · 09:14": the pen's "api.nomi.app" would be the app's internal backend address (`API_URL`), meaningless or misleading to the person. §9. |
| L1 | J8au5J | Filters row 44 | Fixed: 46 ([0,16,12,16]). |
| L2 | fbJBa | Card name spacing | Fixed: 3 apart, the line at 13/1.35 (18). |
| L3 | i7GQct | Sort check 20 | Fixed: 18 (`SheetOption`, used only by the sort sheet now). |
| L4 | v81Hi | Delete line height, icon | Fixed: 15/1.4 (21). The icon is already trash-2 (alias, as M4). |
| L5 | tDuEM | Couldn't-load sizes | Fixed (with P2's component): tile r14, title 17, gap 10, code 12/16, line 1.45 (20). |
| L6 | hkMeL | Sheet body top padding | Fixed: 4 on New project and Move. |

## How to reach each Projects fix (390 wide)
- Search, chips, no match: Projects → type in the search (Cancel, the clear, the counts); type
  something nothing matches.
- Sort: the Sort button. Workspaces: the workspace pill.
- A card's sheet: ⋯ on a card (Move, Delete from there). New project: New.
- The project's screen: tap a card's name (ask bar at the foot; a failed deploy shows its line and
  alert).
- Loading and couldn't load: stop the backend and reload (couldn't load), then Retry (loading).
