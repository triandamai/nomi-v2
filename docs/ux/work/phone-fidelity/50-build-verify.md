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

## Projects

Added 18 Nov 2026, once pen.dev access was back.

**How it was measured:**
- The pen: the Mobile Projects frames, read with `Get(…, {resolveInstances: true})`.
- The build: the same production build of `8d665993` (a clean `git archive` copy built in the scratchpad, because this worktree's build had moved on with the chat fixes). It ran against the 8080 backend at 390 × 844 with an iPhone UA, dark and light.
- The account: "Siti's space" with 4 projects and a second workspace, "Warung Bu Siti". The 4 projects:
  - warung-kopi-menu: live, through a publish row inserted into the database
  - kos-booking: first deploy failed
  - arisan-tracker: stopped
  - laundry-orders: has files, never reported done, so it shows as Stopped
- Coordinates: as above, a build y equals pen y − 54.
- Light pen twins: not screenshotted. Colours were compared as tokens.

### Summary
29 deviations: **11 high · 12 medium · 6 low**.

### Deviations

#### Search, filters, sort (Find & filter `owsMq`)
P1. **Searching doesn't change the bar.** Severity: high.
- Frame: `epdb3` → `oFdFf`.
- Build, with "kos" typed:
  - The field stays 310 wide with no clear button.
  - Sort stays at its right.
  - The field shows an `accent` border and the focus shadow.
- Pen:
  - The field narrows to 299 and gets a 20px round clear button (`text-3` fill, `x` 12 in `surface-2`).
  - A text "Cancel" (16, `text`) replaces Sort.
  - The field's stroke is `$nm-focus`, 1px.
- Likely file: `src/lib/ds/projects/PhoneProjectSearch.svelte`.

P2. **The no-match state is the desktop's, not the phone's.** Severity: high.
- Frame: `Ifa4i` → `OC2PS`.
- Build:
  - An outlined box (`border`, radius 10, padding 24).
  - A bare 20px `search-x` in `text-3`; title 13/500; text 13 at line height 20.
  - "Clear search": 44 tall, radius 6, 13/400, no icon.
- Pen:
  - No box; padding [40,24].
  - A 48 × 48 `surface-2` tile (radius 14) holding a 22px `search-x` in `text-2`.
  - Title "No projects match “kasir”" at 17/600; text 14 at 1.45.
  - "Clear search": a 40-tall pill (radius 20, `border-strong`, 14/500) with an `x` icon.
- Likely file: `src/routes/(app)/projects/+page.svelte` (`phoneEmpty`), `src/lib/ds/projects/ProjectEmpty.svelte`.

P3. **Sort sheet: order and labels.** Severity: high.
- Frame: `TIuwF` → `M7SCRS`.
- Build: Updated, Name, Status, Created.
- Pen: Updated, "Status · needs you first", "Name · A–Z", Created.
- The selected check is 20 in the build, 18 in the pen (see L3).
- Likely file: `src/routes/(app)/projects/+page.svelte` (the sort sheet).

M1. **Filter counts don't follow the search.** Severity: medium.
- Frame: `epdb3` → `wDi32`.
- Build: with one card shown, the chips still say All 4, Live 1.
- Pen: the counts are the search's ("All 1", "Live 0").
- Likely file: `src/routes/(app)/projects/+page.svelte` (`filterOptions`).

L1. **Filters row height.** Severity: low.
- Frame: `J8au5J`.
- Build: 44 tall (bottom padding 10).
- Pen: 46 (padding [0,16,12,16]).
- Likely file: `src/lib/ds/projects/PhoneFilters.svelte`.

#### Header and cards (`etEGD`, `zz1kw`, `a7KP1`)
M2. **The workspace pill's initial is in mono.** Severity: medium.
- Frame: `LXujm/AXKpW`.
- Build: Geist Mono 12/400.
- Pen: sans 12/600.
- Likely file: `src/lib/ds/projects/PhoneProjectsHeader.svelte`.

L2. **Card name and description spacing.** Severity: low.
- Frame: `fbJBa`.
- Build: gap 2; description at line height 17.
- Pen: gap 3; line height 1.35 (18).
- Likely file: `src/lib/ds/projects/ProjectCard.svelte` (phone).

The rest of the card matches the pen:
- The card itself: 358 wide, padding 16, radius 14, `border-strong`.
- Tile: 40, radius 10, Fraunces 17.
- State block: radius 10, padding [10,12]; `danger-bg` when failed.
- Footer: a top stroke, padding top 12.
- Actions: 40-tall pills, with the right icons (play, wrench, arrow-right).
- Live line: the arrow-up-right 12 icon.

#### Card sheet (Card menu `T9IbH`, `M4p0u`)
M3. **Subtitle starts with the chat count, not the state.** Severity: medium.
- Frame: `OFwQo/G8cFz`.
- Build: "1 chat · deploy #1 · 22h ago" (the card's footer).
- Pen: "live · deploy #15 · 2d ago".
- Likely file: `src/routes/(app)/projects/+page.svelte` (`subtitle={card.footer}`).

M4. **Icons differ.** Severity: medium.
- Frame: `R4a7xS`.

| Row | Build | Pen |
|---|---|---|
| Open workspace | `arrow-right` | `app-window` |
| Visit live site | `globe` | `arrow-up-right` |
| Take down | `eye-off` | `power` |
| Delete project | `trash` | `trash-2` |

- Likely file: `src/lib/projects` / `projectMenu` (the phone sheet's icons).

M5. **Separators.** Severity: medium.
- Frames: `XDW6C`, `ZRj78`.
- Build: 1px lines across the full width.
- Pen: a 9-tall separator (padding [4,20]) with a 350-wide inset line, so the sheet is 16px taller.
- Likely file: `src/routes/(app)/projects/+page.svelte` / `SheetRow` divider.

M6. **No chevrons.** Severity: medium.
- Frames: `VvKNR`, `gvrNK`, `nEjEw`.
- Build: Move to workspace…, Share… and Settings have none.
- Pen: each of the three has chevron-right 16 in `text-3`.
- Likely file: `src/routes/(app)/projects/+page.svelte`.

#### Move (`WbD1H`) and Delete (`Z6OsC`)
P4. **Move title repeats the subtitle and wraps.** Severity: high.
- Frame: `rbozn`.
- Build: title "Move laundry-orders to another workspace" (2 lines, 44 tall) over the subtitle "To another workspace".
- Pen: title "Move salah-tracker", subtitle "To another workspace".
- Likely file: `src/lib/ds/projects/ProjectMoveSheet.svelte` (or its message).

P5. **The current workspace's meta repeats "current".** Severity: high.
- Frame: `KEIJW`.
- Build: "current · current · only you".
- Pen: "current · 3 members".
- Likely file: `ProjectMoveSheet.svelte`.

M7. **Workspace cards.** Severity: medium.
- Frame: `I5UHw`.
- Build: 64 tall, padding 12/14, tile 32 with radius 10, name 16/400, radio border 2.
- Pen: 68 tall, padding 14, tile 36 with radius 9, name 16/500, radio stroke 1.5; body padding top 4.
- Not checked: the pen's note under the cards (who sees it, what stays). It appears only once a workspace is chosen, and I didn't choose one.
- Likely file: `ProjectMoveSheet.svelte`.

P6. **Delete sheet copy.** Severity: high.
- Frame: `AHDzi`.
- Build: "Its files, its live app and its data go for good. Its chat stays."
- Pen: "The live URL stops working and the database is removed. Your code stays in your account f…" (cut off in the dump).
- Likely file: `src/lib/ds/projects/DeleteProjectSheet.svelte` / `messages/en.json`.

L4. **Delete details.** Severity: low.
- Frame: `v81Hi`.
- Build: body line height 19; the button's icon is `trash`.
- Pen: line height 1.4 (21); the icon is `trash-2`.
- Likely file: `DeleteProjectSheet.svelte`.

#### Workspaces sheet (Workspaces & people `F2Tuh`, `V3fmpB`)
P7. **Workspaces are full-width rows, not inset cards.** Severity: high.
- Frame: `a9NrP`.
- Build:
  - Rows run edge to edge: 60 tall, padding [8,20].
  - The tile is round, 36, filled `surface-2`; name 16/400.
  - The chosen workspace has a check but no fill.
- Pen:
  - Cards 64 tall, padding [12,14], radius 14, inside body padding [4,16] with gap 8.
  - The tile is 36 with radius 9, filled `bg`; name 16/500.
  - The chosen card is filled `surface-2` and has the check.
- Likely file: `src/routes/(app)/projects/+page.svelte` (the Workspaces sheet).

P8. **The boundary note is missing.** Severity: high.
- Frame: `C8evkZ`.
- Build: nothing.
- Pen: a `bg` box (padding 14, radius 12) under the workspaces, with two parts:
  - "Shared in a workspace / Projects, their chats and shared crew templates" (users icon)
  - "Always personal / Your chats, memory, money and inbox" (lock icon)
- The desktop's §9 note says to drop "shared crew templates"; the box itself should still be there.
- Likely file: `src/routes/(app)/projects/+page.svelte`.

M8. **Rows under the workspaces.** Severity: medium.
- Frame: `eTxpl`.
- Build:
  - A 1px divider above the rows.
  - "Create a workspace", with a chevron.
- Pen:
  - No divider; padding top 8.
  - "Create workspace", with no chevron.
- Likely file: `src/routes/(app)/projects/+page.svelte`.

#### New project (`H4ngT2`, `xNV1T`)
M9. **Chip labels.** Severity: medium.
- Frame: `CvjGu`.
- Build: `text-2`.
- Pen: `text-3`; the accent `database · for orders` chip appears once a database is detected.
- The URL chip ("*.trian.space") doesn't show here: the local backend gives no base domain, so I didn't count it.
- Likely file: `src/lib/ds/projects/ProjectComposer.svelte` (phone).

L6. **Sheet body padding.** Severity: low.
- Frame: `hkMeL`.
- Build: top padding 0 (here and in Move).
- Pen: [4,16,0,16].
- Likely files: `ProjectComposer.svelte`, `ProjectMoveSheet.svelte`.

#### Project detail (Detail panel `G4Mwvd`, `V9w2zU`)
P9. **Ask bar at the foot.** Severity: high.
- Frame: `kB2Dt`.
- Build: the chat's `PhoneComposer`, a 38-tall pill reading "Ask Koda to change something", with no mascot or mic.
- Pen: the Mobile / Ask Bar:
  - The bar: 52 tall, radius 26, `surface`, `border-strong`, shadow.
  - Inside it: Koda's 20px mascot, the placeholder at 15, and a 40px round `surface-2` mic.
  - Its wrap: padding [10,16,4,16] with a top stroke.
- Likely file: `src/routes/(app)/projects/+page.svelte` (`phoneDetail`'s `ask` snippet).

P10. **Status line under the name.** Severity: high.
- Frame: `AL711`.
- Build: the card's footer, "1 chat · deploy #1 · 23h ago".
- Pen: the state, "deploy #6 failed · #5 still live".
- Likely file: `src/routes/(app)/projects/+page.svelte` (`liveLine(...) ?? card.footer`).

P11. **The live project's status line says "deploy #0".** Severity: high.
- Build: "live · deploy #0 · 23h ago", while its card says "deploy #1".
- The publish was seeded straight into the database (no Shipyard), so this may come from the deploy overview being empty. Even so, the line shouldn't show #0.
- Likely file: `src/routes/(app)/projects/+page.svelte` (`liveLine`).

M10. **Alert title has no deploy number.** Severity: medium.
- Frame: `x7xv6`.
- Build: "Deploy failed".
- Pen: "Deploy #6 failed".
- Likely file: `src/routes/(app)/projects/+page.svelte`.

The rest of the detail matches the pen:
- Back bar: 48 tall, "Projects".
- Hero: tile 56, radius 14; name 20/600.
- Alert: `danger-bg`, padding 14, radius 12, with "Ask Koda to fix".
- Open workspace and Visit: 46 tall, radius 12.
- The parts rows: 52 tall, 18px icons, 15 labels, values in mono 12.
- The chats list, and no tab bar.

#### Loading and couldn't load (Loading, offline & errors `EkEA9`)
M11. **Loading cards.** Severity: medium.
- Frame: `hc5uw` → `V6bMR`.
- Build:
  - Cards 204 tall with a `border` stroke.
  - Name bars 10 and 8 tall, radius 3, gap 6.
  - An empty 56-tall state block; a footer bar 8 tall.
- Pen:
  - Cards 194 tall with a `border-strong` stroke.
  - Bars 12 and 10 tall, radius 4, gap 3.
  - The state block (46 tall) holds two bars (90 × 10, 220 × 10); the footer bar is 10.
- Likely file: `src/lib/ds/projects/ProjectCardLoading.svelte` (phone).

M12. **Couldn't-load spacing and copy.** Severity: medium.
- Frame: `ZSUL6` → `tDuEM`.
- Build: top padding 48; the code line says "503 from Nomi’s server · 11:49".
- Pen: top padding 40; the code line says "503 from api.nomi.app · 09:14".
- Likely file: `src/lib/ds/projects/LoadError.svelte` / `projects_load_failed_reason`.

L5. **Couldn't-load sizes.** Severity: low.
- Frame: `tDuEM`.
- Build: icon tile radius 12; title 16; gap 8; code 11/14; text line height 18.
- Pen: radius 14; title 17; gap 10; code 12/16; line height 1.45 (20).
- Likely file: `LoadError.svelte`.

L3. **Sort sheet's check.** Severity: low.
- Frame: `i7GQct`.
- Build: 20.
- Pen: 18.
- Likely file: the sort sheet in `src/routes/(app)/projects/+page.svelte`.

### Projects not compared, and why
- **Stop sheet** (`w2yWm`) and **Deploy sheet** (`xvEu2`): not reachable locally.
  - Stop needs a project chat that's working at that moment.
  - The Deploy action needs Shipyard publishing.
- **Not captured:**
  - Card menu: Rename, Renamed, Duplicate (`O79XLV`, `X6aX7`, `w5PNXl`).
  - Find & filter: Empty filter (`L2NzD`).
  - New project: Questions, Building, Live (`P8ljBK`, `m7Tzqf`, `gY5Yt`).
  - Workspaces & people: Invite and Invited (`PVYvG`, `rPrc3`).
  - Detail panel: Ask, Changing, Export (`EUo3M`, `kZIuK`, `JznvP`).
- **Offline, App down, Koda checks** (`vKZi8`, `epdkW`, `iKLNC`): §9 says these aren't built yet.

### §9 items confirmed still accurate (Projects)
- `PhoneProjectsHeader` (workspace pill, member count, Invite, large title, New).
- `PhoneProjectSearch` with Sort.
- `PhoneFilters`, with Needs you first while something needs the person.
- Phone cards with the ⋯ sheet carrying the desktop menu's items.
- New opens the New project sheet with the composer, and no Import from GitHub.
- Tapping a card opens `PhoneProjectDetail`: no tab bar, no "Next prayer" picture.
- Delete wants the name typed; Move picks among workspace cards.
- Loading and couldn't-load keep the search and filters, with counts left out.
- The Sort sheet has no "Saved for …" subtitle, and the Workspaces sheet has no Join.

### Result (Projects)
changes-required: P1–P11, M1–M12, L1–L6.
