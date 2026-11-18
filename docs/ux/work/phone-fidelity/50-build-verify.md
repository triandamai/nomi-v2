---
work: phone-fidelity
gate: 3
round: 1
result: changes-required (partial: the chat flows are audited, Projects isn't)
date: 18 Nov 2026
---

# Gate 3: phone-fidelity (partial)

**Status: incomplete.** The pen.dev plan's MCP usage limit was reached partway through. After that,
`mcp__pencil__execute` refused every call, so I couldn't read the Mobile Projects frames
(fGjF3, T9IbH, G4Mwvd, owsMq, H4ngT2, F2Tuh, EkEA9) or take the light-twin pen screenshots. All
Projects screens and states were captured from the build (list below) but not compared with the
pen. The chat findings come from pen values read with `Get(…, {resolveInstances: true})` and dark
pen screenshots.

**How it was measured.**
- The build: production build of `origin/main` (8d665993) served on 4185 against the backend
  on 8080 (fake model). Playwright Chromium ran with `--font-render-hinting=none` at 390×844 with an iPhone UA, in dark and light.
- Theme: the person's saved preference resets the `nm_theme` cookie to `system`, so the theme
  came from `colorScheme`.
- Boxes and styles: each box comes from `getBoundingClientRect` and `getComputedStyle`, with
  colours mapped back to token names.
- Coordinates: the build has no drawn status bar or home indicator, so a build y equals pen y − 54.
- Files: scripts, screenshots and dumps are in
  `/private/tmp/claude-501/-Users-marioprasetyo-Documents-Nomi/7ff24e31-b37d-4e91-90b3-5ff5b1682437/scratchpad/fidelity/`
  (`build/<theme>/<screen>.png|.txt`).

## Summary

35 deviations in the chat flows: **13 high · 15 medium · 7 low**. Projects is not compared yet.

## Flows touched
| Flow | Mobile dark | Mobile light | Compared |
|---|---|---|---|
| Start & answer | FvlVs | r8Gz3 | steps 1, 2, 3 (toolbar), 6 |
| Manage chat (chat sheet, delete) | W3a8j | N2N1d | steps 1, 5 |
| Like & dislike | M0VaJC | mg56o | steps 1, 3 |
| Share & delete | yoVD6 | Pjc23 | step 1 |
| Message controls | ctP7Q | m2cy3I | steps 3, 4, 6 |
| Mobile Projects (7 flows) | fGjF3 T9IbH G4Mwvd owsMq H4ngT2 F2Tuh EkEA9 | twins | **not compared** (pen limit) |

## Deviations

### A chat: thread, nav bar, composer (Answer `yr0gh`, Mention picker `hUA50`)
1. **Composer has no attach (+) or mic.** Severity: high.
   - Frames: `yr0gh`/`Uuu5f` → `Composer` (`W44dP` Plus, `aPB6O` Mic).
   - Build: the pill spans the full row, 12→378 (366 × 38), with no buttons.
   - Pen: a 38 × 38 round `surface-2` Plus with a 20px `plus` in `text-2`, a gap of 8, then the pill (58→378, 320 wide) with a 20px `mic` in `text-2` at its right end.
   - Likely file: `src/lib/chat/ChatView.svelte` (doesn't pass `onattach`/`onrecord`) → `src/lib/ds/thread/PhoneComposer.svelte`. Attachments are UXP-001; §9 says nothing about leaving these out.
2. **A finished reply's reasoning row reads "Thinking".** Severity: high.
   - Frame: `yr0gh` → `k8JkX2` (Chat / Reasoning / Collapsed).
   - Build: "Thinking" plus a chevron on a completed reply (reply text present, `thoughtFor` unknown after reload).
   - Pen: "Thought for 8s".
   - Likely file: `src/lib/chat/ChatView.svelte:921`, which falls back to `m.reasoning_thinking()`.
3. **Date divider above the first message.** Severity: medium.
   - Frame: `yr0gh` → `RnQjl`.
   - Build: a "TODAY" mono divider with rules (358 × 14) at the top of a single-day thread.
   - Pen: no divider.
   - Likely file: `src/lib/chat/ChatView.svelte`.
4. **Nav bar: the "who" mascot is too small.** Severity: medium.
   - Frame: `yr0gh` → `BoURL/RzQVs`.
   - Build: the mascot is 16 × 16, so the Who row is 16 tall and the centre column 39.
   - Pen: Who Mascot 20 × 20, so the row is 20 and the column 43.
   - Likely file: `src/lib/ds/mobile/NavBar.svelte:21` (`size={16}`).
5. **Composer pill turns accent when focused.** Severity: medium.
   - Frame: `hUA50` → `vfYWh`.
   - Build: `focus-within:border-accent`.
   - Pen: while typing, with the keyboard up, the pill keeps `border-strong`.
   - Likely file: `src/lib/ds/thread/PhoneComposer.svelte`.
6. **User bubble width.** Severity: low.
   - Frame: `yr0gh` → `nltkF`.
   - Build: `max-w-70`, so the bubble is 280 and the text 252.
   - Pen: text fixed at 250, so the bubble is 278.
   - Likely file: `src/lib/ds/mobile/UserMessage.svelte`.

### New chat (`Uuu5f`, FvlVs step 2)
7. **Different starters.** Severity: high.
   - Frame: `kEp8c`.
   - Build:
     - "Summarize this week’s spending → dana"
     - "Plan my Saturday → rena"
     - "Start a small website project → koda"
     - "Remind me to stretch at 4pm → kala"
   - Pen:
     - "Build a menu page for my warung → koda"
     - "Turn a photo of my price list into an order form → maya"
     - "What can I build in 10 minutes? → nomi"
     - "Remind me to check pre-orders at 4pm → rena"
   - Likely file: `src/routes/(app)/chat/new/+page.svelte` (the starters).
8. **Starter label size.** Severity: medium.
   - Frame: `x0y5RK`.
   - Build: 13/17 (`text-ui`), every row 40 tall.
   - Pen: 15 `text`; long labels wrap to two lines (rows 58).
   - Likely file: `src/routes/(app)/chat/new/+page.svelte` (or its starter component).
9. Composer: the same as item 1.

### Chat sheet (`bVTSI`, W3a8j step 1) and the Chats long-press sheet
10. **Sheet subtitle drops the message count.** Severity: medium.
    - Frame: `G8cFz`.
    - Build: the nav bar's More sheet says "Nomi".
    - Pen: "14 messages · Dana".
    - Likely file: `src/lib/chat/ChatView.svelte` (the chat sheet).
11. **The Chats list's long-press sheet subtitle is the time.** Severity: medium.
    - Frame: `G8cFz`.
    - Build: "05:51".
    - Pen: the chat sheet's subtitle is "14 messages · Dana".
    - Likely file: `src/routes/(app)/chats/+page.svelte`.
12. **Export row has no chevron.** Severity: medium.
    - Frame: `GnNVF` (`i7GQct`).
    - Build: the label, then the "md" meta, then nothing.
    - Pen: a chevron-right (16, `text-3`) after the meta.
    - Likely file: `src/lib/chat/ChatView.svelte`.
13. **Delete chat icon.** Severity: low.
    - Frame: `PMfiF/rt7Zb`.
    - Build: `trash`, in both chat sheets.
    - Pen: `trash-2`.
    - Likely files: `ChatView.svelte`, `chats/+page.svelte`.

### Reply sheet (`SwByh`, M0VaJC step 1)
14. **Rows missing.** Severity: high.
    - Frames: `Wl0VM`, `YyT8Y`, `hUTDH`, `XPHtc`.
    - Build: only Regenerate and "Edit my question".
    - Pen, in addition:
      - "Try another model" (cpu, meta "claude-sonnet", chevron)
      - "Save to memory" (bookmark-plus)
      - a divider, then "Delete reply" (trash-2, `danger`)
    - §9 lists the built set but gives no reason for leaving these out.
    - Likely file: `src/lib/chat/ChatView.svelte` (the reply sheet).
15. **Quick tiles.** Severity: medium.
    - Frames: `MpUUC`, `P3hgut`.
    - Build: tiles 83.5 × 64, icon-to-label gap 4, label colour `text`, row bottom padding 8 (row 72).
    - Pen: tiles 84 × 68, gap 6, label `text-2`, row padding [0,16,10,16] (row 78).
    - Likely file: `src/lib/ds/overlays/SheetTiles.svelte`.
16. **Regenerate row shows a chevron.** Severity: low.
    - Frame: `GthtH`.
    - Build: chevron-right 16.
    - Pen: none.
    - Likely file: `ChatView.svelte`.

### Regenerate sheet (`lVDqs`, ctP7Q step 6)
17. **Subtitle copy.** Severity: high.
    - Frame: `j06yy/G8cFz`.
    - Build: "Nomi".
    - Pen: "Each result is kept as a version".
    - Likely file: `ChatView.svelte`.
18. **Another-model group missing.** Severity: high.
    - Frames: `IS9Ct`, `rTi6V`, `NCaNG`.
    - Build: ends after "As a list".
    - Pen: the mono label "WITH ANOTHER MODEL", then the rows "claude-sonnet · your key" and "deepseek-flash · Nomi credits" (cpu icon).
    - Likely file: `ChatView.svelte`.

### Dislike sheet (`D4O2g`, M0VaJC step 3)
19. **One reason too many.** Severity: high.
    - Frame: `ZxLos`.
    - Build: 7 chips, adding "Unsafe or harmful".
    - Pen: 6 chips: Not accurate, Wrong crew member, Didn't do what I asked, Too long, Out of date, Other.
    - Likely file: `src/lib/ds/thread/PhoneDislikeSheet.svelte`.
20. **Chip text.** Severity: medium.
    - Frame: `Jn7ay`.
    - Build: 14/18 in `text`.
    - Pen: 15, unselected `text-2`.
    - Likely file: `PhoneDislikeSheet.svelte`.
21. **Note field.** Severity: medium.
    - Frame: `IZhE5`.
    - Build: fill `surface`, radius 14, padding 12/14, 87 tall.
    - Pen: fill `bg`, radius 12, padding 12 all round, 84 tall.
    - Likely file: `PhoneDislikeSheet.svelte`.
22. **Spacing and label size.** Severity: low.
    - Frames: `TFfm6`, `RzZD0`.
    - Build: body gap 16; "Include the whole chat" at 14.
    - Pen: gap 14; label 15.
    - Likely file: `PhoneDislikeSheet.svelte`.

### Share sheet (`eMuOP`, yoVD6 step 1)
23. **App row.** Severity: high.
    - Frame: `F3QDj`.
    - Build: 4 tiles (no More), each 60 wide and spread out; icon 20; label 12.
    - Pen: 5 tiles: Copy link, WhatsApp, Telegram, Mail and More (ellipsis). Each is 65 wide with gap 8; icon 22; label 11.
    - Likely file: `src/lib/ds/thread/PhoneShareSheet.svelte`.
24. **Footnote copy.** Severity: high.
    - Frame: `nXEZB`.
    - Build: "Expires in 7 days".
    - Pen: "expires in 7 days · revoke in Settings › Data".
    - Likely file: `PhoneShareSheet.svelte`.
25. **Link helper copy.** Severity: high.
    - Frame: `lTymT`.
    - Build: "Read-only. No sign-in, no replies."
    - Pen: "Read-only, no sign-in".
    - Likely file: `PhoneShareSheet.svelte` / `messages/en.json`.
26. **Preview card.** Severity: medium.
    - Frame: `RkTF2`.
    - Build: fill `surface-2`, no stroke, radius 14, text in `text` at line height 18. Masking wasn't checked: the fake reply has no amounts.
    - Pen: fill `bg` with a 1px `border` stroke, radius 12, padding 12, text 14 at 1.45 in `text-2`, masked "Rp •••".
    - Likely file: `PhoneShareSheet.svelte`.
27. **Scope segment.** Severity: medium.
    - Frame: `QbRoJ`.
    - Build: 44 tall, padding 4, radius 12; the selected option is filled `surface`, 36 tall, weight 500.
    - Pen: 40 tall, padding 3, radius 10, gap 3; the selected option is filled `bg`, 34 tall, weight 600.
    - Likely file: `PhoneShareSheet.svelte`.
28. **Spacing and label size.** Severity: low.
    - Frame: `urNRO`.
    - Build: body gap 14; option labels 14.
    - Pen: gap 12; labels 15.
    - Likely file: `PhoneShareSheet.svelte`.

### Delete chat (`jk7EC`, W3a8j step 5)
29. **Message copy.** Severity: high.
    - Frame: `d4806i`.
    - Build: "All 3 messages and their cards are removed. Reminders, money records and emails the crew already created or sent stay where they are." It runs to 3 lines, so the head is 100 tall.
    - Pen: "14 messages and their cards are removed. Reminders, money records and sent emails stay." It runs to 2 lines; the head is 85.
    - Likely file: `messages/en.json` (`chat_delete_body`).
30. **Message line height.** Severity: low.
    - Frame: `d4806i`.
    - Build: 17.
    - Pen: 1.4 (18).
    - Likely file: `src/lib/ds/overlays/ActionSheet.svelte`.

### Your message (`nzT7f`, ctP7Q step 3) and editing (`lnxaH`, step 4)
31. **No lifted bubble.** Severity: high.
    - Frame: `l4j0Xz`.
    - Build: the bubble stays in the thread, dimmed under the scrim.
    - Pen: the bubble is lifted above the scrim, right-aligned just over the sheet, with a `border-strong` stroke and a shadow.
    - Likely file: `ChatView.svelte`.
32. **"Select text" row missing.** Severity: high.
    - Frame: `sKD62`.
    - Build: only Edit and resend and Copy text.
    - Pen: a third row, Select text, with the `text-select` icon.
    - Likely file: `ChatView.svelte`.
33. **Close shows a focus ring after a long press.** Severity: medium.
    - Frame: `rQ4w3`.
    - Build: a 2px `focus` outline on Close as the sheet opens from a pointer.
    - Pen: no ring.
    - Likely file: `src/lib/ds/overlays/BottomSheet.svelte` (initial focus).
34. **Editor buttons.** Severity: medium.
    - Frames: `t80BL2`, `uEd5M`.
    - Build:
      - Cancel and Send are 40 tall, padding 16, labels 14/500.
      - Cancel has no border and is `text-2`.
      - The Send arrow is 16.
    - Pen:
      - Both are 34 tall, padding 14, radius 17.
      - Cancel has a `border-strong` stroke, its label 15/400 in `text`.
      - Send's label is 15/500 and its arrow-up 14.
    - Likely file: `src/lib/ds/thread/PhoneMessageEditor.svelte`.
35. **Editor details.** Severity: low.
    - Frame: `bRx3K`.
    - Build: editor radius 18; the reply under it at opacity 0.4.
    - Pen: radius 16; the reply at opacity 0.35.
    - Likely file: `PhoneMessageEditor.svelte`.

### Chats list (`Ki4mr`, FvlVs step 1)
Matches the pen except for one item (listed here because it's less visible than the items above):
- The title, the search pill, the mono day labels, the rows, the tab bar and New chat all match:
  - Title: 32/600, letter-spacing −0.6.
  - Search pill: 358 × 38, radius 12.
  - Rows: 66 tall, with 16/500 titles and 14 previews.
  - Tab bar: 50 tall, padding [8,12,0,12], top stroke, icons 22, labels 10.
  - New chat: 56, 6px above the tab bar.

- **Row previews have no "Name:" prefix.** Severity: medium; not numbered above.
  - Frame: `r3rrB/oXF2h`.
  - Build: the reply text on its own.
  - Pen: "Tara: Share the itinerary with Sari?".
  - This has the same cause as the §9 avatar item (the list doesn't say who answered last), but §9 doesn't list the prefix. Add it to §9 or fix it.
  - Likely file: `src/routes/(app)/chats/+page.svelte`.

## Not compared, and why
- **All of Projects** (header, search, sort sheet, filters, cards, card sheet, New project sheet,
  Workspaces sheet, detail for failed, live and stopped projects, delete, move, couldn't load, loading).
  - The build side is captured in dark and light as `p01`–`p14`.
  - The pen.dev usage limit blocked reading the frames.
- **Stop and Deploy sheets:** not reachable locally.
  - The Stop button needs a project chat that's working at that moment.
  - Deploy needs Shipyard publishing, which this backend doesn't have.
- **Model sheet and crew sheet:** the pen has no frame for either.
  - The pen draws only the inline mention picker (`hUA50`) and the toolbar chips; the chips match the pen (28 tall, cpu 12, mono 11, `@` 13).
  - The build's "Crew" chip opens a bottom sheet ("Ask someone on the crew") that the pen doesn't draw.
- **Not captured:** Liked (`hoNKx`), Feedback sent (`g6QMdE`), Stop, Stopped, Branch and Copied (ctP7Q steps 1, 2, 5, 7), and the selected dislike chip.
- **Light theme:** build screenshots exist for every screen above, but the light pen twins
  weren't screenshotted before the limit. Colours were compared as tokens, which theme the same way.
- **Unread dot** on Chats rows: the seeded data had no unread chats.

## §9 items confirmed still accurate
- Chats:
  - The tab bar replaces the drawer.
  - Each row's avatar is the chat glyph.
  - There are no crew filters.
  - A long press opens a sheet with Pin to top and Delete chat.
- A chat's nav bar:
  - Back, the title, who answers, and More.
  - More's sheet has Rename, Pin to top, Move to a project, Export, Share whole chat and Delete chat.
- The thread:
  - Padded [16,16,20,16] and sitting at the bottom while it's short.
  - The bubble has radius 18.
  - Reply prose is 16 / 1.55 (25px).
  - The round action bar: 38px buttons, 18px icons, gap 2.
- New chat has Nomi's mascot, the question at 22 and the line.
- Export offers "md" only.
- The phone toolbar has the model and crew chips, with no thinking level.
- A reply's sheet:
  - Good, Bad, Copy and Share tiles over Regenerate (its own sheet) and Edit my question.
  - Bad opens the dislike sheet.
  - Share opens the phone share sheet.
  - Deleting a chat asks in an ActionSheet (Export first, Delete chat, Cancel).
- Your message: a long press opens its sheet (Edit and resend, Copy text), and editing is in place.
- Not re-checked: the 300-wide mention picker, and You → Settings.

## Result
changes-required for the chat flows (items 1–35). Re-run this gate for Projects once pen.dev access is back.
