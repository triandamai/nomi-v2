---
name: taste-app-screens
description: "How Taste's principles apply to app screens it does not cover itself (dashboards, lists, forms, checkout, settings, empty states): the same system, edges, type, colour and copy, without the one-section-per-screen rule."
license: "MIT, Copyright (c) 2026 Leonxlnx"
source: "https://github.com/Leonxlnx/taste-skill (2.5.1)"
---
# Taste: app screens

How Taste's principles apply to app screens it does not cover itself (dashboards, lists, forms, checkout, settings, empty states): the same system, edges, type, colour and copy, without the one-section-per-screen rule.

Taste 2.5 is written for company sites. Where it says nothing about app screens, these are Sora's applications of its rules; where Taste does speak (forms, the app window, UI fragments, tap size), its numbers are used as given.

## What carries over, what does not

- **Carries over:** one system (one button pair, one card style, one radius ladder, one link look, one heading scale), the 12-column grid and equal edges, the spacing scale, the type sizes and weights, 90-95% neutral colour with one accent, the contrast numbers, real icons, real data, plain copy, nothing under 13px, 44px tap targets, two boxes deep at most, never violet.
- **Does not apply:** one section per screen, hero compositions, moments, photo counts. An app screen is one frame per state at 1440x900 (or fit_content when it scrolls), 768x1024 and 390x844.

## The shell

- **Top bar or sidebar:** opaque fill, one 1px hairline in `$color-border`, no blur, shadow or floating pill; the same on every screen. Sidebar items one or two words, 4-8 of them, the current one marked by a 2px bar or a `$color-accent` row, never by a coloured icon set.
- **Phone:** a bottom bar of 4-5 items at 56+ tall, or the action bar (one 342x48 primary, or two 165x48). Content 342 wide at x 24. Every tap target 44 tall and 8+ apart.
- **Page head:** the screen title as the h1 on the left edge (28-32 on desktop app screens, 24 on a phone; a site's 72px hero size is for marketing pages), one grey line under it only when it adds a fact, the one primary action on the right edge.

## Dashboards and lists

- **Show the product's own subject, not a generic dashboard:** the orders, bookings, invoices or stock the person really works on, as in Taste's app window: a list beside the open item (5/12 + 7/12 at 1440; the list alone or the item alone on a phone).
- **Numbers are real data with units** ("Rp 4.250.000 bulan ini", "12 pesanan menunggu"), one size per kind of figure, never larger than the screen title. A few figures that matter, each beside what it proves; no strip of decorative KPIs, no sparkline without an axis meaning.
- **Rows:** 44 tall (64 with a second line), closed by a 1px rule inside the content edges; name 16/500 left, value right in tabular figures, meta 14 in `$color-muted-foreground`. Status chips in ink tints; the accent marks only the row being worked on or one urgent state, once per view; never a row of coloured dots.
- **Tables:** column heads 13/500 sentence case, no tracked capitals; 48 rows; on a phone each record becomes its own block of rows, never a sideways scroll.
- **Charts:** still, one accent series plus ink tints, axis labels 13px, a heading that says what it shows.
- **Sample data** fits the person's town and generation (Siti Rahayu, Budi Santoso, Jl. Malioboro, Rp 249.000, 26 Jul 2026), uneven where real; never Lorem ipsum, never the default cast (Priya, Okafor, Acme).

## Forms (Taste's recipe)

- Fields 48 tall, label 14px above each field (always visible; a placeholder is never the label), field text 16px, at most six fields a form (split longer flows into steps), one column under 640.
- Labels say what goes in ("Tanggal datang", "Jumlah tamu", "Email"), never a question.
- Errors under their own field, naming the field and what to type ("Tulis alamat email dengan @"), shown on submit; the first failed field focused; never only red, never only at the top. Draw one error state frame.
- One status line under the button ("Mengirim…", then the result). Success only after a real send; never a modal "Terima kasih!".
- The button names the action, word for word the main action ("Kirim permintaan booking"); a line under the form says when the reply comes.
- Focus: a 2px ring in `$color-ring`, 2px off the control. Inputs stroke `$color-input`.

## Checkout and payment

- The plain zone: facts only, no humour, no hype. The total in one place, one wording, with its unit ("Total Rp 249.000"), and the button carries it ("Bayar Rp 249.000").
- Each fee priced literally ("Ongkir Rp 15.000"); a risk reversal beside the button only when true ("Batal gratis sampai H-2").
- Payment methods by their real logos in full colour (QRIS, GoPay, OVO, DANA, BCA), as tiles with notes when the exact mark is not available (`taste-cards-icons`).
- One filled primary; the secondary action a plain link ("Kembali ke keranjang").

## Empty, loading, error, 404

- Every list and dashboard gets an empty-state frame: one line of what will appear and when ("Belum ada pesanan. Pesanan pertama muncul di sini begitu pelanggan membayar."), one action. This is where the register's voice is welcome.
- Loading: the layout at rest with its frames, never a spinner alone in an empty screen.
- 404: one plain line and the way back.

## App screens inside a marketing page

When a site shows the product's UI (an app window, a feature card's panel), draw it at true size with real data: text 13, meta 12, never under 11, weights 400/500, cut by its frame's edge by 24+ (`taste-cards-icons`).

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
