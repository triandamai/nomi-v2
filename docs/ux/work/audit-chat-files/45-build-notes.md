---
work: audit-chat-files
contract: 40-ux-contract.md (status: verified)
engineer: ui-engineer
branch: feat/frontend-new-v2-chat-files
---

# Build notes — audit-chat-files

## Starting point

`frontend-new` had no attachments in a chat at all: ChatView drew the composer without its `+`, and a
user message carrying files showed their raw reference tags. Every P0 gap that the backend can reach
needs files in the composer first, so the build adds the base path the reference steps (D1–D7,
M1–M8) already draw: pick → upload with real bytes → ready chip → send with reference tags → the
sent message shows its files. It uses the backend as the old app does (`POST /attachments`, streamed
by `ws-proxy/upload-proxy.js` in production, `X-File-Name`; `DELETE /api/attachments/:id`; the
reference tags in the message text).

Twin rules: `ds/chat`, `ds/conversation`, `ds/mobile` and the shared overlays/actions are the old
app's twins, so `AttachmentChip`, `MessageUser`, `SheetHeader`, `ShareDialog` and `IconButton` are
composed, never changed. New components live in `src/lib/ds/thread/` (not twinned).

## Plan: scenario → buildable now / waits → components, state, API, tests

| # | Scenario | Now? | Why / what it needs | Components · state · API | Tests |
|---|---|---|---|---|---|
| 1 | Upload fails mid-way | **Built** | The client knows a dropped connection (XHR `error`/`abort` from the network) and the HTTP status. Reasons by status: network → `connection lost`, 507 → `storage full`, 413 → `too large`, other 4xx/5xx → `server error`. The server's own codes wait (BACKEND_TODO). | `thread/ComposerFile` (failed), `chat/uploads.svelte.ts` (per-chip machine), `chat/attachments.ts` (hint, reasons), `ChatComposer`/`PhoneComposer` `blocked` | unit (reasons, hint), component (chip, ↵ blocked, aria-disabled/describedby), backend e2e (route aborted → failed chip, hint, ↵ does nothing) |
| 2 | Retry or remove (mobile sheet) | **Built** | — | `BottomSheet` + `header` snippet, `thread/FileSheetHeader`, `mobile/SheetRow` | component (header), backend e2e phone (⋯ → sheet → Retry) |
| 3 | Retrying | **Built, restart copy (M1)** | No resumable upload: `Uploading again · 0% · 0 of 5 MB`, sheet meta `starts the upload again`. | uploads machine `failed → uploading` (new request) | unit, backend e2e (Retry → ready) |
| 4 | Removed | **Built** | `Remove` on a failed chip drops it (nothing was stored). Line `<name> removed · N file(s) ready`. | ComposerFile, composer helper/notice | backend e2e |
| 5 | Unsupported file picked | **Partly** | No accepted-type list, so no client pre-check (AC "zero requests" can't hold). The server refuses programs with 415: that chip becomes `Rejected` (`.exe files can't be read`), Send hint `Remove <name> to send.`. The helper `Maya reads PDF, images, sheets and zip.` is left out: it states a list the backend doesn't have (it reads far more). | ComposerFile (rejected) | unit, component, backend e2e (an `MZ` file → rejected chip, hint) |
| 6 | Mixed pick | **Partly** | Each file uploads on its own, so one refused file never stops the others. `.heic` is an image (the backend sniffs it as one). The `.pages` rejection waits on the accepted list. | uploads machine | backend e2e (two files, one refused) |
| 7 | Model warning | **Built** | `GET /api/llm/model-options` gives `capabilities` (vision). | `thread/ComposerFile` (warning), `thread/ModelFix`, focus to Switch model | unit (vision helpers), component (focus), backend e2e |
| 8 | Switch model | **Built** | Same. Rows: model (mono) · `<group> · reads images`; hidden count; Models & providers. | `thread/VisionModelMenu` (desktop popover / phone sheet) | component (keyboard, Esc back to Switch model), backend e2e |
| 9 | No capable model | **Built** | Same. The draft (text + ready files) is kept in `sessionStorage` per chat, so it survives the trip to Settings. | VisionModelMenu (empty), `chat/draft.ts` | unit (draft), backend e2e (Settings and back) |
| 10 | Send text only | **Built, note on this visit** | The message goes without the image (the upload is deleted). The bubble note is kept for this visit only: nothing in the backend stores "sent without" (BACKEND_TODO). Maya's own line about it is the crew's reply, not UI. | `thread/FileNote` | backend e2e (no attachment tag in the sent message) |
| 11–12 | Cost before a big read | **Waits** | No pre-send estimate (pages, tokens, Rp, payer). | — | — |
| 13–17 | File can't be read (locked, scanned, damaged, zip) | **Waits** | No per-file read result codes; a scanned PDF is read by the files model today (`needs_model`), so "Couldn't read" would be false. | — | — |
| 18–24 | Maya hands back a file | **Waits** | No output-file storage, events, download, delete. | — | — |
| 25 | Open a citation | **Waits** | Sources carry no `file_id`, pages or row spans. | — | — |
| 26 | Viewer loading | **Built (page 1)** | `GET /api/attachments/:id` gives `details.pages` for a PDF: `Opening page 1 of 12 · 0:02`, real elapsed time. | `thread/FileViewer` (loading) | component, backend e2e |
| 27 | Viewer error | **Built** | A 30 s timeout is the client's own (`preview_timeout · 30 s`); a refused file says its status (`preview_failed · 404`). `Try again`, `Download`. | FileViewer (error) | component |
| 28 | Photo | **Built** | `/attachments/:id/content` serves images inline. Zoom 50–200%, Download, Close, `esc closes`. The footer says which photo of how many; "read by Maya for …" has nothing behind it. | `thread/PhotoViewer` | component (Esc, zoom), backend e2e |
| 29 | Preview Maya's file | **Waits** | No output files. | — | — |
| 30 | Open a sent chip | **Built** | Desktop: side panel 440; phone: full screen. PDF pages are drawn by the browser's own PDF viewer (`#page=1`); cite bar and footer hidden. Other kinds download. | FileViewer, `thread/SentFiles` | backend e2e |
| 31 | Share a reply that used files | **Waits** | The dialog's row is the `Let viewers open files` switch, and shares have no `files_open`. Without it the row would be a switch drawn dead. | — | — |
| 32–33, 35–36 | Files open to viewers, Maya's file | **Waits** | No `files_open`, no shared-page files, no output files. | — | — |
| 34 | Viewer, files private | **Built** | Today a shared link never gives a file URL: the page lists each file from the message's reference tags with a lock and `PDF · private`, the meta adds `files private`, and the note under the reply names the owner as the API gives them (`shared_by`, initials when masked). Sources have no file rows (no citation data). | SentFiles (locked), FileNote | backend e2e (shared link shows lock chips, no file URL) |

Access (all built parts): composer focus order `+` → chips → fix line → field → model → Send;
removing a chip moves focus to the next chip or the field; Backspace on a focused chip removes it;
the G6 warning moves focus to `Switch model`; the menu focuses its first model (or `Add one…`), Esc
returns to `Switch model`; the viewer focuses Close and gives focus back to the chip that opened it.
Live regions: upload done / failed (assertive) / rejected / model warning / viewer page.

## What was built

Components (all `frontend-new/src/lib/ds/thread/`, presentational, each with its `ComponentDoc` in
`src/lib/docs/entries/thread.ts` and live examples in `src/lib/docs/examples/<slug>/`):

| Component | Pen | Example(s) |
|---|---|---|
| `ComposerFile` | BD98f Failed, xpYoV Rejected, rrjYM Model Warning (+ AttachmentChip / AttachmentImage for uploading and ready) | `composer-file/01-what-became-of-each`, `02-phone` |
| `ModelFix` | kthOB (D11), mobile Notice (M13) | `model-fix/01-desktop-and-phone` |
| `VisionModelMenu` | PZvaP, er95B; mobile sheets M14–M15 | `vision-model-menu/01-can-see-images`, `02-no-match` |
| `FileSheetHeader` | BhXcx Sheet Header r9vDW | `file-sheet-header/01-failed-upload` |
| `SentFiles` | ENNIK files; Link D5 private files (RCgbo · YWmux) | `sent-files/01-in-the-chat`, `02-shared-link` |
| `FileNote` | JVUHc "Left Out", yhuBX helper, RCgbo "Files Note" | `file-note/01-notes` |
| `FileViewer` | G534E, HVM9y, K1phVf; GbEIf, b3lX47, nQ4Iq | `file-viewer/01-opening`, `02-error` |
| `PhotoViewer` | rCBA7, LiYjQ | `photo-viewer/01-lightbox` |

Changed (not twins): `thread/ChatComposer` and `thread/PhoneComposer` (`files`, `notice`,
`attached`, `blocked`, `describedBy`; example `chat-composer/04-file-did-not-upload`),
`overlays/BottomSheet` (`header` snippet). App: `lib/chat/attachments.ts`, `uploads.svelte.ts`,
`draft.ts`, `ChatView.svelte`, the `/attachments` routes, and the shared page.

## How to reach each built scenario

Run the app against the backend (`npm run dev`, or `node server.js` after `npm run build`), sign in,
and open a chat (`/chat/<id>`, New chat from the sidebar). The composer's `+` opens the file picker.
Desktop is ≥ 768 wide; the phone screens show below 768 (or with a phone user agent). Dark and
light follow the account's theme or the system's.

| # | Scenario | How |
|---|---|---|
| 1 | Upload fails mid-way | Pick two PDFs; cut the network for one (DevTools › Network › Offline as it uploads, or block the `/attachments` request). The failed chip shows `Upload failed · connection lost` with Retry · Remove; Send is off with `1 file didn't upload…`; Enter does nothing. A full storage quota gives `storage full`. |
| 2 | Mobile sheet | Same, on a phone: the chip's ⋯ opens the sheet (file name, reason, Retry `starts the upload again`, Remove `stays on your phone`). |
| 3 | Retrying | Retry: the chip shows `Uploading again · 0% · 0 of … ` and real bytes, then ready. |
| 4 | Removed | Remove on the failed chip: `<name> removed · N files ready`; Send is free. |
| 5–6 | Refused file | Pick a program (a Windows `.exe`): the chip shows `.exe files can't be read`, Send hint `Remove kasir-setup.exe to send.`; Backspace on the focused chip removes it. Other files in the same pick carry on. |
| 7 | Model warning | Use a chat whose model has no `vision` capability (the e2e fake model, or any text-only model) and attach a PNG/JPG: the chip names the model, the fix line appears, focus lands on Switch model, Send is off. |
| 8 | Switch model | Have a model that can see images (Admin › Models with Images, or a key for one): Switch model lists only those, `<model> hidden · can't see images`; picking one clears the warning. |
| 9 | No capable model | With none (hide them in Settings › Models & providers): `No model here can see images`, `Add one in Models & providers` (→ /settings/models), Send text only. Go there and press Back: text and picture are still in the composer. |
| 10 | Send text only | Click it: the message goes without the picture (deleted, never sent); `Sent without <name> · <model> can't see images` under the bubble (this visit only). |
| 26–27, 30 | Viewer | Send a PDF, then click its card over the bubble: desktop opens the 440 panel (`Opening page 1 of N · 0:0s` while it loads, focus on Close, Esc closes and returns focus); phone opens full screen (swipe down closes). If it doesn't load in 30 s: `The preview didn't load`, `preview_timeout · 30 s`, Try again, Download. |
| 28 | Photo | Send a picture with a model that can see it, then click its photo: the lightbox with zoom, Download, `1 of 1 photo in this message`, `esc closes`. |
| 34 | Shared link, files private | Share the whole chat (top bar › Share) after sending files, open the link signed out: each file is locked, `PDF · private`; the meta adds `files private`; under the reply, `<who shared> kept the files in this reply private. You see their names only.` No address of a file is in the page. A shared single reply carries no user message, so it has no files to name. |

The real-backend e2e (`e2e/backend/chat-files.e2e.ts`) walks 1–10, 26–28, 30 and 34.

## Left out, and why (never faked)

- **G3** server reason codes (the chip reads the HTTP status) and resuming (M1: restart copy).
- **G4** the type check before upload and its helper line: no accepted-type list; only the server's
  own refusal (415) is shown, after the upload. AC "zero requests for it" can't hold yet.
- **G7** the estimate row and meter: no estimate endpoint.
- **G9** every read-result callout: no locked / no_text / damaged / partial codes; a scanned PDF is
  read by the files model today.
- **G11** every agent-file state: no output files, events, download, delete or remake.
- **G12** citations (#25) and previewing Maya's file (#29): no `file_id` / pages / spans on
  citations, no output files. PDFs open in the browser's own reader at page 1 (no page images to
  draw highlights on).
- **G16** the share dialog's files row and `Let viewers open files` (#31–33), files open on a link
  (#35–36): no `files_open` on shares and no files on the shared page.
- **M3** masking file names. Entries for all of these: `frontend-new/BACKEND_TODO.md`, "Chat files".

## Differences from the contract (listed in AGENTS.md §9)

- Tab order follows the layout (chips → field → fix line → attach → model → Send); the contract puts
  attach first, but attach is drawn last in the toolbar.
- The "Sent without…" note sits under the bubble (MessageUser is a twin and has no slot), and lasts
  for the visit only.
- A message with files can't be edited in place (G14 is P1; editing it would show its tags).
- The warning chip's icon is 12, not the pen's 11 (not a size Icon takes).
- The removed line counts the picture waiting on the fix line as one of the files ready (it uploaded).
- Sent PDF cards say `PDF · 4,2 MB` (the size from the message), not `PDF · 12 pages`: pages need a
  call per file.

## Question for ux-lead

1. **Focus order (Accessibility).** The contract's order starts with attach `+`, but attach sits in
   the toolbar under the field, so DOM and visual order put it after the chips and the field. Keep the
   visual order (built), or move attach before the chips in the tab order only?
2. **Hint priority.** With a failed upload and a picture the model can't see in the same composer,
   the failed-upload hint wins and the fix line waits until the failure is retried or removed. Is that
   the intended order?
