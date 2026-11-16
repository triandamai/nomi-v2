---
work: audit-chat-files
proposal: UXP-001
author: pen-designer
date: 16 Nov 2026
---

# Pen changes — UXP-001

## Part A

Gaps G3, G4, G6, G7. All frames built in `Nomi.pen`, desktop dark/light and mobile dark/light,
using only existing variables. All root flows have `placeholder` unset. A layout-problem scan of every
new node found 0 problems, and there are 0 overlaps between top-level frames in the chat section.

### Canvas layout change (read before Part B)
- Desktop `Steps` rows can hold at most 8 steps before they run into the phone label column at x=14886.
  Iyq9S / ORByl already had 7 steps, so the new steps sit in a **second row** inside each flow:
  `Steps · UXP-001 files`: **vnsf3** (dark, Iyq9S) and **P5FFEI** (light, ORByl). Numbering continues at 8.
  This row is now full (8 steps, 12672 px wide).
- To make room, the 40 top-level nodes in the chat rows below Attachments (Manage chat → Shared link,
  all four columns plus their labels; y 27919–41697, x ≥ 0) were **moved down by 953 px**. Nothing was
  restyled or deleted. Shared link now starts at y=42650. The Build group still starts at y=46890,
  which leaves about 3000 px of space.
- Part B: start a third row in Iyq9S / ORByl (e.g. `Steps · UXP-001 answers`), then shift the same
  chat rows down by that row's height again.
- Mobile flows grow sideways (plenty of room to the right). Errors & recovery steps were added to the
  existing scenario rows. Their heights did not change.

### New components
Placed in `Design System / Chat responses` → `Board / Attachments` (xzB2R), under a new label
`CHIP STATES · FAILED · REJECTED · MODEL WARNING` (m8qPR), row `Chip States` (Lh3yO).

| Component | Id | Built from |
|---|---|---|
| `Chat / Attachment / Failed` | **BD98f** | o8o2x / iAYlM chip shape + the danger chip from TpuRc "File too large" |
| `Chat / Attachment / Rejected` | **xpYoV** | same family |
| `Chat / Attachment / Model Warning` | **rrjYM** | iAYlM shape, `$nm-warning-bg` + `$nm-border-strong` (same pairing as existing chips) |

Override keys: Failed `s33iK` name · `v47rw` reason · `Z1n6b` actions (Retry `Hv0dc`, Remove `R963D`).
Rejected `gt5xs` name · `p1cG32` reason · `ziwk0` actions. Model Warning `ODbju` name · `F76Rd` meta · `eBjXH` thumb.

---

### G3 · Upload failed
| Surface | Frames |
|---|---|
| Desktop dark (Iyq9S) | Step 8 **Q7n2MM** "Upload failed" · Step 9 **N3Iyy** "Retry picks up where it stopped" |
| Desktop light (ORByl) | Step 8 **fhdWO** · Step 9 **vTWuT** |
| Mobile dark (FLwdb) | Step 9 **rrKfg** "Upload failed" · Step 10 **BhXcx** "Retry or remove" (long-press sheet) · Step 11 **bITK4** "Retrying" |
| Mobile light (Fpgnv) | Step 9 **QAikj** · Step 10 **APtff** · Step 11 **QSA24** |
| Errors desktop dark (TpuRc, scenario 6 "Problems before or as you send") | Step 5 **yhuBX** "Upload failed, then removed" |
| Errors desktop light (Nud6J, scenario 6) | Step 5 **F5kA4** |
| Errors mobile dark (AyrHG, scenario 6 "Send failures and Nomi credits") | Step 4 **S61SO** "Upload failed, then removed" |
| Errors mobile light (uuCam, scenario 6) | Step 4 **g2x0P** |

States covered:
- **Failed:** a ready chip next to a failed chip, Send disabled, with the hint in the toolbar.
- **Retrying:** the chip goes back to progress.
- **Removed:** the failed chip is gone and Send is enabled. This state is in the Errors & recovery steps.

Copy used: chip `bca-oct-2026.pdf` · `Upload failed · connection lost` · `Retry` `Remove`.
Send hint `1 file didn't upload. Retry or remove it to send.` (desktop: replaces the `↵ send` hint next to the disabled Send; mobile: the composer Notice line).

Copy I added (not in the proposal):
- Retrying: `Resuming · 41% · 2,1 of 5 MB`
- After removing: `bca-oct-2026.pdf removed · 1 file ready`
- Mobile sheet: header `bca-oct-2026.pdf` / `Upload failed · connection lost`; rows `Retry` (meta `picks up at 41%`) and `Remove` (meta `stays on your phone`)

Mobile: the failed chip is too narrow for both buttons. It shows a `⋯` button instead (the `Actions` node replaced in the instance), and long-press or tap opens a sheet built from `Mobile / Sheet Header` s9xrCL and `Mobile / Sheet Row` Rxo00. Chips stack vertically on mobile so the failed chip stays visible.

### G4 · File type not supported
| Surface | Frames |
|---|---|
| Desktop dark | Step 10 **IvQAw** "File type not supported" (`kasir-setup.dmg`) |
| Desktop light | Step 10 **IpiJU** |
| Mobile dark | Step 12 **rEdL2** |
| Mobile light | Step 12 **WtYon** |
| Errors desktop dark / light | Step 6 **ymrwy** / **EF25o** "A file type Maya can't read". This shows a pick of 3 files: ready, uploading, and `catatan-kas.pages` rejected. |
| Errors mobile dark / light | Step 5 **U6S0r** / **WUqGg** (same mixed pick) |

