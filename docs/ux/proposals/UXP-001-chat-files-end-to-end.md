---
id: UXP-001
title: Chat files end to end — upload, agent reading, agent answers with and about files
area: chat
author: ux-lead
status: accepted
accepted: 16 Nov 2026
work: audit-chat-files
flows: [Iyq9S, ORByl, FLwdb, Fpgnv, TpuRc, Nud6J, AyrHG, uuCam, b8Flw8, Yyjkb]
audited: 16 Nov 2026
---

# UXP-001 — Chat files end to end

## Why
Today's frames cover the happy path well: pick a file, see it upload, Maya reads
it and answers with page citations. What's missing is everything around it —
the upload that fails, the file Maya can't read, the model that can't see images,
the 180-page PDF that quietly costs money, and the moment Maya hands a file
*back*. Those are the moments where a non-coder decides whether Nomi is a serious
tool. They're also where unbounded cost (business) and missing backend events
(tech) hide.

## Already covered (don't redesign)
- Iyq9S · 1 — Attach menu: upload, photo, Google Drive via Tara, paste a link; limit `PDF, images, sheets, zip · max 20 MB`.
- Iyq9S · 2 — Uploading chips with per-file progress and cancel; Send waits.
- Iyq9S · 3 — Message keeps its files; Maya shows file and page she's on.
- Iyq9S · 4 — Answer with page citations back to file + page.
- Iyq9S · 5–6 — Voice record, transcribe into the composer, edit before send.
- Iyq9S · 7 — Text over 10.000 characters: send as a file instead.
- TpuRc · File too large (desktop) and FLwdb · 8 (mobile) — caught in the chip before sending.
- TpuRc — offline queue, message not sent, credits low / out, signed out.

## Gaps

### Getting files in

#### G1 · Drag and drop onto the chat · P1
- **Scenario:** user drags files from Finder/Explorer onto the chat or composer.
- **Expected:** whole chat column shows a drop target; on drop each file becomes a
  chip. Files that break a rule land as an error chip (G3/G4), the rest upload.
- **Copy:** `Drop to attach` · `PDF, images, sheets, zip · max 20 MB each`
- **States:** idle, dragging over, dropped (mixed accepted/rejected).
- **Reuse:** `Chat / Attachment / Uploading` o8o2x. New: `Chat / Drop Target`.
- **Mobile:** n/a — covered by the attach sheet; add "Share to Nomi" from the OS share sheet as P2.
- **Accept when:** dropping 3 files where 1 is a `.dmg` gives 2 uploading chips + 1 rejected chip, Send still waits for the 2.

#### G2 · Paste from clipboard · P1
- **Scenario:** user pastes a screenshot, or pastes a copied file.
- **Expected:** pasted image becomes an image chip named `Screenshot 16 Nov 2026 14.32.png`
  and uploads like any file. Pasted long text keeps the existing 10.000-char rule.
- **Reuse:** `Chat / Attachment / Image` vFPxs.
- **Accept when:** ⌘V / Ctrl V of an image in the composer creates one image chip; focus stays in the field.

#### G3 · Upload failed · P0
- **Scenario:** network drops or the server rejects the upload mid-way.
- **Expected:** that chip turns to error with the reason and two actions. Other
  chips continue. Send is disabled while any chip is failed, with a hint saying why.
- **Copy:** chip `Upload failed · connection lost` · actions `Retry` `Remove` ·
  send hint `1 file didn't upload. Retry or remove it to send.`
- **States:** failed, retrying (back to progress), removed.
- **Reuse:** o8o2x. New: `Chat / Attachment / Failed`.
- **Mobile:** same chip; actions in a long-press sheet if the chip is too narrow.
- **Tech needs:** per-file upload status events; resumable or restartable uploads; server reason codes.
- **Accept when:** killing the network at 40% shows the failed chip within 5 s, Retry resumes, Send unlocks only when all chips are ready.

#### G4 · File type not supported · P0
- **Scenario:** user picks `.exe`, `.dmg`, `.pages`, `.heic` (decide), or a zip containing only unsupported files.
- **Expected:** rejected at the chip before any upload, with what *is* accepted and a way out.
- **Copy:** `.dmg files can't be read` · sub `Maya reads PDF, images, sheets and zip.` · action `Remove`
- **Reuse:** same chip family. New: `Chat / Attachment / Rejected`.
- **Business:** blocks executables early (abuse + storage).
- **Accept when:** an unsupported file never hits the upload endpoint.

#### G5 · Too many files in one message · P2
- **Copy:** `Up to <N> files per message. Send these first, then add the rest.`
- **Tech needs:** N from config — **tech to confirm; don't hard-code in design**.
- **Accept when:** the N+1th file is rejected with that copy; the first N stay.

### Before sending

