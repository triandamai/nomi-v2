---
name: references-patterns
description: "Interaction and UI patterns learned from beUI, rare-ui and Beautiful UI, grouped by job, each in your own words: what it does, when it helps a small-business app, how to draw its states on a static canvas, and a one-line motion note for Koda."
license: "Ideas from beUI (MIT, (c) 2026 Saurabh Chauhan), Beautiful UI (MIT, (c) 2026 Shane Levine) and rare-ui (reference only)"
source: "https://beui.dev, https://www.beautifului.dev, https://rareui.com"
---
# References: Interaction Patterns

Interaction and UI patterns learned from beUI, rare-ui and Beautiful UI, grouped by job, each in your own words: what it does, when it helps a small-business app, how to draw its states on a static canvas, and a one-line motion note for Koda.

These are ideas, not components to install. Build each from shadcn parts and your own tokens (see `references-rules`). Each entry ends with the library you learned it from. "Draw" lists the frames to put side by side; "Motion" is the note you hand Koda.

## Feedback and status

**Confirm in place.** A delete button that turns into its own confirmation ("Yakin hapus?" with a cancel) instead of opening a dialog. Helps on dense lists (products, orders) where a dialog per row is heavy. Draw: resting button, armed state with the question and two small actions, done (row removed or an undo toast). Motion: the button widens to show the question, then shrinks back. Keep Alert Dialog for anything that loses money or data. *(rare-ui)*

**Toast with a live status icon.** A toast whose icon changes as the action resolves: spinner, then a check or a cross. Fits "Mengirim invoice…" turning into "Invoice terkirim". Draw: loading, success, error versions of one Sonner toast. Motion: icon cross-fades between states; toast slides in from its edge. *(rare-ui, beUI)*

**Stacked toasts.** Several toasts collapse into a short stack and fan out when hovered or tapped. Useful when bulk actions fire many messages. Draw: collapsed stack (front toast plus two thin edges behind) and expanded list. Motion: cards spread on hover, swipe to dismiss. *(beUI)*

**Notification stack.** A summary card ("3 pesanan baru") that opens into the separate cards. Draw: summary and expanded. Motion: cards spring out of the summary. *(beUI)*

**Field feedback that waits.** Validate a field only after the person leaves it, keep space reserved under the field so the layout does not jump, and show a small check when the value is valid. Draw: untouched, typing (no error yet), error after blur, valid with check. Motion: a short shake on error, check draws on. *(beUI)*

**Inline AI status line.** One line that names what the assistant is doing right now: thinking, looking something up (with the site it is reading), writing, done. Fits a "Tanya Sora" helper or a product-description generator. Draw: one frame per phase, each with a small icon and label. Motion: label swaps with a short fade; a shimmer passes over the text while busy. *(rare-ui, Beautiful UI, beUI)*

## Input

**OTP with a travelling caret.** Code slots where the focus ring moves from slot to slot and each digit settles in as it is typed; an error shakes the row, success draws a check. Fits WhatsApp or SMS login. Draw: empty with caret on slot 1, half filled, complete, error. Motion: ring glides to the next slot; digits roll in. *(rare-ui, beUI)*

**Password with a strength meter.** An eye toggle to show or hide, and a segmented bar under the field that fills and changes colour as the password gets stronger. Draw: hidden, shown, weak, medium, strong (bar segments on `$color-destructive`, a warning token, then a success token). Motion: segments fill left to right. *(rare-ui, beUI)*

**Duration or quantity picker.** Two linked stepper wheels (hours and minutes, or a quantity with fixed bounds) whose size stays put when the number hits its limit. Fits booking length for a salon or a rental. Draw: default, at minimum (minus disabled), at maximum. Motion: digits roll up or down on change. *(rare-ui, beUI)*

**Choice ring that glides.** In a swatch or option row, the selection ring travels to the new choice instead of jumping. Fits colour or size choice on a product page. Draw: two frames with the ring on different options. Motion: ring slides along a short arc. *(beUI)*

**Theme picker as tiny previews.** Each theme option is a miniature of the app in that theme, with a ring on the chosen one. Fits a "Tampilan" setting or a shop template picker. Draw: three thumbnails, one selected. Motion: ring moves; app repaints. *(rare-ui)*

**Prompt box with intent.** A composer that grows with the text, holds attachments, a model or mode picker, and turns its send button into a stop button while working. One reference varies effort by how long send is held; for small businesses prefer a visible choice. Draw: empty with placeholder, typing with attachment chip, working (stop button), disabled. Motion: send morphs to stop and back. *(rare-ui, beUI, Beautiful UI)*

**Selection actions.** Highlight a passage and a small bar offers "Perbaiki, Persingkat, Ubah nada". Fits editing a product description or a promo caption. Draw: text with a selection highlight and the floating bar above it. Motion: bar fades up near the selection. *(Beautiful UI)*

## Navigation

**Active indicator that travels.** In a sidebar, tab list or page nav, the active marker (a dot, pill or rail) moves to the new item instead of reappearing. Draw: two frames with different active items; the marker is one shape. Motion: marker springs to the new item. *(rare-ui, beUI)*

**Reading-position table of contents.** A slim rail beside a long page that marks the section being read, sometimes collapsing into a progress pill that opens into a section list. Fits a long menu, price list or terms page. Draw: rail with current section highlighted; pill closed and opened. Motion: marker follows scroll. *(rare-ui)*

