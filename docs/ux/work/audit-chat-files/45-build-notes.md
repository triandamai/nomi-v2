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

