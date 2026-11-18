---
work: audit-chat-files
gate: 3 (build verify)
verified_by: ux-lead
verified_on: 18 Nov 2026
build: feat/frontend-new-v2-chat-files (PR #305) at b87a04f5, `npm run build` + `node server.js` on :4190, backend on :8080 (fake model)
contract: 40-ux-contract.md (verified 16 Nov 2026)
verdict: CHANGES REQUIRED (0 high · 4 medium · 4 low)
---

# Build verify: audit-chat-files

## How it was checked

- **Viewports.** Desktop 1512×862, and phone 390×844 with an iPhone user agent and touch. Each in dark and light (`colorScheme`), Chromium with `--font-render-hinting=none`.
- **Accounts and seeding.** Made the way `e2e/backend/support/accounts.ts` makes them. The scenarios were reached and seeded the way `e2e/backend/chat-files.e2e.ts` reaches them:
  - a route abort for a dropped upload;
  - an `MZ` file for the 415;
  - hidden vision models for "no capable model";
  - an inserted `admin_llm_models` row with `media_inputs = {image}` for Switch model and the photo.
- **Viewer states.** Loading was held by delaying `/attachments/:id/content` for 2.5 s. The error was forced by a 404 on `/attachments/:id`.
- **What was measured.** `getBoundingClientRect` and computed styles were checked against `Get(…, {resolveVariables, resolveInstances})` of:
  - the D8, D9, D11, Errors D5, D10 and Errors D6 composers;
  - M9, M4 and M13;
  - the sheets BhXcx, DgnEc and s10RHA;
  - the menus PZvaP and er95B;
  - the viewers HVM9y, K1phVf and rCBA7;
  - the D14 bubble JVUHc;
  - the Link D5 page RCgbo.
- **Where the files are.** Screenshots and the measurement JSON are in `/private/tmp/claude-501/-Users-marioprasetyo-Documents-Nomi/7ff24e31-b37d-4e91-90b3-5ff5b1682437/scratchpad/gate3/shots/cf/`.

## What matches (no action)

- **G3 failed chip (BD98f), desktop.** The chip matches the pen exactly:
  - background `#F07B72` at 6%, border at 20%, r6, padding 6/8/6/10, gap 10;
  - 16 icon, 12 mono name, 11 mono reason in danger;
  - actions gap 12, padding 0/2/0/6, `Retry` 12/500 and `Remove` 12/400.

  The hint copy is verbatim. Send has `aria-disabled` and `aria-describedby`, and `↵` leaves the text in the field.
- **G3 retry (M1 restart copy).** The chip reads `Uploading again · 0% · 0 of 380 B`. It has the 3 px track and 4 px gaps of D9, and a Cancel button.
- **Removed line.** On desktop it reads `bca-oct-copy.pdf removed · 2 files ready`, 11 mono `#64646C`, 6 under the chips (yhuBX). Send turns on.
- **G4 refused (415).**
  - The chip reads `.exe files can't be read`, the hint reads `Remove kasir-setup.exe to send.`, and Backspace on the chip removes it.
  - In a mixed pick, the other files carry on.
  - The missing pre-upload check and helper line are stated in AGENTS.md §9 and BACKEND_TODO ("The accepted types").
- **G6 warning (rrjYM / kthOB).**
  - Chip: warning background at 8%, 28 thumb r4. The 12 icon (pen 11) is stated.
  - Fix line: 12 icon, label gap 8. `Switch model` has padding 2/6, r6, 12/500. `Send text only` is 12 `#A1A1A9`.
  - On desktop, focus lands on `Switch model` with the 2 px `#7BE3A4`/40% ring (XHDLj).
- **Menus.**
  - Switch model (PZvaP): 320 wide, padding 4, group 22, rows 40, divider, hidden row 32, Models & providers 32 at 13 `#A1A1A9`. Focus is on the first model.
  - No match (er95B): 392×142 exactly, title 13/500, 12 icon plus 11 mono hidden line, Add row 36, footer 46 with a 30-high `Send text only`. Focus is on `Add one in Models & providers`.
  - `Esc` returns focus to `Switch model`.
  - The draft survives the trip to Settings and back (e2e).
- **G6 Send text only.** No attachment tag is sent. The note reads `Sent without struk-belanja-nov.png · E2E default (fake) can't see images`, 11 mono `#64646C` with a 12 icon. It sits under the bubble, which is stated in §9.
- **Sent files (ENNIK).** Cards are 200×47, padding 8/10, 28 tile r4, 12 mono name, 11 mono meta, gap 8, right-aligned. `PDF · 380 B` instead of pages is stated.
- **Viewer (HVM9y / K1phVf / G534E, desktop).**
  - Panel 440, header 52 (14 icon, name 12 mono, meta 11 mono, gap 2), step bar 42 with `Opening page 1 · 0:01`.
  - Page skeleton: 20 padding, bars 10/8 high, gap 12. Footer reads `The answer stays usable while this opens.`
  - Error: 18 danger icon, title 13/500, body 12, `preview_failed · 404`, `Try again` (30 high, bordered) and `Download`.
  - Focus goes to Close; `Esc` gives it back to the card.
- **Photo (rCBA7).** Header with Zoom out · `100%` · Zoom in · Download · Close. The footer reads `1 of 1 photo in this message` (11 mono `#A1A1A9`) and `esc closes`.
- **Shared link (RCgbo, Link D5).**
  - Locked chips read `PDF · private`, and the meta reads `… · files private`.
  - The note reads `E. kept the files in this reply private. You see their names only.` (12, `#64646C`, 12 lock icon, gap 6).
  - No `/attachments/` address is in the page.
- **Phone sheet (BhXcx, M10).**
  - Header: 15/600 mono name, 11 mono danger reason.
  - Rows 52 (icon 20, label 16), `starts the upload again`, `stays on your phone`, Remove in danger.
- **Phone sheets M14 and M15 (DgnEc, s10RHA).**
  - Header 17/600 and 13 sub.
  - Model rows: 15 mono name, 11 mono meta. Hidden row padding 12/20/4/20.
  - Add row: 16 label, 16 chevron. `Send text only` 358×46 r12, 16/500.
- **Light theme.** All screens were captured. The tokens flip correctly: warning tint, danger tint and accent ring.
- **Left out because the backend lacks them.** These are each in BACKEND_TODO "Chat files" and AGENTS.md §9, so they aren't failures:
  - G3 reason codes and resume;
  - the G4 type list and helper;
  - G7 estimate;
  - G9 read results;
  - G11 Maya's files;
  - G12 citations and highlights;
  - G16 files row and files open;
  - M3 name masking.

## Deviations

| # | Frame | What the build does (measured) | Contract / pen | Severity |
|---|---|---|---|---|
| 1 | M9 rrKfg, M12 rEdL2, M13 sE5RV, M4 S61SO (chips) | On a phone, every composer chip is **56 high**. Its Remove or `⋯` button is drawn 44×44 inside the chip (`pointer-coarse:size-11`). So the ready chip is 213×56, the failed chip 303×56, the rejected chip 256×56 and the warning chip 357×56. | Chips are 43 high (ready 184×43, failed 283×43, warning 280×43). The More button is 24×24 and Remove 14. AGENTS.md wants 44 pt targets, but the target should be padded outside the drawing (negative margin or `::after`), not grow the chip. | medium |
| 2 | M13 sE5RV (Notice › Actions) | `Switch model` and `Send text only` are pills **44 high** (119×44 and 124×44). | Pills are 32 high, padding 0/16, r20. Same fix as #1: pad the hit area. | medium |
| 3 | M4 S61SO (Notice) | The removed line on a phone starts at x 12: its 12 icon is at 12 and the text at 30. | Notice padding is [0,4], so the icon sits at 16 and the text at 34. That's 4 px off (the failed-upload Notice is correctly at 16 and 34). | medium |
| 4 | M14 DgnEc, M15 s10RHA (accessibility) | On a phone, opening Switch model focuses the sheet's **Close** (checked for both the list and the empty case). | Contract › Accessibility: "The model menu opens → the first model. With no capable model, `Add one in Models & providers`." Desktop does this; the phone sheet doesn't. | medium |
| 5 | M15 s10RHA | `Send text only` in the empty sheet has a 16 send icon after the label (x 245). | The sheet's `Send text only` has its icon disabled (NeZlH); label only. | low |
| 6 | K1phVf (desktop error) | `Try again` draws its icon **before** the label (icon x 797, label x 817). | Label first, then the 14 icon (HxM4q at 12, EhwSk at 73), as `Send text only` does. | low |
| 7 | Accessibility (chip removed) | Removing the last chip moves focus to the **previous** chip (`[at] ?? [at-1]`). | "The next chip, otherwise the field." Either is reasonable. If the previous chip is preferred, amend the contract; don't leave it implicit. | low |
| 8 | D14 JVUHc (note under bubble) | The `Sent without…` note sits 39 px under the bubble (bubble ends at y 172, note at 211), past the hidden hover actions' row. So it reads closer to Nomi's reply (49 px below) than to its own message. | The placement under the bubble is a stated deviation. Keep the note within 6–8 px of the bubble (the pen's gap is 6) so it stays attached to the message it describes. | low |

Not counted:
- the chat composer's own padding (13/11/9 inside a 1 px border, gap 14), which is the pre-existing chat composer, not D8's Home composer;
- the shared page's user bubble text colour (pre-existing).

**Verdict: changes required.** #1–#4 are medium. Items #1 and #2 have one cause: 44 pt targets drawn as visible size instead of hit area. Fix that once, in `ComposerFile` and `ModelFix`. The twin `AttachmentChip`'s coarse-pointer button needs the same treatment from the non-twin wrapper (for example an `::after` hit area) without changing its props. After the fix, re-check M9, M12 and M13 at 390 only.

## Answers to the engineer's questions

1. **Focus order: keep the visual order (as built).**
   - Tab order must follow what's on screen (WCAG 2.4.3), and attach is drawn in the toolbar under the field. Don't reorder with `tabindex`.
   - The contract's Accessibility order is amended to: chips → field → fix line actions → attach → model → Send.
   - The contract's intent still holds: each chip is one stop, and the fix line comes before Send.
2. **Hint priority: yes, a failed upload wins.**
   - Retry or Remove is the step that unblocks Send, and the fix line can't help until the file is there.
   - Two conditions:
     - **The picture's chip keeps its warning meta** (`… can't see images`) while the failed hint shows, so nothing is hidden.
     - **Once the failure clears, the fix line appears and focus moves to `Switch model`**, as in the plain G6 case.
   - If both chips are removed or fixed in one go, follow the same rule: the next blocking reason replaces the hint.