Copy: `.dmg files can't be read` · `Maya reads PDF, images, sheets and zip.` · `Remove`.
Errors variant: `.pages files can't be read`. No `.heic` anywhere (D2).

Copy I added: `Remove kasir-setup.dmg to send.` / `Remove catatan-kas.pages to send.` (the Send hint).

### G6 · Model can't read this file
| Surface | Frames |
|---|---|
| Desktop dark | Step 11 **deDgy** "Model can't read this file" · Step 12 **mDEso** "Switch to a model that can" (menu open, frame PZvaP) · Step 13 **JVUHc** "Send text only says so" |
| Desktop light | Step 11 **XUc6r** · Step 12 **CSvJi** · Step 13 **xA6ae** |
| Mobile dark | Step 13 **sE5RV** · Step 14 **DgnEc** (model sheet) · Step 15 **WDeji** (sent bubble) |
| Mobile light | Step 13 **a7GWtv** · Step 14 **W3w6z** · Step 15 **L3Vmo** |

Copy: chip `struk-belanja-nov.jpg` · `DeepSeek V3 can't see images`; actions `Switch model` · `Send text only`.

Copy I added:
- Inline fix line: `Switch to a model that can see images, or send your message without the photo.` (mobile drops "your message")
- Menu, filtered: `YOUR KEYS` → `claude-sonnet` `Anthropic · reads images`; `NOMI CREDITS` → `gemini-flash` `Nomi credits · reads images`; `deepseek-v3 hidden · can't see images`; `Models & providers`. Mobile sheet: title `Switch model`, subtitle `Models that can see images`.
- Sent bubble (the result is shown inside the bubble): `Sent without struk-belanja-nov.jpg · DeepSeek V3 can't see images`
- Nomi's reply (byline `11:20 · deepseek-v3`): "I didn't get the photo, so I can't read the receipt. Type the items and prices here and I'll add them up and mark the ones for the house, or switch to a model that can see images and send the photo again."

Reuse:
- The desktop model chip is the existing toolbar `Chip / Model` pattern, relabelled `deepseek-v3`.
- The menu copies the structure and tokens of TpuRc step 2's model menu (QnM7s). It uses a `cpu` icon for both groups, not that menu's `sparkles` icon.
- The mobile sheet uses s9xrCL / Rxo00, with model ids in mono.

### G7 · Cost and provider before a big read
| Surface | Frames |
|---|---|
| Desktop dark | Step 14 **Z3efXY** "Cost before a big read" (Nomi credits) · Step 15 **HAczP** "On your own key" |
| Desktop light | Step 14 **u34vwy** · Step 15 **GAeyu** |
| Mobile dark | Step 16 **z274p9** · Step 17 **ch6hu** |
| Mobile light | Step 16 **EO53V** · Step 17 **kin2h** |

Copy: `laporan-keuangan-2026.pdf · 184 pages · about 96.000 tokens` · `Runs on Nomi credits · ≈ Rp 1.900` · BYOK `Runs on your Anthropic key`. Chip meta `PDF · 184 pages · 14,6 MB`. All values are mono.

