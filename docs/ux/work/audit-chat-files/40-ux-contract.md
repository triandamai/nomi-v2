---
work: audit-chat-files
status: verified
verified_by: ux-lead
verified_on: 16 Nov 2026
proposal: UXP-001
scope: P0 gaps G3, G4, G6, G7, G9, G11, G12, G16
gate2_rounds: 2 (round 1 changes-required, round 2 pass; see 30-ux-verify.md)
---

# UX contract — audit-chat-files

The engineer builds exactly this. Anything not here is out of scope or a question.
Frame ids are `Nomi.pen` node ids. Every scenario exists in four versions, always listed as
**desktop dark / desktop light · mobile dark / mobile light**. Light is the same frame with
`theme: {mode: light}`. Its copy is identical to dark, except where noted.

Flows:
- **Attachments, voice & limits:** Iyq9S / ORByl · FLwdb / Fpgnv.
  - Desktop steps 1–7 are Mario's reference steps. UXP-001 adds desktop steps 8–31 in rows vnsf3 / P5FFEI (8–15), w5Zb1L / wAtbD (16–23) and X4u51c / IxzLO (24–31).
  - Mobile steps 1–8 are reference steps. UXP-001 adds mobile steps 9–33.
- **Errors & recovery, scenario 6:** TpuRc / Nud6J · AyrHG / uuCam.
- **Share & delete:** TnscX / lSpCm · yoVD6 / Pjc23, steps 6–8.
- **Shared link:** b8Flw8 / Wgdu2 · Yyjkb / e0FI49, steps 5–7.

Out of scope (P1/P2, not built in this round): drag and drop (G1), paste (G2), the per-message
file limit (G5), long-read progress (G8), partial reads of big files (G10), the Files list (G13),
editing a message with files (G14), offline with files (G15) and the drawn keyboard path (G17).
The Accessibility section still binds this build.

## Scenarios