**Bottom drawer with stacked views.** One Drawer whose content changes between steps (choose item, choose variant, confirm) while the panel height adjusts. Fits checkout on phones. Draw: each step as its own drawer frame with the same header position. Motion: height morphs between steps; content cross-fades. *(rare-ui, beUI)*

**Collapsed breadcrumb.** Long trails fold their middle into an ellipsis that opens a menu. Draw: full trail and folded trail. Motion: items slide in. *(beUI)*

**Sidebar that becomes a sheet.** A rail on desktop, a sheet with the same groups on phones, with a collapse to icons. Draw: expanded, icon-only, mobile sheet. Motion: width animates; sheet slides from the left. *(beUI, Beautiful UI)*

**Swipe row actions.** Swipe a list row to reveal "Arsip" or "Hapus". Fits phone order lists. Draw: resting row and row swiped with the action buttons showing; always keep a visible alternative (a "..." menu). Motion: row follows the finger, snaps open or closed. *(beUI)*

## AI and agent UI

**Approval card.** Before the agent acts, a card asks a question with a few options, a custom answer, Skip and Continue, and a step counter when there are several questions. Fits "Kirim pengingat ke 12 pelanggan?" Draw: question with options, one selected, last step. Motion: steps slide sideways. *(Beautiful UI, beUI)*

**Tool permission card.** Allow once, always allow, or deny for an action the agent wants to take. Draw: pending, allowed (collapsed to one line), denied. Motion: card folds to a summary line. *(beUI)*

**Task list with live status.** Numbered steps of a plan, each with a status mark (waiting, running, done, failed) and a completion count; done items can strike through and move down. Draw: mixed statuses in one frame plus the all-done frame. Motion: status marks morph; finished rows slide down. *(Beautiful UI, beUI, rare-ui)*

**Collapsible reasoning and tool trace.** A "Berpikir…" line that expands into the steps, searches and tool calls behind an answer; tool calls shown as compact chips. Draw: collapsed label, expanded trace. Motion: height expands; line shimmers while active. *(Beautiful UI, beUI)*

**Streaming answer with sources.** Text appears progressively with a quiet caret, then gains source chips, inline citation numbers and a reply bar (copy, retry, follow-ups). Draw: mid-stream (half paragraph, caret), complete with sources and actions. Motion: words fade into focus as they arrive. *(rare-ui, Beautiful UI, beUI)*

**Recommendation with confidence.** A suggestion card showing a confidence level, alternatives and Accept. Fits "Harga yang disarankan: Rp 35.000". Draw: high and low confidence versions. Motion: none needed. *(Beautiful UI)*

**Proposed changes table.** A table where AI edits are marked per cell (old value struck, new value highlighted) to accept or reject. Fits bulk price or stock updates. Draw: table with changed cells, then accepted state. Motion: changes reveal row by row. *(Beautiful UI, beUI)*

**Image generation states.** Queued, coarse preview, refined, final, in the same frame size so nothing jumps. Fits product photo backgrounds. Draw: those four frames. Motion: grid of tiles resolves into the picture. *(rare-ui, beUI)*

## Data and charts

**Number that rolls.** A total or count that rolls digit by digit to its new value. Fits "Omzet hari ini Rp 2.450.000" on a dashboard. Draw: final value only, note the effect. Motion: digits roll like an odometer. *(rare-ui, beUI)*

**Filter chips with counts.** Status chips (Semua 24, Diproses 5, Selesai 19) above a table that reorders live. Draw: two filter states. Motion: rows reorder. *(Beautiful UI)*

**Activity calendar.** A heat grid of days with a legend and per-day tooltip; a summary panel can slide over it. Fits orders per day or class attendance. Draw: grid with legend, one tooltip open. Motion: summary panel slides up. *(rare-ui, beUI)*

**Sparkline with change badge.** A tiny trend line next to a number and a "+12%" Badge. Fits stat cards. Draw: rising and falling versions. Motion: line draws on. *(beUI)*

**Insight cards.** Paged cards of short findings with a small chart each. Fits a weekly sales summary. Draw: one card with pager. Motion: pages slide. *(Beautiful UI)*

## Delight (use sparingly)

**Reactions that float up.** A tapback bar of emoji; the chosen one floats up from the button. Fits reviews or community posts. *(rare-ui)*

**Step player.** A stepped progress track where the active step stretches into a filling bar, with play and pause. Fits onboarding stories or a recipe walkthrough. Draw: steps with one active and half filled. *(rare-ui)*

**FAQ as a chat thread.** Questions and answers drawn as message bubbles. Fits a friendly brand's FAQ. Draw: two Q&A pairs as Bubbles. Motion: answers appear like replies. *(rare-ui)*

**Before and after slider.** A draggable divider over two photos. Fits renovations, salons, photo editing services. Draw: divider at the middle with a handle. Motion: one hint sweep on first view. *(beUI)*

Keep delight to one moment per screen, never on checkout or payment, and always with a reduced-motion fallback.

Sources: beUI (MIT, Copyright (c) 2026 Saurabh Chauhan), rare-ui (MIT + Commons Clause + Attribution, Copyright (c) 2026 Swami Malode; ideas only), Beautiful UI (MIT, Copyright (c) 2026 Shane Levine); all described in our own words, nothing copied.