Meter i5A8H (credits variant only): label `Nomi credits`, value `82% left` (matches TpuRc's "82% left this month"), caption `≈ 80% left after this read`. The meter values are mine and are examples only.

The estimate is a quiet row in the composer above the toolbar and Send, with a top border and no box. The BYOK variant has no meter, because Nomi doesn't bill that read.

---

### Open issues / for ux-lead
1. **G4 blocks Send.** The rejected chip keeps Send disabled until it's removed. This follows TpuRc "File too large" ("Send stays disabled until the file is removed"). The proposal doesn't say, so please confirm.
2. **G6 Send state.** While the model warning shows, Send is drawn disabled and the two actions are the only ways forward. If "warning, not error" should mean Send stays live (and sends text only), steps 11 and 13 need a small update.
3. **Rejected sub-copy placement.** `Maya reads PDF, images, sheets and zip.` is a helper line under the chips, as in the existing too-large pattern. It is not inside the `Rejected` component.
4. **Model naming.** I used `deepseek-v3` (mono id) in the chips and menus so it matches the proposal's "DeepSeek V3". The rest of the file uses `deepseek-flash`.
5. **Scenario headers not updated.** The Errors & recovery scenario 6 descriptions (TpuRc/Nud6J/AyrHG/uuCam) still describe only their original steps. I didn't edit the existing copy.
6. **Verification limits.**
   - Screenshots render small. Nested-node screenshots show on a white background, which is a pen artefact.
   - Structure was checked with bounds and a layout-problem scan (0 problems).
   - Step 12's menu (PZvaP) is an absolute popover that extends past the composer on purpose.
   - Warnings about `fill_container` outside a layout come from hidden slots in `Mobile / Composer` (oFN7G) and from a disabled message. They don't affect anything visible.
7. Flow registry (`docs/ux/flow-registry.md`) not updated, per instructions.

---

## Part B

Gaps G9, G11, G12, G16. All frames built in `Nomi.pen`, desktop dark/light and mobile dark/light, using only
existing variables (no `SetVariables`). Part A's frames were not changed. No `placeholder` flags are left. A final
layout-problem scan of all 68 new step frames (visible nodes) found 0 problems, and there are 0 overlaps between
top-level frames anywhere on the canvas.

### Canvas layout change (read before the next round)
- **Attachments flows (Iyq9S / ORByl) have two more rows:**
  - `Steps · UXP-001 answers`: dark **w5Zb1L**, light **wAtbD**. Steps 16–22, 7 steps, so there is room for 1 more.
  - `Steps · UXP-001 viewer`: dark **X4u51c**, light **IxzLO**. Steps 23–27.
  - Each flow is now 4052 px tall.
- **Chat rows moved down 1906 px (2 × 953).** These are the same 40 top-level nodes part A moved: Manage chat → Shared link, all four columns plus their labels.
  - Shared link now starts at y=44556 and ends at 45749. The Build group is still at 46890, so about 1140 px are free.
- **Design system frame grew.**
  - New section `Section / Chat files · UXP-001` (**UqFLI**) at the end of `Design System / Chat responses` (Ljvtr). Ljvtr now ends at y=41727.
  - To keep the gap, `Label / Design System / Build` (nCyOl) and `Design System / Build` (DcG0H) moved down 4032 px, from 37900 to 41932. They now end at 53726.
  - `Design System / Projects` (54200) was not moved. The gap is 474 px.
- **Other flows grew sideways only:**
  - Mobile attachments steps (FLwdb / Fpgnv): to 29.
  - Errors & recovery: TpuRc / Nud6J scenario 6 has 7 steps, AyrHG / uuCam scenario 6 has 6.
  - Share & delete: 7 desktop steps, 7 mobile.
  - Shared link: 6 steps.

### New components
All are in `Section / Chat files · UXP-001` (UqFLI) in Ljvtr.

| Component | Id | Board | Notes |
|---|---|---|---|
| `Chat / Agent File` | **JSxfH** | Board / Agent files XXGd8 | Tile + name + meta, a "what it contains" line, actions `Download` (Button / Secondary) · `Preview` · `Save to project` (Button / Ghost). Width is fill, so it is used at 720 (desktop) and 358 (phone). |
| `Chat / Agent File / Generating` | **vRi2e** | XXGd8 | Real step + elapsed + `Stop`. No bar. |
| `Chat / Agent File / Expired` | **s1yQM** | XXGd8 | No download, `Make it again`. |
| `Chat / File Viewer` | **G534E** | Board / File viewer rCsM1 | Side panel 440 wide. Header (name, meta, Download, Close), cite bar (`[3]`, the claim, page 3 / 12 with prev/next), page with highlighted rows, footer with the highlight total and `Page 8`. |
| `Chat / File Viewer / Loading` | **HVM9y** | rCsM1 | Page-shaped skeleton + `Opening page 3 of 12 · 0:02`. |
| `Chat / File Viewer / Error` | **K1phVf** | rCsM1 | What, why (mono code), `Try again` · `Download`. |
| `Chat / File Viewer / Sheet` | **l8KfHe** | rCsM1 | First 20 rows of the CSV, `First 20 of 214 rows`, `Download to see all`. |
| `Chat / File Viewer / Image` | **rCBA7** | rCsM1 | Lightbox: zoom, Download, Close, `esc closes`. |
| `Mobile / File Viewer` | **GbEIf** | Board / Mobile file viewer D5g9C | Full-screen sheet, 390 × 756. It sits between Status Bar and Home Indicator. |
| `Mobile / File Viewer / Loading` | **b3lX47** | D5g9C | |
| `Mobile / File Viewer / Error` | **nQ4Iq** | D5g9C | Uses Mobile / Button / Primary + Secondary. |
| `Mobile / File Viewer / Sheet` | **snTHJ** | D5g9C | First 12 rows, `Download to see all`. |
| `Mobile / File Viewer / Image` | **LiYjQ** | D5g9C | |
| `Chat / Share Dialog / Files` | **e1shsD** | Board / Share with files niN59 | Toggle off (default). |
| `Chat / Share Dialog / Files Open` | **zWgDC** | niN59 | Toggle on. |

The two share dialogs are built from eemXK's structure, as a copy and not an instance, because pen can't add a row inside an instance. The only differences from eemXK:
- Maya's preview.
- A `Files` line listing the file names.
- A new option row `Let viewers open files` with `Toggle / Off` CAJtO or `Toggle / On` D1S92S.

ui-engineer should build this as eemXK plus one option, not as a second dialog.

---

### G9 · File can't be read
Pattern:
- A `Callout / Error` (mhAYP) instance inside Maya's message, placed under her answer to the files she *could* read.
- Header: icon + `Couldn't read <file>` + a mono reason code + type/size. The type/size is hidden on phone.
- Details: Maya's line, plus the D3 estimate where it applies.
- The callout's Actions bar holds `Chat / Quick Replies` (K4uh0l) in place of its buttons.
- Sources lists only the files that were read.

| Surface | Locked | Scanned | Damaged | Zip partial |
|---|---|---|---|---|
| Desktop dark (Iyq9S, row w5Zb1L) | Step 16 **a1cvaM** | Step 17 **jHLXS** | Step 18 **jEVTP** | Step 19 **P4LQp** |
| Desktop light (ORByl, row wAtbD) | Step 16 **VT0HL** | Step 17 **fXXsl** | Step 18 **G5sHKu** | Step 19 **B4Jkj** |
| Mobile dark (FLwdb) | Step 18 **obOdo** | Step 19 **g3LRC** | Step 20 **tEAce** | Step 21 **t1fDK** |
| Mobile light (Fpgnv) | Step 18 **MbibE** | Step 19 **jYJRJ** | Step 20 **rDXIQ** | Step 21 **DqRJc** |

Errors & recovery, scenario 6, "A file Maya can't read, then replaced":

| Surface | Step |
|---|---|
| Desktop dark (TpuRc) | Step 7 **aF3PA** |
| Desktop light (Nud6J) | Step 7 **Ev7si** |
| Mobile dark (AyrHG) | Step 6 **K2r8A** |
| Mobile light (uuCam) | Step 6 **E1NuW** |

The step shows the locked callout, then the composer holding the unlocked copy:
- Chip `bca-oct-2026.pdf` · `PDF · 11 pages · unlocked`.
- Note `replaces the locked copy` (desktop) or `Replaces the locked bca-oct-2026.pdf in this chat.` (mobile).
- Composer text `Here's the unlocked copy.`

Copy, per case:
- **Locked**
  - Files `bca-sep-2026.pdf` + `bca-oct-2026.pdf`.
  - Maya: "I could only read September. Rent was Rp 2,2 jt to Bu Sri and food was Rp 1,75 jt. October isn't in this answer yet."
  - Callout `Couldn't read bca-oct-2026.pdf` · `locked · password` · "bca-oct-2026.pdf is locked with a password. Upload an unlocked copy and I'll read it." · `Replace file`.
- **Scanned (D3)**
  - Files `bca-sep-2026.pdf` + `nota-sep-2026.pdf` (`PDF · 18 pages`).
  - Callout `Couldn't read nota-sep-2026.pdf` · `no_text · scanned` · "This PDF is scanned images, so there's no text to read. I can read the pages as images."
  - G7-style estimate: `nota-sep-2026.pdf · 18 pages · about 41.000 tokens` / wallet icon `Runs on Nomi credits · ≈ Rp 800`.
  - Actions `Read as images` `Skip`.
  - The answer covers the statement: food Rp 1,75 jt, Rp 1,12 jt on p. 3 + Rp 630 rb on p. 8.
- **Damaged**
  - Files `invoice-mei.xlsx` (`XLSX · 24 KB`) + `invoice-juni.xlsx` (`XLSX · 31 KB`).
  - Maya answers June: Rp 12.450.000 across 7 invoices, biggest PT Sinar Jaya Rp 4.200.000.
  - Callout `Couldn't open invoice-mei.xlsx` · `damaged` · "invoice-mei.xlsx looks damaged — I couldn't open it. Export it again from Excel or Google Sheets and upload the new copy." · `Replace file`.
- **Zip partial**
  - File `assets.zip` · `ZIP · 9 files · 14,2 MB`.
  - Maya answers from the 6 files she read.
  - Callout `Skipped 3 files in assets.zip` · `partial · 6 of 9 read` · "I read 6 of 9 files in assets.zip. Skipped: 3 .psd files. Export them as PNG if you need them on the page." · `See skipped`.

Copy I added: the second sentence in the damaged and zip lines, the callout titles and codes, and Maya's answers to the files she could read.

### G11 · Maya hands back a file
| Surface | Generating | Ready | No longer stored |
|---|---|---|---|
| Desktop dark (row w5Zb1L) | Step 20 **LE0sS** | Step 21 **HcwU7** | Step 22 **FbWFY** |
| Desktop light (row wAtbD) | Step 20 **Rj2cf** | Step 21 **pEABq** | Step 22 **RKhjR** |
| Mobile dark (FLwdb) | Step 22 **XuJTt** | Step 23 **p1Z7mN** | Step 24 **m4EvW0** |
| Mobile light (Fpgnv) | Step 22 **Mm4pd** | Step 23 **Uo6A1** | Step 24 **Ct8u1** |

Layout:
- The card sits under a `MADE BY MAYA` label, styled like the Sources title.
- `SOURCES` (your attachments, `your attachment` / `yours`) is listed separately below it, so produced output is never mixed with uploads.
- In the Generating step, Sources and response actions are hidden because the reply is still running.

Copy:
- User: "Put both months in one CSV I can open in Excel."
- Card: `pengeluaran-sep-okt-2026.csv` · `CSV · 18 KB · 2 months, 214 rows`. Contents line: "Every September and October transaction from your two statements, one row each, with rent and food tagged."
- Actions: `Download` `Preview` `Save to project`.
- Generating: `Writing rows from bca-oct-2026.pdf` · `0:06` · `Stop`.
- No longer stored:
  - Meta `CSV · 18 KB · no longer stored`.
  - "This file was deleted from the chat on 14 Nov 2026, so there's nothing to download. Maya can make it again from the same two statements."
  - Action `Make it again`.
- `Save to project` context: the top bar reads `Rent and food / project · budget-rumah`. On mobile, the nav subtitle is `Maya · budget-rumah`.

### G12 · Preview and jump to a citation
| Surface | Citation [3] → page 3 | Loading | Error | Photo lightbox | Sheet preview |
|---|---|---|---|---|---|
| Desktop dark (row X4u51c) | Step 23 **z3WgN** | Step 24 **IOcPG** | Step 25 **FZYBl** | Step 26 **l91ie** | Step 27 **Z4ulB** |
| Desktop light (row IxzLO) | Step 23 **d5tGK** | Step 24 **AvQxk** | Step 25 **rN0MN** | Step 26 **MaKLC** | Step 27 **wOHkC** |
| Mobile dark (FLwdb) | Step 25 **P3rVq** | Step 26 **QVezi** | Step 27 **HZNUG** | Step 28 **VHx9h** | Step 29 **LifdT** |
| Mobile light (Fpgnv) | Step 25 **jWohS** | Step 26 **aQBES** | Step 27 **MSSmK** | Step 28 **V42TL3** | Step 29 **A6XcF** |

Desktop:
- The viewer panel replaces the Crew panel, in the same slot as the `Workspace / Sheet` pattern. Main narrows to 824.
- The answer's body is one claim per line with `Chat / Citation` chips after each line. Pen text can't hold inline chips.
  - "Rent was the same both months, Rp 2,2 jt to Bu Sri." `[1][2]`
  - "Food went from Rp 1,75 jt in September to Rp 2,1 jt in October." `[3][4]`, with `[3]` active.
- Sources (mrOeg) lists 4 rows:
  - [1] `bca-sep-2026.pdf · page 8`
  - [2] `bca-oct-2026.pdf · page 2`
  - [3] `bca-sep-2026.pdf · pages 3, 8` · `open · page 3`
  - [4] `bca-oct-2026.pdf · pages 2, 7`
  - Rows 2–4 replace the instance's Source 2 slot. This is an override, not a detach.
- Active citation style: `$nm-accent-bg` fill + `$nm-accent` stroke. It is an instance override, not a new C6xkT variant.
- Page 3 of `bca-sep-2026.pdf` shows 14 statement rows. The 9 food rows are highlighted, and they total `Rp 1.120.000`. With the Rp 630 rb on page 8, that makes Sep food Rp 1,75 jt.
- The cite bar reads `Food in September · cited on pages 3 and 8`, and the footer has `Page 8`.
- Photo: the lightbox covers main + crew, so it reads the same in both themes.
  - The photo was unnamed in step 4, so I named it `struk-belanja-okt.jpg` · `JPG · 1,8 MB`.
- Sheet: opened from `Preview` on Maya's card. The Preview button is shown pressed.

Mobile:
- The viewer is a full-screen sheet that opens from tapping `[3]` in Sources. There are no inline chips on the phone.
- It shows the first 12 rows on the sheet.

Copy:
- Loading `Opening page 3 of 12 · 0:02`.
- Error: `The preview didn't load` · "Your file is fine and still in this chat. Try again, or download it and open page 3 yourself." · `preview_timeout · 30 s` · `Try again` `Download`.

### G16 · Files in a shared chat link
The **share dialog steps were added to Share & delete**, because that is where the dialog already lives (Step 1). The **viewer pages were added to Shared link.**

| Surface | Dialog, files private (default) | Dialog, files open | Viewer, names only | Viewer, files open |
|---|---|---|---|---|
| Desktop dark | TnscX Step 6 **jWihN** | TnscX Step 7 **eGAIJ** | b8Flw8 Step 5 **RCgbo** | b8Flw8 Step 6 **y8IKKB** |
| Desktop light | lSpCm Step 6 **UpQs9** | lSpCm Step 7 **C8R438** | Wgdu2 Step 5 **aoKlu** | Wgdu2 Step 6 **iBFXM** |
| Mobile dark | yoVD6 Step 6 **w7JuxE** | yoVD6 Step 7 **GDoGz** | Yyjkb Step 5 **YWmux** | Yyjkb Step 6 **TkJne** |
| Mobile light | Pjc23 Step 6 **bot0x** | Pjc23 Step 7 **iKPaT** | e0FI49 Step 5 **U1Ik68** | e0FI49 Step 6 **H5R4YC** |

Copy:
- Toggle `Let viewers open files`.
  - Off helper: `Files stay private. People with the link see file names only.`
  - On helper (my addition): `Anyone with the link can open and download these 2 files. Amounts inside the files aren't hidden.` Mobile drops "the files".
- Preview files line: `bca-sep-2026.pdf · bca-oct-2026.pdf`.
- Viewer meta line: `… · amounts and names hidden · files private` or `… · files open to viewers`. Mobile: `amounts hidden · files private` / `files open`.
- Viewer chips:
  - Off: lock icon + `PDF · private`.
  - On: `PDF · 12 pages · open`.
- Viewer Sources domains: `private · name only` or `opens here · download` (mobile `private` / `opens · download`).
- Viewer note:
  - Off: `M. kept the files in this reply private. You see their names only.`
  - On: `M. lets viewers open these files. Amounts inside them aren't hidden.`

---

### Open issues / for ux-lead
1. **"Expired" vs D1.** Under D1, files are kept until the chat is deleted, so the expired card can only appear after a per-file delete (G13) or a future retention change. I wrote it as "deleted from the chat on 14 Nov 2026" with `Make it again`. Signed-URL expiry should never surface: the client should re-sign on click. The component is still named `Chat / Agent File / Expired` as briefed. Please confirm the copy.
2. **Masking doesn't reach inside files.** "Hide amounts and names" masks the reply only. An opened PDF shows real amounts, so the on-state helper and the viewer note say so. This is my addition and needs a product decision. Option: offer the toggle only when masking is off, or warn as drawn.
3. **Partial zip uses Callout / Error** (danger header), as briefed. A partial success may deserve a warning-toned callout, but none exists, so I didn't invent one.
4. **Invented context.** Project `budget-rumah` (to justify `Save to project`) and the file names `nota-sep-2026.pdf`, `invoice-mei.xlsx` / `invoice-juni.xlsx`, `assets.zip` and `struk-belanja-okt.jpg`. `Rp 800` is derived from G7's example ratio (96.000 tokens ≈ Rp 1.900) and is an example only.
5. **Share dialog duplicate.** e1shsD / zWgDC are a structural copy of eemXK, not instances, because instances can't take an extra row. If eemXK changes, these two won't follow.
6. **Callout titles** carry file names in sans, from the component's Title style. File names are mono in chips, metas, sources and estimates.
7. **Not updated:** flow headers and scenario descriptions (Iyq9S/ORByl/FLwdb/Fpgnv, Errors scenario 6, Share & delete, Shared link), and `docs/ux/flow-registry.md`, per instructions.
8. **Verification limits.**
   - Exports were checked at 0.25–0.8 scale. Exports render on a white backdrop, which is a pen artefact.
   - The scans right after each build sometimes reported "partially clipped" nodes that cleared on a re-scan with bounds inside their parents. The final scan of all 68 steps reports 0.
   - Warnings about `fill_container` outside a layout come from hidden instance slots (Thought Steps `Steps`, Callout `Row / provider`, Sources `Source 2`, Mobile Composer slots), the same as part A.
   - Mobile G9 and recovery threads are bottom-anchored. Their first chip starts about 11–15 px from the top, inside the thread's top padding.

---

## Round 2 fixes

These are the fixes for gate 2 round 1 (`30-ux-verify.md`, fixes 1–12). Every change was made in place in `Nomi.pen`, with existing variables only (`$nm-warning`, `$nm-warning-bg`, `$nm-danger` and `$nm-focus` already existed, and `SetVariables` was not called). No existing frame was deleted. Each root flow I touched had `placeholder` set while I worked on it and unset when it was done.

Checks:
- A final layout-problem scan of every changed or new step, board and header (75 roots, visible nodes) found 0 problems.
- An overlap scan of all 340 top-level frames found 0 overlaps.
- Each section was screenshotted, in dark and light where it changed.

Known exclusions from the scan:
- The absolute model menu and toasts, which sit outside their parents on purpose.
- The `fill_container` warnings from hidden instance slots, the same ones as in rounds A and B.

### Canvas layout change (read before the next round)
**Desktop Attachments rows were reflowed.** Each row still holds at most 8 steps, and the steps are renumbered **8–31**.

| Row | Dark | Light | Steps |
|---|---|---|---|
| files | vnsf3 | P5FFEI | 8–15 |
| answers | w5Zb1L | wAtbD | 16–23 |
| viewer | X4u51c | IxzLO | 24–31 |

To make room:
- `HAczP` / `GAeyu` (G7 BYOK) moved to the start of the answers row.
- `FbWFY` / `RKhjR` (deleted file) moved into the viewer row.

| Old step | New step |
|---|---|
| 8–12 | 8–12 |
| — | **13 no capable model (new)** |
| 13 | 14 |
| 14 | 15 |
| 15 | 16 |
| 16–20 | 17–21 |
| — | **22 file failed (new)** |
| 21 | 23 |
| — | **24 saved (new)**, **25 save failed (new)** |
| 22 | 26 |
| 23–27 | 27–31 |

**Mobile Attachments (FLwdb / Fpgnv) is renumbered 1–33.**

| Old step | New step |
|---|---|
| 1–14 | 1–14 |
| — | **15 no capable model** |
| 15–22 | 16–23 |
| — | **24 file failed** |
| 23 | 25 |
| — | **26 saved**, **27 save failed** |
| 24–29 | 28–33 |

**Shared link:** a new step 6 was added, and the old step 6 is now 7.
**Share & delete:** a new step 8 was added.

**Flow sizes:**
- The flow headers are taller because their copy wrapped. The Attachments header went from 112 to 178 px, and the eyebrow now wraps inside a fixed 880 px header (vg1Um / UXuwv / q7ER6k / uxzqU are now `width: 880`).
- Desktop Attachments flows grew 82 px. The gap below Iyq9S / ORByl is now 158 px.

**Design system:**
- New row `Row · round 2` (**hBnZ7**) in `Section / Chat files · UXP-001` (UqFLI) holds `Board / Callouts` (**r17Ay**).
- `Board / Agent files` (XXGd8) gained label **BQD1K** and the Failed component.
- Ljvtr grew 478 px, so these moved down 478 px, which keeps the old gaps:
  - `Design System / Build` (DcG0H) and its label (nCyOl): 41932 → 42410.
  - `Design System / Projects` (URIB6) and its label (W6GRLD): 54200 → 54678.

### New components
| Component | Id | Where | Built from |
|---|---|---|---|
| `Callout / Warning` | **S971Sp** | `Board / Callouts` r17Ay (in row hBnZ7) | Structural copy of `Callout / Error` mhAYP |
| `Chat / Agent File / Failed` | **xyZCb** | `Board / Agent files` XXGd8, label BQD1K | Structural copy of `Chat / Agent File / Expired` s1yQM |

**`Callout / Warning` (S971Sp):**
- Header fill `$nm-warning-bg`. Icon `triangle-alert` and title in `$nm-warning`.
- Outer stroke `$nm-border-strong`, the same pairing as the `rrjYM` chip.
- Header id map (Error → Warning): C0hQt→R0fge8 · jzkWd→Iy7b7 · TRkW0→mg9tp · zYjOx→J2XQi · yQXVL→vfiTB.
- Body id map (Error → Warning): O2OSR0→ljQKk · n92ETq→m1wUNb · NjkZ8→HhQXu · EHOUH→OJTBP.
- Default action: `See skipped`.

**`Chat / Agent File / Failed` (xyZCb):**
- Icon `file-x` in `$nm-danger`, name, meta `CSV · not made` (mono).
- Mono code `write_failed` (OcAax) in `$nm-danger`. The card has no danger fill.
- Line: "Maya couldn't finish this file. Nothing was saved, and the two statements are still in this chat."
- Actions: `Try again` (Button / Secondary, kPdz4) and `Ask Maya why` (Button / Ghost, kbHxu).

### Per fix
**1 · G7 meter.**
- Meter text is now `2% used` and `≈ 12% used after this read`. The fill bar is 2% of the track: 4 px at 200 px wide, 7 px at 358 px wide.
- Meter instances: Z3efXY → YT3x5 · u34vwy → wRymc · z274p9 → cOebw/zVeXz · EO53V → KBz6d/x0Xoqv.
- M4 (Rp vs %) is not decided. The `≈ Rp 1.900` line is unchanged.

**2 · G11 row count.**
- 214 is now 142 everywhere (17 text overrides plus the components):
  - JSxfH meta `CSV · 18 KB · 2 months, 142 rows`.
  - l8KfHe `First 20 of 142 rows`.
  - snTHJ `First 12 of 142 rows`.
  - The `Done: 142 transactions…` bodies.
  - The captions of Z4ulB / wOHkC / LifdT / A6XcF (`Download to see all 142`).
- A search of Iyq9S, ORByl, FLwdb, Fpgnv and the three components found no `214` left.

**3 · G9 tone.** The 8 partial and scanned callouts are now `Callout / Warning` instances. They were swapped with Replace, the overrides were remapped, and the copy is unchanged.

| Case | Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|---|
| Zip partial | P4LQp → **bYazl** | B4Jkj → **WPhDu** | t1fDK → **p03cO** | DqRJc → **LohED** |
| Scanned | jHLXS → **IJOvD** | fXXsl → **ELvfx** | g3LRC → **d1h2Wl** | jYJRJ → **Y494On** |

- The per-file icon overrides (e.g. `file-archive`) were dropped so that `triangle-alert` shows, per the fix.
- Locked (a1cvaM…) and damaged (jEVTP…) still use `Callout / Error`.

**4 · G12 mobile cite.**
- The cite chip reads `1`: P1vmd0/gJynJ/XnByL in P3rVq, and beQw5/gJynJ/XnByL in jWohS.
- The captions GNrko / Y8P7e now say `Tapping [1] in Sources opens bca-sep-2026.pdf full screen at page 3, cited rows highlighted. Swipe down to go back.`

**5 · G6 model name.**
- The `rrjYM` meta (F76Rd) is now `deepseek-v3 can't see images` in mono, along with the 4 instance overrides.
- The sent notes aw1Dn · XWmYt · aBbKs · UNt4f now read `Sent without struk-belanja-nov.jpg · deepseek-v3 can't see images`.
- **Ruling A2 (Send disabled, focus on Switch model).**
  - Desktop: the focus ring is drawn. `Switch model` sits in a new frame with an outer 2 px `$nm-focus` stroke: **XHDLj** (deDgy) and **nkhFC** (XUc6r).
  - Desktop captions RHdh0 / RNPmg: "…Send stays off and ↵ doesn't send until you pick a fix, and focus lands on Switch model."
  - Mobile captions u4dTo / x8PPj gain "Send stays off until you pick one."
  - **Not drawn on mobile:** I didn't draw a focus ring on the phone pills. There's no hardware-keyboard focus there, and screen-reader focus isn't a visible ring. Tell me if you want one.

**6 · G16 masked link with open files.** In the files-open dialogs (component zWgDC, instances eGAIJ / C8R438; mobile GDoGz / iKPaT):

| Change | Copy | Ids |
|---|---|---|
| `Hide amounts and names` turned off | Toggle / Off CAJtO | **b8yeZM**, **f0k0mX**, **YtrSI** |
| Hide helper | `Off while viewers can open files. Amounts inside files can't be hidden.` | U3YvC, bw1K8, cDNk8. bw1K8 and cDNk8 now wrap. |
| Files-open helper, desktop | `Anyone with the link can open and download these 2 files, with every amount in them.` | Qk1IN |
| Files-open helper, mobile | `…these 2 files, amounts included.` | T87eW, w2gfr |
| Dialog preview unmasked | `Rp 2,2 jt to Bu Sri…` | v9QLbS, VzOGu, xi92O |
| Step captions: turning Hide on closes files | — | zQmk8, IyKMt, g0JMF, Ul29G |

The dialog preview was unmasked because Hide is off.

Viewers y8IKKB / iBFXM / TkJne / H5R4YC:
- The reply is unmasked: `Rp 2,2 jt to Bu Sri … Rp 1,75 jt … Rp 2,1 jt`.
- Meta: `expires in 7 days · files open to viewers` (mobile: `files open`).
- Note: `M. lets viewers open these files.`
- The captions mention that Hide is off.

The default dialogs (jWihN · UpQs9 · w7JuxE · bot0x) and eemXK are unchanged. M2 (server-side redaction) and M3 ("and names") are not decided, so the existing `Hide amounts and names` / initials copy stays.

**7 · Save to project.** New steps 24 and 25.

| Step | Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|---|
| Saved | **mXjAV** | **yck4A** | **rmDzX** | **bIrng** |
| Save failed | **htKXj** | **g2zGl** | **RA75h** | **j9gTY** |

Saved:
- `Save to project` becomes a quiet confirmation: a check icon, `Saved to budget-rumah`, the mono path `budget-rumah/pengeluaran-sep-okt-2026.csv`, and a `Link` (R64KKR) `Open`.
- Desktop: the confirmation replaces the button in the card (M2JbGw / b4FC6v).
- Mobile: the actions become Download · Preview, with the confirmation on its own row underneath (T8PJ78 / y1v4W), because a 358 px card can't fit it on one line.

Save failed:
- Toast / Error eT6bB (desktop **S1wMx** / **adA8H**) and Mobile / Toast SCzar with a `circle-alert` icon in `$nm-danger` (**WDMsd** / **mr0II**).
- Copy: title `Couldn't save to budget-rumah`, mono detail `storage_full · the file is still in this chat`, action `Try again`. The card keeps `Save to project`.
- I put the code in the mono detail line rather than the title, so that it stays mono.

**8 · File generation fails.** New component **xyZCb** (above), shown in new steps:

| Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|
| **M9fOk** (step 22, card kxQoK) | **j8tL6** (step 22, card rTFNR) | **Zh65u** (step 24, card v6L24G) | **ecvv2** (step 24, card ZjbeH) |

- The byline reads `1 file not made · 0:14`.
- Maya: "I couldn't finish the CSV from your two statements."
- The thought steps read `writing CSV failed · 0:14`.
- Response actions are back on, because the reply ended. Sources stay hidden.

**9 · No model can see images.** New step 13 (desktop) / 15 (mobile).

| Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|
| **alcxf** (menu er95B) | **j3PeSH** (menu sn1jx) | **s10RHA** (sheet Sc4iM) | **g1YGU** (sheet Tpknj) |

- Title `No model here can see images`.
- Mono line `deepseek-v3 and 2 others hidden · can't see images`.
- Row `Add one in Models & providers` with a link arrow.
- Secondary `Send text only`.
- The desktop menu is 392 px wide (it was 320), so the mono line fits. It is anchored above the model chip.
- Captions: "When none of your models can see images (some BYOK models are text-only), …".

**10 · Maya's file in a shared link.**

New Shared link step 6, "Maya's file in a shared reply":

| Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|
| **gpoF2** | **a9Wfi** | **BBZuX** | **tRiRa** |

- The reply is the CSV reply. The JSxfH card sits under `MADE BY MAYA` with a lock icon and `CSV · 18 KB · private` (meta frames cDToL / l14Lzb / jmO48 / xAl48). It has no Actions row: no Download, Preview, Save to project or Make it again.
- The files note gains "including the one Maya made" (mobile: "Maya's too").
- The caption states the files-open rule: only `Download` and `Preview`, never `Save to project` or `Make it again`.
- I hid the upload chips on the user's message in this step. The CSV request had no attachments, the same as in Iyq9S step 23. On the phone this also keeps the thread from overflowing.

New Share & delete step 8, "Share a reply with Maya's file":

| Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|
| **O35v6r** | **TuPME** | **RCHEQ** | **x92w7** |

- Each step is the ready chat with the files-private share overlay. Scrims: K195G / MW4kb / un860 / furlK.
- The Files line lists `bca-sep-2026.pdf · bca-oct-2026.pdf · pengeluaran-sep-okt-2026.csv`.
- To make that fit, the `Files` row in **e1shsD** (N105Dx) and **zWgDC** (NuIbD) now wraps. This is a layout change only, and short lists look the same as before.

**11 · Headers.**

| Header | Desktop dark | Desktop light | Mobile dark | Mobile light |
|---|---|---|---|---|
| Attachments | mCiph, HOpC0 | BOla1, ZMKvz | bykYj, hW6kY | IMuUK, t7IFda |
| Errors scenario 6 | D9zrJ: cSTic, oq6A1 | SQSJJ: GoJ5Q, a03rm | diWmy: Ez7rs, k1YTPQ, fZdmv | ZTS3M: j1GoYm, WNNrG, NWmv8 |
| Shared link | OzvQt, N9kuRZ | riH8g, OB56y | HcCuB, LoTJR | oNZtE, d8iF6q |
| Share & delete | tkYN7, y6oIv | pNbqb, lT552 | OLDwg, XvR33 | r94djJ, gh9Ht |

- **Attachments:** the eyebrow and description now name upload failed, unsupported, model can't see, cost, can't read, Maya's files and the viewer.
- **Errors scenario 6:**
  - Eyebrow `… · UPLOAD · TYPE · UNREADABLE`, plus one sentence.
  - The mobile title is now `Send failures, files and Nomi credits`.
- **Shared link and Share & delete:** the eyebrow and description now cover file privacy and Maya's file.

**12 · Statement content.**
- Page 3 of `G534E`: the `01/09 SALDO AWAL` row (uS5r0) is hidden (`enabled: false`), not deleted, so the page now runs `03/09 … 27/09`. The 9 highlighted rows still add up to `Rp 1.120.000` (45.000 + 128.500 + 312.000 + 96.000 + 38.000 + 154.500 + 142.000 + 168.000 + 36.000).
- CSV `l8KfHe`:
  - The September rent row (OhAux) is re-dated `28/09/2026` and moved into date order.
  - The October rent at `28/10/2026` falls after the first 20 rows, so it isn't shown. Its slot (VgqEo, `01/10/2026`) is now `Alfamart Tebet · Food · 54.000`, which is invented to keep the 20 rows in date order.
- Mobile `snTHJ`: the old 02/09 rent row (vTIhN) is now `27/09/2026 · QRIS Warung Bu Tini · Food · 36.000`, moved last, so the first 12 rows stay in date order.
- **Flag:** Sources cite October rent on `bca-oct-2026.pdf · page 2`, and a `28/10` date on page 2 of 11 is unusual. I followed the ruling. If page 2 is wrong, change it in Sources next round.

### Not done / left as is
- **M1–M4 are not decided.**
  - The G3 "Resuming · 41%" copy, the `Hide amounts and names` / initials copy, and the Rp estimate next to the % meter are unchanged.
  - No server-side redaction is implied anywhere.
- **Expired = deleted.** The copy was already "deleted from the chat". The "sources gone, `Make it again` hidden" variant is contract text only, as the ruling says, so it isn't drawn.
- **Share dialogs.** e1shsD and zWgDC are still structural copies of eemXK, and eemXK itself is untouched. ui-engineer builds one dialog: eemXK plus an optional Files line and the `Let viewers open files` row.
- **Not updated:** `docs/ux/flow-registry.md` and other repo files, per instructions.
- **Verification limits.**
  - Screenshots render small. Close-ups were taken of the menus, cards, callouts, dialog and toasts.
  - Some post-edit scans briefly reported stale "clipped" flags, which cleared on a re-scan or after a size update. The final scan is 0.