| # | Scenario | Pen frames (desktop dark / light · mobile dark / light) | Steps |
|---|---|---|---|
| **G3** | **Upload failed** | | |
| 1 | Upload fails mid-way | Q7n2MM / fhdWO · rrKfg / QAikj (D8 · M9) | 1. One chip fails and the other chips carry on. 2. The failed chip shows `Upload failed · connection lost` with `Retry` and `Remove`. On mobile it shows a `⋯` button instead. 3. Send is disabled, and the Send hint becomes `1 file didn't upload. Retry or remove it to send.` On mobile the hint is the composer Notice line. 4. `↵` does not send. |
| 2 | Retry or remove (mobile sheet) | — · BhXcx / APtff (M10) | 1. Long-press or tap `⋯` to open the sheet. 2. The header shows the file name and reason. 3. The rows are `Retry` (meta per M1) and `Remove` (meta `stays on your phone`). |
| 3 | Retrying | N3Iyy / vTWuT · bITK4 / QSA24 (D9 · M11) | 1. The chip goes back to progress, `Resuming · 41% · 2,1 of 5 MB` (or the restart copy, per M1). 2. Send waits until every chip is ready. |
| 4 | Removed | yhuBX / F5kA4 · S61SO / g2x0P (Errors D5 · M4) | 1. `Remove` drops the chip from the message, never from the device. 2. The line reads `bca-oct-2026.pdf removed · 1 file ready`. 3. Send is enabled for the files that made it. |
| **G4** | **File type not supported** | | |
| 5 | Unsupported file picked | IvQAw / IpiJU · rEdL2 / WtYon (D10 · M12) | 1. The type check runs on the client before any upload request. 2. The chip shows `.dmg files can't be read`. 3. A helper line under the chips reads `Maya reads PDF, images, sheets and zip.` 4. The chip has `Remove`. 5. Send is disabled, with the hint `Remove kasir-setup.dmg to send.` |
| 6 | Mixed pick | ymrwy / EF25o · U6S0r / WUqGg (Errors D6 · M5) | Of 3 files, 1 is ready, 1 is uploading and `catatan-kas.pages` is rejected. The others keep going. `.heic` is never rejected: it is converted on upload (D2). |
| **G6** | **Model can't see the file** | | |
| 7 | Warning | deDgy / XUc6r · sE5RV / a7GWtv (D11 · M13) | 1. The chip meta reads `deepseek-v3 can't see images` (mono). 2. The fix line reads `Switch to a model that can see images, or send your message without the photo.` On mobile it drops "your message". 3. The actions are `Switch model` and `Send text only`. 4. Send is disabled and `↵` doesn't send. 5. Focus moves to `Switch model`: on desktop the ring is XHDLj / nkhFC. |
| 8 | Switch model | mDEso / CSvJi · DgnEc / W3w6z (D12 · M14) | The model menu (the mobile sheet `Switch model` · `Models that can see images`) lists only models with the `vision` capability. Hidden models are counted: `deepseek-v3 hidden · can't see images`. Picking one turns the chip to Ready and enables Send. |
| 9 | No capable model | alcxf / j3PeSH · s10RHA / g1YGU (D13 · M15) | 1. The filtered list is empty. 2. The title reads `No model here can see images`. 3. A mono line reads `deepseek-v3 and 2 others hidden · can't see images`. 4. The row `Add one in Models & providers` opens Settings › Models & providers. 5. The secondary action is `Send text only`. 6. The draft (text and chip) is kept while the user is in Settings and is there on return. |
| 10 | Send text only | JVUHc / xA6ae · WDeji / L3Vmo (D14 · M16) | 1. The message is sent without the file. 2. The bubble note reads `Sent without struk-belanja-nov.jpg · deepseek-v3 can't see images` (mono). 3. Maya's reply says she didn't get the photo. The file is not uploaded. |
| **G7** | **Cost before a big read** | | |
| 11 | Nomi credits | Z3efXY / u34vwy · z274p9 / EO53V (D15 · M17) | 1. Above the threshold, a quiet row sits above Send: `laporan-keuangan-2026.pdf · 184 pages · about 96.000 tokens`. 2. Below it: `Runs on Nomi credits · ≈ Rp 1.900`. 3. The Meter shows `Nomi credits` · `2% used` · `≈ 12% used after this read`, in the same unit and direction as the sidebar plan meter. 4. Below the threshold, nothing extra shows. |
| 12 | Own key (BYOK) | HAczP / GAeyu · ch6hu / kin2h (D16 · M18) | The estimate line plus `Runs on your Anthropic key`. No meter and no Rp. |
| **G9** | **File can't be read** (one message per case; the readable files are still answered) | | |
| 13 | Locked | a1cvaM / VT0HL · obOdo / MbibE (D17 · M19) | Maya answers September, then shows a `Callout / Error` (mhAYP): `Couldn't read bca-oct-2026.pdf` · `locked · password`, Maya's line, and `Replace file`. |
| 14 | Locked → replaced | aF3PA / Ev7si · K2r8A / E1NuW (Errors D7 · M6) | 1. `Replace file` opens the picker. 2. The composer holds chip `bca-oct-2026.pdf` · `PDF · 11 pages · unlocked`. 3. The note reads `replaces the locked copy` (on mobile: `Replaces the locked bca-oct-2026.pdf in this chat.`). 4. Nothing else in the message is read again. |
| 15 | Scanned (D3) | jHLXS / fXXsl · g3LRC / jYJRJ (D18 · M20); callouts IJOvD / ELvfx · d1h2Wl / Y494On | 1. A `Callout / Warning` (S971Sp): `Couldn't read nota-sep-2026.pdf` · `no_text · scanned`. 2. The estimate `nota-sep-2026.pdf · 18 pages · about 41.000 tokens`, then `Runs on Nomi credits · ≈ Rp 800` (or the BYOK line). 3. The actions are `Read as images` and `Skip`. |
| 16 | Damaged | jEVTP / G5sHKu · tEAce / rDXIQ (D19 · M21) | A `Callout / Error`: `Couldn't open invoice-mei.xlsx` · `damaged` · `Replace file`. June is still answered. |
| 17 | Zip partly read | P4LQp / B4Jkj · t1fDK / DqRJc (D20 · M22); callouts bYazl / WPhDu · p03cO / LohED | A `Callout / Warning`: `Skipped 3 files in assets.zip` · `partial · 6 of 9 read` (mobile `partial`) · `See skipped`. |
| **G11** | **Maya hands back a file** | | |
| 18 | Generating | LE0sS / Rj2cf · XuJTt / Mm4pd (D21 · M23) | 1. The card (vRi2e) shows the real step `Writing rows from bca-oct-2026.pdf` and the elapsed time `0:06`, with `Stop`. 2. There is no progress bar. 3. Sources and response actions are hidden until the reply ends. |
| 19 | Generation fails | M9fOk / j8tL6 · Zh65u / ecvv2 (D22 · M24); cards kxQoK / rTFNR · v6L24G / ZjbeH | 1. The card is `Chat / Agent File / Failed` (xyZCb): the name, `CSV · not made`, `write_failed` in `$nm-danger`, the line, `Try again` and `Ask Maya why`. 2. The byline reads `1 file not made · 0:14`. 3. Maya says `I couldn't finish the CSV from your two statements.` 4. Response actions come back. Sources stay hidden. |
| 20 | Stopped while generating (contract text, no frame) | reuse xyZCb | 1. After `Stop`, the card turns into xyZCb with code `stopped` and the line `You stopped Maya before this file was finished. Nothing was saved.` 2. The actions are `Try again` only. 3. The reply ends with the usual `Chat / Stopped` (bTCPw). |
| 21 | Ready | HcwU7 / pEABq · p1Z7mN / Uo6A1 (D23 · M25) | 1. The card JSxfH sits under `MADE BY MAYA`, separate from `SOURCES`. 2. It shows `pengeluaran-sep-okt-2026.csv` · `CSV · 18 KB · 2 months, 142 rows` and the contents line. 3. The actions are `Download`, `Preview` and `Save to project`. `Save to project` shows only when the chat sits in a project; otherwise it's hidden, not disabled. |
| 22 | Saved | mXjAV / yck4A · rmDzX / bIrng (D24 · M26) | 1. The button is replaced by a check icon, `Saved to budget-rumah` and the mono path `budget-rumah/pengeluaran-sep-okt-2026.csv`. 2. `Open` (Link R64KKR) opens the file in the project. 3. On mobile the confirmation sits on its own row under `Download` · `Preview`. |
| 23 | Save fails | htKXj / g2zGl · RA75h / j9gTY (D25 · M27); toasts S1wMx / adA8H · WDMsd / mr0II | 1. A toast: `Couldn't save to budget-rumah`, the mono detail `storage_full · the file is still in this chat`, and `Try again`. 2. The card keeps `Save to project`. |
| 24 | Deleted from the chat | FbWFY / RKhjR · m4EvW0 / Ct8u1 (D26 · M28) | 1. The card s1yQM shows `CSV · 18 KB · no longer stored` and the line, with `Make it again`. 2. It is never a broken link. A signed-URL expiry is never shown: the client re-signs on click. 3. If the source files were deleted too, `Make it again` is hidden and the line becomes the "sources gone" text in the copy table. |
| **G12** | **Viewer and citation** | | |
| 25 | Open a citation | z3WgN / d5tGK · P3rVq / jWohS (D27 · M29) | **Desktop:** `[3]` opens the side panel (G534E, 440 wide) at page 3 of `bca-sep-2026.pdf`. The 9 food rows are highlighted, and the footer reads `Highlighted · 9 food rows · Rp 1.120.000` with `Page 8`. The cite bar reads `Food in September · cited on pages 3 and 8`, with `3 / 12` and prev/next. **Mobile:** tapping `[1]` in Sources opens the full-screen sheet GbEIf. Its cite bar reads `Food in September · p. 3, 8`. Swipe down closes it. |
| 26 | Viewer loading | IOcPG / AvQxk · QVezi / aQBES (D28 · M30) | A page-shaped skeleton and `Opening page 3 of 12 · 0:02`. |
| 27 | Viewer error | FZYBl / rN0MN · HZNUG / MSSmK (D29 · M31) | `The preview didn't load`, Maya's line, `preview_timeout · 30 s`, `Try again` and `Download`. |
| 28 | Photo | l91ie / MaKLC · VHx9h / V42TL3 (D30 · M32) | A lightbox (rCBA7 / LiYjQ) with zoom, Download and Close. On desktop the hint reads `esc closes`. |
| 29 | Preview Maya's file | Z4ulB / wOHkC · LifdT / A6XcF (D31 · M33) | The first 20 rows (desktop) or 12 rows (mobile), then `First 20 of 142 rows` / `First 12 of 142 rows` and `Download to see all`. |
| 30 | Open a sent attachment chip (contract text, no frame) | reuse G534E / GbEIf | The viewer opens at page 1, with the cite bar (RWrTx) and footer (d0ylL) hidden. |
| **G16** | **Files in share links** | | |
| 31 | Share a reply that used files | jWihN / UpQs9 · w7JuxE / bot0x (Share D6 · M6) | 1. The default is files private: the files line plus `Let viewers open files` off, with `Files stay private. People with the link see file names only.` 2. `Hide amounts and names` is on. |
| 32 | Let viewers open files | eGAIJ / C8R438 · GDoGz / iKPaT (Share D7 · M7) | 1. Files open **on** forces Hide **off**. 2. The Hide helper reads `Off while viewers can open files. Amounts inside files can't be hidden.` 3. The files-open helper reads `Anyone with the link can open and download these 2 files, with every amount in them.` (mobile: `…these 2 files, amounts included.`). 4. The preview is unmasked. 5. Turning Hide back on turns files open off. |
| 33 | Share a reply with Maya's file | O35v6r / TuPME · RCHEQ / x92w7 (Share D8 · M8) | 1. The Files line lists uploads and produced files: `bca-sep-2026.pdf · bca-oct-2026.pdf · pengeluaran-sep-okt-2026.csv`. 2. One switch covers all of them. |
| 34 | Viewer, files private | RCgbo / aoKlu · YWmux / U1Ik68 (Link D5 · M5) | 1. Each file shows a lock icon, its name and `PDF · private`. 2. There is no preview, download or page link. 3. Sources read `private · name only` (mobile `private`). 4. The note reads `M. kept the files in this reply private. You see their names only.` |
| 35 | Viewer, Maya's file | gpoF2 / a9Wfi · BBZuX / tRiRa (Link D6 · M6) | 1. The JSxfH card shows a lock icon and `CSV · 18 KB · private`, with no actions. 2. The note reads `…including the one Maya made.` (mobile `Maya's too`). 3. With files open, the card shows `Download` and `Preview` only, never `Save to project` or `Make it again`. That variant is contract text, not drawn. |
| 36 | Viewer, files open | y8IKKB / iBFXM · TkJne / H5R4YC (Link D7 · M7) | 1. The reply is unmasked. 2. The meta reads `expires in 7 days · files open to viewers` (mobile `files open`). 3. Chips show `PDF · 12 pages · open`. Sources read `opens here · download` (mobile `opens · download`). 4. The note reads `M. lets viewers open these files.` |

## State machine

**Attachment chip** (per file, one composer):
- `picked` → client type check against the accepted-type list:
  - not accepted → `rejected` (G4);
  - accepted → `uploading`.
- `uploading` →
  - `ready` on 2xx (`status: ready | pending`);
  - `failed(reason)` on a network drop, 4xx or 5xx. The chip appears within 5 s of the drop.
- `failed` → `Retry` → `uploading`. Under M1, this resumes from the last byte or starts again.
- `failed` or `rejected` → `Remove` → gone. Focus moves to the next chip, or to the field.
- `ready` + an image + a model without `vision` → `model_warning` (G6). Then:
  - `Switch model` (a vision model picked) → `ready`;
  - `Send text only` → the message is sent without this file.
- Picking a non-vision model while an image chip is ready turns the chip to `model_warning`.

**Send button.** It is enabled only when all of these hold:
- no chip is `uploading`, `failed`, `rejected` or `model_warning`;
- the composer text is within the limit.

When Send is disabled, `↵` does nothing, and the hint says why (the copy table has each case).

**Estimate row (G7).** It shows when the sum of the attachments' estimates is over the configured
threshold. It recomputes when a file is added or removed, or the model changes. The variant depends
on the payer:
- `credits`: the estimate, the Rp line and the Meter;
- `byok`: the estimate and `Runs on your <Provider> key`.

**Read result per file (G9):** `ok | locked | no_text | damaged | partial(read, total, skipped[])`.
`ok` files are answered first. Each non-`ok` file gets one callout in the same reply:
- `locked` and `damaged` → `Callout / Error`;
- `no_text` and `partial` → `Callout / Warning`.

Quick replies (`Read as images`, `Skip`, `See skipped`) post the choice as the next user message
(the K4uh0l pattern), and Maya answers. `Read as images` uses the G7 estimate and the usual payer
(D3). `Replace file` opens the picker. The new chip is marked as the replacement.

**Agent file (G11):**
- `generating` →
  - `ready` when the file is written;
  - `failed(write_failed)` when writing fails;
  - `stopped` on `Stop`.
- `failed` or `stopped` → `Try again` → `generating`.
- `failed` → `Ask Maya why` → a user message, then Maya answers.
- `ready` → `Save to project` → `saving` → `saved(path)`, or `save_failed(code)` (toast, card back to `ready`).
- `ready` or `saved` → the file is deleted from the chat (per-file delete, D1) → `deleted`.
- `deleted` → `Make it again` → `generating`, only while every source file still exists.

**Viewer (G12):**
- `closed` → `loading` (from a citation, a Sources row, `Preview`, a chip or a photo).
- `loading` → `page(n, highlights)` or `error(code)`.
- `error` → `Try again` → `loading`.
- Close / `Esc` / swipe down → `closed`. Focus goes back to the trigger.

**Share options (G16):**
- `mask = true, files_open = false` is the default.
- `files_open := true` ⇒ `mask := false`.
- `mask := true` ⇒ `files_open := false`.
- The two are never both true. This is enforced on the client and must be rejected by the server too.

## Copy table (verbatim)

| Key | Text | Where |
|---|---|---|
| upload.failed | `Upload failed · connection lost` (the reason varies: `connection lost`, `server error`, `storage full`) | chip BD98f |
| upload.retry / remove | `Retry` · `Remove` | chip, mobile sheet BhXcx |
| upload.hint | `1 file didn't upload. Retry or remove it to send.` | Send hint (desktop), Notice line (mobile) |
| upload.resuming | `Resuming · 41% · 2,1 of 5 MB` — **M1:** if restart, `Uploading again · 0% · 0 of 5 MB` | chip while retrying |
| upload.sheet.retry_meta | `picks up at 41%` — **M1:** if restart, `starts the upload again` | BhXcx |
| upload.sheet.remove_meta | `stays on your phone` | BhXcx |
| upload.removed | `bca-oct-2026.pdf removed · 1 file ready` | yhuBX |
| type.rejected | `.dmg files can't be read` | chip xpYoV |
| type.helper | `Maya reads PDF, images, sheets and zip.` | under chips |
| type.hint | `Remove kasir-setup.dmg to send.` | Send hint |
| model.warning | `deepseek-v3 can't see images` (mono model id) | chip rrjYM |
| model.fix_line | `Switch to a model that can see images, or send your message without the photo.` (mobile: `…or send without the photo.`) | above the field |
| model.actions | `Switch model` · `Send text only` | fix line |
| model.menu.hidden | `deepseek-v3 hidden · can't see images` | filtered menu / sheet |
| model.menu.sheet | `Switch model` · `Models that can see images` | mobile sheet header |
| model.none.title | `No model here can see images` | er95B / Sc4iM |
| model.none.hidden | `deepseek-v3 and 2 others hidden · can't see images` (mono) | er95B / Sc4iM |
| model.none.add | `Add one in Models & providers` | row with arrow → Settings |
| model.sent_note | `Sent without struk-belanja-nov.jpg · deepseek-v3 can't see images` (mono) | user bubble |
| cost.estimate | `laporan-keuangan-2026.pdf · 184 pages · about 96.000 tokens` (mono) | estimate row |
| cost.credits | `Runs on Nomi credits · ≈ Rp 1.900` (**M4**) | estimate row |
| cost.byok | `Runs on your Anthropic key` | estimate row |
| cost.meter | `Nomi credits` · `2% used` · `≈ 12% used after this read` | Meter i5A8H |
| read.locked | Title `Couldn't read bca-oct-2026.pdf` · code `locked · password` · `bca-oct-2026.pdf is locked with a password. Upload an unlocked copy and I'll read it.` · `Replace file` | Callout / Error |
| read.scanned | Title `Couldn't read nota-sep-2026.pdf` · code `no_text · scanned` · `This PDF is scanned images, so there's no text to read. I can read the pages as images.` · `nota-sep-2026.pdf · 18 pages · about 41.000 tokens` · `Runs on Nomi credits · ≈ Rp 800` · `Read as images` `Skip` | Callout / Warning |
| read.damaged | Title `Couldn't open invoice-mei.xlsx` · code `damaged` · `invoice-mei.xlsx looks damaged — I couldn't open it. Export it again from Excel or Google Sheets and upload the new copy.` · `Replace file` | Callout / Error |
| read.partial | Title `Skipped 3 files in assets.zip` · code `partial · 6 of 9 read` · `I read 6 of 9 files in assets.zip. Skipped: 3 .psd files. Export them as PNG if you need them on the page.` (mobile ends `…if you need them.`) · `See skipped` | Callout / Warning |
| read.replacement | `PDF · 11 pages · unlocked` · `replaces the locked copy` (mobile `Replaces the locked bca-oct-2026.pdf in this chat.`) | composer chip |
| file.label | `MADE BY MAYA` | above the card |
| file.ready.meta | `CSV · 18 KB · 2 months, 142 rows` | JSxfH |
| file.actions | `Download` · `Preview` · `Save to project` | JSxfH |
| file.generating | `Writing rows from bca-oct-2026.pdf` · `0:06` · `Stop` | vRi2e |
| file.failed | `CSV · not made` · `write_failed` · `Maya couldn't finish this file. Nothing was saved, and the two statements are still in this chat.` · `Try again` · `Ask Maya why` | xyZCb |
| file.failed.reply | `I couldn't finish the CSV from your two statements.` · byline `1 file not made · 0:14` | Maya's message |
| file.stopped | `CSV · not made` · `stopped` · `You stopped Maya before this file was finished. Nothing was saved.` · `Try again` | xyZCb (contract text) |
| file.saved | `Saved to budget-rumah` · `budget-rumah/pengeluaran-sep-okt-2026.csv` (mono) · `Open` | card |
| file.save_failed | `Couldn't save to budget-rumah` · `storage_full · the file is still in this chat` (mono) · `Try again` | Toast / Error eT6bB, Mobile / Toast SCzar |
| file.deleted | `CSV · 18 KB · no longer stored` · `This file was deleted from the chat on 14 Nov 2026, so there's nothing to download. Maya can make it again from the same two statements.` · `Make it again` | s1yQM |
| file.deleted.sources_gone | `This file and the files it came from were deleted from the chat, so Maya can't make it again.` (no action) | s1yQM (contract text) |
| viewer.loading | `Opening page 3 of 12 · 0:02` | HVM9y / b3lX47 |
| viewer.error | `The preview didn't load` · `Your file is fine and still in this chat. Try again, or download it and open page 3 yourself.` · `preview_timeout · 30 s` · `Try again` `Download` | K1phVf / nQ4Iq |
| viewer.cite_bar | desktop `Food in September · cited on pages 3 and 8` · mobile `Food in September · p. 3, 8` | G534E / GbEIf |
| viewer.highlight | `Highlighted · 9 food rows · Rp 1.120.000` · `Page 8` (mobile `Highlighted · 9 rows · Rp 1.120.000`) | viewer footer |
| viewer.sheet | `First 20 of 142 rows` (mobile `First 12 of 142 rows`) · `Download to see all` | l8KfHe / snTHJ |
| viewer.lightbox | `esc closes` | rCBA7 |
| share.files_line | file names joined with ` · `, uploads first, then Maya's files | e1shsD / zWgDC |
| share.files_toggle | `Let viewers open files` | dialog |
| share.files_off | `Files stay private. People with the link see file names only.` | dialog |
| share.files_on | `Anyone with the link can open and download these <n> files, with every amount in them.` (mobile `…these <n> files, amounts included.`) | dialog |
| share.mask_forced_off | `Off while viewers can open files. Amounts inside files can't be hidden.` | dialog, under Hide |
| link.meta.private | `expires in 7 days · amounts and names hidden · files private` (mobile `amounts hidden · files private`) | viewer page |
| link.meta.open | `expires in 7 days · files open to viewers` (mobile `files open`) | viewer page |
| link.chip.private / open | `PDF · private` (with lock) / `PDF · 12 pages · open` | viewer page |
| link.sources.private / open | `private · name only` / `opens here · download` (mobile `private` / `opens · download`) | viewer Sources |
| link.note.private | `M. kept the files in this reply private. You see their names only.` · with Maya's file `M. kept the files in this reply private, including the one Maya made. You see their names only.` (mobile `M. kept these files private. You see their names only.` / `M. kept these files private, Maya's too. You see their names only.`) | viewer page |
| link.note.open | `M. lets viewers open these files.` | viewer page |
| link.maya_file.private | `CSV · 18 KB · private` (lock, no actions) | viewer page |

Names, numbers and dates in the frames are sample data. Build from live values, with the same formats:
- Rp with `.` thousands (`Rp 1.900`);
- token counts with `.` thousands;
- sizes with `,` decimals (`14,6 MB`);
- dates `14 Nov 2026`;
- elapsed time `0:14`.

## Data and events the UI depends on

Re-checked read-only on 16 Nov 2026 in `backend/crates/nomi-server/src/app.rs`, `routes/attachments.rs`,
`routes/shares.rs` and `frontend-new/BACKEND_TODO.md`.

| Need | Source (API / event / config) | Exists? |
|---|---|---|
| Per-file upload + status | `POST /api/attachments` (one streamed request per file) → `status: ready \| pending`; `GET /api/attachments/:id` | **Partial**: errors are plain strings (`413 file is too big`, `507 storage is full`) with no reason codes |
| Upload failure reason codes | `connection_lost` (client), `too_large`, `storage_full`, `server_error` | **Missing** (server codes) |
| Resumable upload | Content-Range / offset on `POST /api/attachments` | **Missing**: restart only (M1) |
| Accepted types for the G4 client check | `GET /api/attachments/limits` + accepted MIME list | **Partial**: limits give sizes and counts only; no accepted-type list and no `unsupported_type` code |
| Remove an upload | `DELETE /api/attachments/:id` | **Exists** |
| Model capability (vision) | `GET /api/llm/model-options` → `capabilities: [vision, …]` | **Exists** (images). There's no PDF / max-pages capability (not needed for P0) |
| Pre-send estimate (pages, tokens, Rp, payer) | new estimate endpoint | **Missing** |
| Plan meter (% used this month) | `GET /api/usage`, `GET /api/usage/brief` | **Exists** (unit per M4) |
| Read result per file | `ok \| locked \| no_text \| damaged \| partial` + `read` / `total` / `skipped[]` on the attachment or in the reply event | **Partial**: `status` and `details.scanned` exist; no `locked` / `damaged` / `partial` codes and no zip counts |
| Agent output files (store, list, signed download, preview) | output-file routes + reply event `file_generating {step, elapsed}` / `file_ready` / `file_failed {code}` | **Missing** |
| Save to project | `PUT /api/projects/:id/files/*path` | **Exists.** A storage limit returns `402` with a message, not a code. Map it to `storage_full` client-side, or have the server add the code. |
| Per-file delete (D1) + `deleted_at` on output files | delete route for output files | **Missing** (uploads: `DELETE /api/attachments/:id` exists) |
| Viewer page render + highlight rects | `GET /api/attachments/:id/preview`, `/content` | **Partial**: there is a preview, but no page parameter and no highlight / citation rects; auth-only, not signed |
| Citation → file + page + rows | citation payload in the reply (`file_id`, `pages[]`, `spans[]`) | **Partial.** The `sources` view (`nomi-agent-core/src/views.rs`) numbers `[n]` citations with `{title, url?, detail?, kind: web\|file}`. There is no `file_id`, `pages[]` or row spans. |
| Share files flag | `POST /api/sessions/:id/shares` with `files_open` (and server-side `mask ⇒ !files_open`) | **Missing**: shares have `mask`, `anyone` and `expires_in_days` only |
| Shared page files | `GET /api/shared/:token` → message attachments + produced files (name, type, size, `private \| open`) and signed URLs when open | **Missing**: messages only, no attachments |
| Name masking in shares | server-side | **Missing** (BACKEND_TODO line 74, M3) |

ui-engineer can build the client states now. These steps wait on a **Missing** row: G3 reason
codes, the G4 accepted list, G7, the G9 codes, the G11 output files, viewer highlights, and the
G16 files flag. Gate them behind the data, and never fake it in the UI.

## Accessibility

**Composer focus order:** attach `+` → chips (each chip is one stop, its actions inside it) →
fix line actions (G6) → field → model chip → Send.

**Focus moves:**

| When | Focus goes to |
|---|---|
| G6 warning appears | `Switch model`. The ring is the standard `$nm-focus` 2 px outer ring (XHDLj / nkhFC). |
| A chip is removed | The next chip, otherwise the field |
| `Replace file` | The picker, then the new chip |
| The model menu opens | The first model. With no capable model, `Add one in Models & providers`. `Esc` closes it and returns to `Switch model`. |
| The viewer opens | The viewer's Close button. Closing returns focus to the citation, chip or button that opened it. |
| The share dialog's toggles flip each other | Focus stays on the toggle the user changed. |

**Keyboard:**
- `↵` sends only when Send is enabled. While Send is disabled, `↵` does nothing.
- `Esc` closes the menu, viewer or lightbox.
- In the viewer, `←` / `→` (or prev / next) change the page.
- `Backspace` on a focused chip removes it.

**Mobile:** no focus ring is drawn on the phone pills (designer's choice, accepted). Touch has no
visible focus, and VoiceOver and TalkBack draw their own. With a hardware keyboard, the same
`$nm-focus` ring shows on `:focus-visible` at every width, with no extra frames.

**Live regions** (`role=status`, polite, unless marked assertive):

| Event | Announcement |
|---|---|
| Upload done | `bca-oct-2026.pdf uploaded` |
| Upload failed (assertive) | `bca-oct-2026.pdf failed, Retry available` |
| Rejected | `kasir-setup.dmg can't be read. Remove it to send.` |
| Model warning | `deepseek-v3 can't see images. Switch model or send text only.` |
| Estimate shows or changes | `Reading this costs about 96.000 tokens, Rp 1.900 on Nomi credits` |
| File generating | `Maya is making pengeluaran-sep-okt-2026.csv` (no per-second updates) |
| File ready | `pengeluaran-sep-okt-2026.csv is ready` |
| File failed (assertive) | `Maya couldn't make pengeluaran-sep-okt-2026.csv` |
| Saved | `Saved to budget-rumah` |
| Save failed (assertive) | `Couldn't save to budget-rumah, storage full` |
| Viewer | `Page 3 of 12, 9 rows highlighted` once loaded; the error text on error |
| Share toggles | The changed state of both toggles, e.g. `Files open to viewers. Hide amounts turned off.` |

**Labelling:**
- Disabled Send has `aria-disabled` and `aria-describedby` pointing at the hint.
- Callout codes and mono reasons are read as text, not hidden.

## Acceptance criteria
- [ ] **AC-G3**
  - Cutting the network at about 40% shows the failed chip within 5 s, and the other chips continue.
  - Send stays disabled, with the hint, until every chip is ready or removed.
  - `Retry` uploads again (resume or restart per M1), and `Remove` frees Send.
- [ ] **AC-G4**
  - An unsupported file never reaches `POST /api/attachments`. Network log: zero requests for it.
  - The chip and helper copy are verbatim, and the Send hint names the file.
  - A `.heic` becomes an image chip.
- [ ] **AC-G6**
  - With a non-vision model and an image chip, the warning shows before Send, Send is disabled, and focus is on `Switch model`.
  - The menu lists only vision models and counts the hidden ones.
  - With no vision model, the empty state shows with `Add one in Models & providers`, and the draft survives the trip to Settings.
  - `Send text only` sends no attachment id, and the bubble shows the sent note.
- [ ] **AC-G7**
  - Over the threshold, the estimate row shows before Send, with the payer.
  - The credits variant's meter uses the plan meter's unit and direction.
  - After the reply, the usage moved within the stated margin of the estimate.
  - Under the threshold, nothing shows. BYOK shows no Rp and no meter.
- [ ] **AC-G9**
  - Each of `locked`, `no_text`, `damaged` and `partial` produces its own callout: Error for locked and damaged, Warning for no_text and partial.
  - The other files in the same message are still answered.
  - `Read as images` shows the estimate before running (D3).
- [ ] **AC-G11**
  - The card shows the real step and elapsed time with no bar.
  - The card ends in one of `ready`, `failed` or `stopped`, never hanging.
  - `Download` works on desktop and on mobile.
  - `Save to project` shows only in a project. It ends in `saved` with `Open`, or in the error toast with `Try again`.
  - A deleted file shows the deleted card, never a broken link. `Make it again` is hidden when the sources are gone.
- [ ] **AC-G12**
  - Desktop: `[3]` opens `bca-sep-2026.pdf` at page 3 with the cited rows highlighted.
  - Mobile: Sources `[1]` opens the same page full screen.
  - The loading state shows the real step. The error state offers `Try again` and `Download`.
  - Focus returns to the trigger on close.
- [ ] **AC-G16**
  - A new share link has files private, and viewers see names and lock icons only. The shared API returns no file URL.
  - Files open and Hide are never both on, in the UI or in the API: a server request with both is rejected.
  - Maya's produced files follow the same switch, and viewers never get `Save to project` or `Make it again`.

## Open items for Mario (not design failures)
- **M1 · Resumable uploads.** The G3 copy promises a resume (`Resuming · 41%`, `picks up at 41%`), but the backend can only restart.
  - **Choice:** build resumable uploads and keep the copy, or accept a restart.
  - **With a restart:** the copy becomes `Uploading again · 0% · 0 of 5 MB` / `starts the upload again`, and the D9 / M11 captions change.
  - **Until you decide:** the build ships the restart copy unless resume exists.
- **M2 · Redacting inside files.** Masking and open files are mutually exclusive today. Server-side redaction of PDFs and CSVs would let both be on. This is a future backend investment.
- **M3 · "Hide amounts and names" over-promises.** The server doesn't mask names, and file names can carry names.
  - **The choice:** drop "and names" and "initials" from the helper, or mask names and file names when Hide is on.
  - This was already true in the existing Share flow (eemXK).
- **M4 · Unit for Nomi credits.** G7 shows an Rp price (`≈ Rp 1.900`) next to a % / token allowance. Choose Rp, %, or both. The current frames keep both.

## Non-blocking notes (no change needed for this gate)
- **N1 · Sample statement data.**
  - The CSV preview's September rows (03/09–30/09, 17 rows) don't add up to the 64 September transactions or to the `Rp 1,75 jt` food total.
  - Sources cite October rent on page 2, while the CSV dates it 28/10 (the designer's question).
  - **Ruling:** keep the frames as they are. This is illustrative data, the build uses live data, and the cited-page totals (`Rp 1.120.000` on page 3) are correct.
  - If these frames are touched again, cite October rent on a late page (e.g. `page 10`) and trim the CSV preview to rows that sit on the cited pages.
- **N2 · Reference frames.** These are pre-existing in Mario's reference frames, not in this scope:
  - TpuRc "Your provider is out of credits" says `82% left this month`, while scenario 6 says `82% … used`.
  - Light Shared link frames say `Report`, while dark ones say `Report this page` (already in step 1, v72jgW vs oGMsN).
- **N3 · One share dialog.** Build eemXK plus an optional Files line and the `Let viewers open files` row, as one dialog. e1shsD and zWgDC are structural copies for the pen only.
- **N4 · Row name.** The desktop row `Steps · UXP-001 viewer` (X4u51c / IxzLO) now also holds steps 24–26 (save and deleted). This is cosmetic.