#### G6 · Model can't read this file · P0
- **Scenario:** the chosen model doesn't accept images or PDFs (some BYOK models are text-only).
- **Expected:** chip shows a warning, not an error; the composer offers the fix
  inline. Reuse the model chip menu from TpuRc step 2, filtered to models that can read it.
- **Copy:** `DeepSeek V3 can't see images` · action `Switch model` ·
  secondary `Send text only`
- **Reuse:** `Chip / Model` k2hb2, model menu from TpuRc · 2. New: `Chat / Attachment / Model Warning`.
- **Tech needs:** model capability matrix (images, PDF, max file size, max pages) from the providers config — Settings already shows a "Vision" label.
- **Accept when:** choosing a text-only model with an image attached shows the warning before Send; `Send text only` sends the message without the file and says so in the bubble.

#### G7 · Cost and provider shown before a big read · P0 (honesty rule)
- **Scenario:** attached files are large enough that reading them is expensive (threshold from config).
- **Expected:** a quiet line above Send with the estimate and who pays, before the
  user commits. Small files show nothing extra.
- **Copy:** `laporan-keuangan-2026.pdf · 184 pages · about 96.000 tokens` ·
  `Runs on Nomi credits · ≈ Rp 1.900` *(example values — tech supplies real estimate)* ·
  for BYOK `Runs on your Anthropic key`
- **Reuse:** `Meter` i5A8H.
- **Business:** prevents surprise credit burn and support tickets; keeps the "never hide cost" promise.
- **Tech needs:** pre-send estimate endpoint (page count, token estimate, price per model).
- **Accept when:** the estimate is visible before Send for any file over the threshold, and matches the cost meter after the reply within the stated margin.

### While Maya reads

#### G8 · Long read in progress · P1
- **Expected:** activity shows the real step and elapsed time, never a fake bar:
  `Reading laporan-keuangan-2026.pdf · page 34 of 184 · 0:41`. `Stop` keeps what was read.
- **Reuse:** `Activity / Live Step` hPp8O, `Chat / Stopped` bTCPw.
- **Accept when:** Stop during a read leaves a reply that states which pages were read.

#### G9 · File can't be read · P0
Four cases, one pattern: Maya says which file, what's wrong, and the next step.
Never a silent skip.
| Case | Copy (Maya's reply line) | Action |
|---|---|---|
| Password-protected PDF | `bca-oct-2026.pdf is locked with a password. Upload an unlocked copy and I'll read it.` | `Replace file` |
| Scanned PDF, no text | `This PDF is scanned images, so there's no text to read. I can read the pages as images · about 41.000 tokens.` | `Read as images` `Skip` |
| Corrupted / empty | `invoice-mei.xlsx looks damaged — I couldn't open it.` | `Replace file` |
| Zip with mixed content | `I read 6 of 9 files in assets.zip. Skipped: 3 .psd files.` | `See skipped` |
- **Reuse:** `Callout / Error` mhAYP inside the agent message; `Chat / Quick Replies` K4uh0l for actions.
- **Tech needs:** extraction pipeline returns a per-file result: `ok | locked | no_text | damaged | partial` + counts.
- **Accept when:** each case produces its own message; other files in the same message are still answered.

#### G10 · Read only part of a file · P1
- **Scenario:** file is bigger than what fits the model's context.
- **Expected:** Maya says which part she read and offers the rest — consistent with the context-limit flow in TpuRc.
- **Copy:** `I read pages 1–120 of 184. Ask about a section, or let me continue from page 121.`
- **Accept when:** the answer never implies it covered pages it didn't read.

### Maya's answer with and about files

#### G11 · Maya hands back a file · P0
- **Scenario:** the answer is a file — cleaned CSV, a PDF summary, a generated image, a zip.
- **Expected:** a file card inside the agent message: name, type, size, what it contains,
  `Download`, `Preview`, and `Save to project` when a project is open. Files are
  listed as produced output, not mixed with the user's uploads.
- **Copy:** `pengeluaran-sep-okt-2026.csv · CSV · 18 KB · 2 months, 214 rows` ·
  actions `Download` `Preview` `Save to project`
- **New:** `Chat / Agent File`, `Chat / Agent File / Generating` (shows the real step), `Chat / Agent File / Expired`.
- **Tech needs:** output file storage, signed download URLs, expiry policy.
- **Accept when:** download works on desktop and mobile; an expired file shows the expired state, never a broken link.

#### G12 · Preview a file and jump to a citation · P0
- **Scenario:** user clicks an attachment chip in a sent message, a citation number, or Maya's file card.
- **Expected:** desktop opens a side panel viewer at the cited page with the cited
  passage highlighted; mobile opens a full-screen sheet. Images open in a lightbox.
  Sheets show the first rows + "Download to see all".
- **Reuse:** `Chat / Citation` C6xkT, `Chat / Sources` mrOeg, `Workspace / Sheet` DhW7n pattern for the panel.
- **New:** `Chat / File Viewer` (panel), `Mobile / File Viewer` (sheet).
- **Accept when:** clicking citation `[3]` opens bca-sep-2026.pdf at page 3 with the passage highlighted.

### After

#### G13 · Files in this chat · P1
- **Expected:** chat menu → `Files` lists every upload and every file Maya produced,
  with who added it, size, date, and `Delete`. Deleting removes it from storage and
  turns its citations into `File deleted` (no broken jumps).
- **Copy:** empty state `No files in this chat yet. Attach with + or drop them here.` ·
  delete confirm `Delete bca-sep-2026.pdf? Maya's earlier answers keep their text, but citations to it won't open.`
- **Reuse:** `Chat / Chat Menu` r4uFeJ, `File Row` CCpOg, `Dialog` qXVeZ.
- **Business:** retention promise must match the privacy policy — see decision D1.
- **Accept when:** a deleted file's citations render the deleted state.

#### G14 · Edit or regenerate a message that has files · P1
- **Expected:** editing keeps the chips, each removable, and lets you add more.
  Regenerate re-reads the same files and shows the new cost estimate if G7 applies.
- **Reuse:** `Chat / User / Editing` jqbjk with `Chat / Attachment / Ready` iAYlM.

#### G15 · Offline with files · P1
- **Expected:** files queue with the message; chips show `Waiting for connection`.
  On reconnect uploads start, then the message sends — matching TpuRc offline steps.
- **Reuse:** `Chat / User / Queued` JCfti, `Chat / Banner / Offline` i9fRJB.

#### G16 · Files in a shared chat link · P0 (privacy)
- **Expected:** viewers see the file name and type only; content, preview and
  download are off by default. The share dialog says so and offers an opt-in toggle.
- **Copy:** dialog `Files stay private. People with the link see file names only.` ·
  toggle `Let viewers open files`
- **Reuse:** `Chat / Share Dialog` eemXK, `Toggle` D1S92S / CAJtO.
- **Business:** a bank statement leaking through a share link would end trust in the product.

#### G17 · Access and keyboard · P1
- **Expected:** `⌘U` / `Ctrl U` opens the file picker; chips are focusable with
  Remove on `Backspace`; upload progress and failures are announced to screen readers
  (`bca-oct-2026.pdf uploaded`, `bca-oct-2026.pdf failed, Retry available`);
  after removing a chip, focus moves to the next chip or back to the field.
- **Accept when:** the whole attach → send → open citation path works without a mouse.

## Scope per frame set
Every gap: desktop dark + light (extend Iyq9S / ORByl, add steps after 7),
mobile dark + light (extend FLwdb / Fpgnv, after step 8). File-error gaps
(G3, G4, G9) also get a step in Errors & recovery (TpuRc / Nud6J / AyrHG / uuCam).
G16 extends Shared link (b8Flw8 / Wgdu2 / Yyjkb / e0FI49).

## Three-lens summary
- **Product:** the happy path is strong; the failure and return paths are missing.
  P0s are the ones where the user is otherwise stuck or misled.
- **Business:** G7 (cost before read), G16 (shared-link privacy) and G13 (retention)
  protect trust and limit support load. G4 blocks abuse early.
- **Tech:** backend must expose per-file upload status, rejection reasons, a model
  capability matrix, a pre-send estimate, extraction results per file, output file
  storage with signed URLs, and a retention job. ui-engineer can't start G3, G6, G7,
  G9, G11 until those exist — flag them in the contract's "Exists?" column.

## Decisions (Mario, 16 Nov 2026)
- **D1 · Retention — decided:** keep files until the chat is deleted, with per-file
  delete (G13). Privacy copy and the Files list say exactly that.
- **D2 · HEIC and other phone formats — decided:** convert on upload. A `.heic`
  photo becomes an image chip like any other; it never lands in G4's rejected state.
- **D3 · OCR cost — decided:** estimate first, then the usual payer. `Read as images`
  (G9, scanned PDF) shows the G7 estimate line and runs on Nomi credits or the
  user's own key, same as any read.

<details><summary>Original open decisions</summary>

### Open decisions for Mario (as proposed)
- **D1 · Retention.** How long are uploads kept? Options: keep until the chat is
  deleted / 30 days / per plan. ux-lead recommends *until the chat is deleted*, with
  per-file delete — simplest to explain, matches "your work belongs to you".
- **D2 · HEIC and other phone formats.** Convert on upload or reject? ux-lead
  recommends convert silently; iPhone users shouldn't hit G4 for a normal photo.
- **D3 · OCR cost.** Who pays for "Read as images" on scanned PDFs? ux-lead
  recommends same rule as any read: estimate first, then the usual payer.

</details>
