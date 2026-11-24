---
name: templates-documents
description: "How you draw documents on the canvas (invoice, kuitansi, price list, laporan, e-guide and katalog, newsletter email, CV, articles, editorial one-pagers): page sizes, sections in order, type and colour, and frames. Read it before any page that is not a website screen."
license: "Apache-2.0, Copyright 2026 Open Design contributors (open-design; html-anything templates, Apache-2.0); parchment doc inspired by tw93/kami, MIT; research brief layout from last30days, MIT, Copyright (c) 2026 Matt Van Horn; DCF and social research workflows credited to virattt/dexter (no text used)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (invoice, finance-report, blog-post, digital-eguide, docs-page, email-marketing, eng-runbook, hr-onboarding, meeting-notes, pm-spec, team-okrs, clinical-case-report, dcf-valuation, x-research, last30days); https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (doc, doc-kami-parchment, article-magazine, resume-modern, release-notes-one-pager, data-report, field-notes-editorial-template); https://github.com/nexu-io/html-anything; https://github.com/tw93/kami; https://github.com/mvanhorn/last30days-skill; https://github.com/virattt/dexter"
---
# Document templates

How you draw paperwork, reports, guides, newsletters, CVs and articles as pages on the canvas, family by family, with what goes on each page and in what order.

## Rules for every document

- **A document is not a website.** It follows its own template logic (folios, tables, running footers, small labels). The moment it is a web page (a blog post at 1440, a docs page, a help centre) Taste wins again.
- **Sizes.** A4 page 794x1123 (portrait). One-pager and CV: one A4. Email: 600 wide, height `fit_content`. Kuitansi: 794x561 (half A4, landscape). Web article or docs page: 1440 and 390.
- **Draw representative pages, not every page:** cover, one body spread, one table page, the back. Each page is its own top-level frame named "Katalog — Sampul", "Katalog — Hal. 6", 120 apart left to right. Give pages a fixed height and `clip: true`, so overflow shows instead of hiding.
- **Margins:** 56-64 inside an A4 page (padding [56, 64]); 40 on a kuitansi; 32 on an email column.
- **Running parts are components.** Draw the header, footer and page number once as reusable frames (`doc-footer`), then place them with `ref` and override only the page number and chapter in `descendants`.
- **Tokens, still.** Paper is `$color-background`, ink `$color-foreground`, quiet text `$color-muted-foreground`, table header and callout grounds `$color-muted`, hairlines `$color-border`, one accent `$color-primary`, charts `$color-chart-1` onward. Change the values per template; never hard-code a hex inside a page.
- **Type for print:** body 14/21 on A4 (13 in dense tables), labels and folios 12, nothing smaller. Headings 20-28, a cover title 44-64. One serif or one sans, or a display face plus a text face: never three. Numbers in tables right-aligned in fixed-width cells so the columns line up.
- **Real content, the person's language.** Usually Indonesian: "Jatuh tempo", "Ditagihkan kepada", "Laporan Bulanan · Oktober 2026". Money as Rp 1.250.000, decimals with a comma (12,5%), dates as 23 November 2026.
- **Never invent:** amounts, sales, stock, percentages, tax or company register numbers, bank account numbers, phone or WhatsApp numbers. A missing value is "—" or a muted pill "Belum diisi"; stand-in emails end in `.example`; addresses are street and district only.
- **No pictures as files.** A photo is a placeholder frame at its real ratio with a `Photo · …` note for Koda. Charts are drawn: bars are rectangles, a line is a `path`, the legend is small ellipses. Icons are stroked `path`s.
- **Budget:** 300 elements across the preview. An invoice page is about 90; a book page about 50-70. Draw 8-12 table rows, not 40.

## Shared parts (draw once, reuse)

- **Table:** a vertical frame of row frames. The header row has a `$color-muted` fill and 12px labels at 500; body rows have a bottom hairline (`stroke` thickness `{ "bottom": 1 }`). Make the row a reusable component and change its texts with `descendants`.
- **Totals block:** label left, amount right, in a frame 280-320 wide pushed to the right edge; the grand total row gets a top rule and a larger weight-600 figure.
- **Signature:** place and date, a 48px gap, a 1px rule 200 wide, then name and role.
- **Progress bar:** a `layout: "none"` frame with a muted track rectangle and a primary rectangle sized to the real percentage.
- **Checkbox:** a 16x16 rectangle, radius 4, border stroke; ticked = primary fill plus a tick `path`.

## 1. Business paperwork

**Invoice (faktur / tagihan).** For a freelancer or UMKM billing a client. Sections in order:
1. Top band: logo or wordmark left; right, "Invoice" large, then nomor, tanggal, jatuh tempo as label/value pairs.
2. Two columns: "Dari" (the sender) and "Ditagihkan kepada" (the client), name and street + city.
3. A thin strip: project or order reference and payment terms ("Pembayaran 14 hari").
4. Line items table: Deskripsi / Jml / Harga satuan / Jumlah. Description takes the remaining width.
5. Totals, right-aligned: Subtotal, then Diskon or Uang muka (DP) if any, then a PPN line only when the person says they charge it, at the rate they give, then "Total tagihan".
6. Payment: bank, account holder and account number exactly as given, else "Belum diisi" pills.
7. A short thank-you line and the signature.
Use the person's invoice number; with none, write "INV-001" and say it is a sample. Neutral paper, ink type, the brand colour once (the wordmark or the total). One sans family.

**Kuitansi (receipt).** Half A4, landscape. Title "Kuitansi" and number; rows "Telah terima dari", "Uang sejumlah" (the amount in words, "Satu juta dua ratus lima puluh ribu rupiah"), "Untuk pembayaran"; bottom left a boxed amount "Rp 1.250.000" in a large figure; bottom right place, date and signature. A meterai box only if the person asks for it.

**Price list and katalog.** One A4 page (or a cover plus pages). Header: business name, what it sells, "Berlaku mulai 1 Desember 2026". Group by category, each with a heading and an optional photo placeholder; item rows have the name and a short note left, the price right on the same baseline, with hairlines between rows. Footer: how to order in words ("Pesan lewat WhatsApp"; the number only if given), street + district, opening hours if given. For a longer katalog, draw the cover, one category page with 4-6 product cards (photo, name, size, price) and the order page.

## 2. Reports

**Laporan bulanan (monthly business or finance report).** One A4 page, or a cover plus two pages. In order: masthead (business, period, an "Internal" tag); a strip of four KPI cards (for a shop: Omzet, Laba kotor, Pengeluaran, Saldo kas), each with the value, the change versus last month with an up or down arrow and percent; a trend chart (12 bars or a line); a cost breakdown bar with two or three lines saying what it shows; a summary table (Pendapatan, Laba kotor, Biaya operasional, Laba bersih; this month vs last month); a top products or customers table; an outlook paragraph; prepared by, date, signature. Every figure comes from the person's data and appears in a labelled chart or table. Accent at most twice.

**Data report.** For a spreadsheet the person shares. Header with title, date range and a "Sumber data" line; 3-5 KPI cards with value, change and a tiny sparkline `path`; at least two main charts at a fixed height (240-280); an excerpt of the real data as a table with alternating `$color-muted` rows; 3-5 short insights led by a number or small icon (not emoji); a methodology note at the bottom.

**Field-notes editorial report.** A premium one-pager for an owner or board memo. Warm paper (#F4F0E8), near-black ink (#181715), soft pastel cards (lime #C9D57A, pink #E4B8CC, peach #E9CBAF), radius about 28. A small spaced top line (title left, period right); a grid where one big card spans two rows (a huge serif figure and an italic serif sentence) beside two smaller cards; then insight cards; then a chart panel with legend and axis labels. Serif display, clean sans body. Unknown values stay "—". No count-up or reveal: drawn still.

**Research brief.** For "what are people saying about…": title and time window, a short summary, a source coverage table (Sumber / Status: dicek, tidak tersedia, tipis / Catatan), findings by theme with [1] markers, the overall read with its confidence, limitations, a numbered source list. Never show coverage of a source that was not checked. More (DCF, clinical case) in [references/work-docs.md](references/work-docs.md).

## 3. Editorial

**Blog post or article (a web page: Taste applies).** At 1440 and 390: a plain masthead (wordmark, 4-6 links); the article header (category label, headline, a one or two sentence deck, author, role, date); a 16:9 hero photo placeholder with no caption under it; the body in one column of at most 62 characters (about 620 wide at 18px) with H2 sections, one pull quote with an accent rule on its left, one figure, one list, one blockquote; an author box (initials in a circle, short bio); three related posts (small photo, title, one line, date). Draw it as two 1440 frames (top, then the pull quote through related) plus 390.

**Magazine article.** A long-form essay layout: big title, optional subtitle, author, reading time, date; one centred column about 700 wide; serif headings over a sans body (or the reverse) for contrast; quotes with a thick accent rule and italic; bullets as small squares; sections divided by a small centred ornament instead of a full rule; a closing card ("Kalau bermanfaat, teruskan ke teman").

**Parchment document (kami style).** For writing that should feel typeset and printed: a one-pager, letter, long report, CV, changelog, small portfolio. Paper #F5F4ED (never pure white), second ground #EFEEE5, ink #1F1D18 (never pure black), quiet text #6B665B, hairline #D4D1C5, one accent only, ink blue #1B365D, for links, tags, key figures and the quote rule. One serif family throughout (Source Serif 4, or Charter where the person has it): body 400, headings 500, heading line height 1.1-1.3, reading text 1.5-1.55. No shadows, corners under 8 (use 2-4), tags as small solid tinted blocks, rules that stop short of the page edge. Hierarchy comes from size and space, not colour. Shapes: a one-pager (italic logotype, title, lede, three points in columns, metadata footer); a letter (sender address, date, recipient, left-aligned body, sign-off, signature rule); a long doc (cover, contents with page numbers, chapter pages with a folio and a section rule, notes, colophon).

## 4. Guides: e-guide, katalog resep, panduan

**E-guide.** A lead magnet or paid PDF (a recipe e-guide, a "Panduan merawat tanaman", a creator's guide). Draw the cover and one inside spread as A4 pages side by side, plus a recipe or lesson page.
- **Cover:** a small label line, the title (one serif display, one word in italic as the flourish), the author, a row of three facts about the guide itself ("12 resep · 4 bab · 1 daftar belanja"), "Isi panduan" as two columns of chapters with page numbers, a footer with a short tagline and page 01.
- **Inside spread:** chapter label ("Bab 02 · Sambal"), a subtitle that owns the page, two columns: an opening paragraph and a numbered list of four steps ("01 Sangrai cabai"), a pull quote with attribution, a "Coba sendiri" callout at the bottom, the footer with chapter and page number.
- **Recipe page (katalog/e-guide resep):** photo placeholder, title, a meta row (Waktu, Porsi, Tingkat), "Bahan" with quantities, "Langkah" numbered, one tip box.
Lots of space; italic once per page; small labels and page numbers in the text face, not mono. Pages sit flat, never tilted.

Docs pages, runbooks and onboarding plans are in [references/work-docs.md](references/work-docs.md).

## 5. Newsletter and marketing email

One 600-wide frame, one idea, one button, readable in ten seconds. In order: a preheader line in muted text; the masthead (wordmark left, two or three short links right, a hairline under); a hero photo placeholder 600x338; a short label in the accent; the headline in the loudest display style of the brand, two or three lines; two or three sentences of body; one primary button ("Lihat menu baru"); then a 2x2 grid of highlights (big figure or price, unit, label, e.g. "Rp 25.000 / Kopi susu aren"); the footer with wordmark, street + district, "Berhenti berlangganan · Lihat di browser". A toko newsletter may swap the grid for three product rows (photo, name, price) and a short story paragraph. Accent at most twice. Optional second frame at 360 if the person wants the phone view.

## 6. CV (resume)

One A4 page. Name large (34-40), one line under it with the role, then a contact row (city, email on `.example` unless given, portfolio link) split by thin vertical rules; no phone unless the person gives it, no photo unless asked (then a placeholder). Optional two columns: 60% main (Pengalaman, Proyek, Pendidikan), 40% side (Keahlian, Bahasa, Penghargaan). Each section label is small in the accent with a short 24x2 accent bar before it. Each job: company and role left, the date range right on the same line, then one to three lines that start with a verb and carry the person's real results. Black, greys and one accent (deep blue or ink green). For a freelancer, put "Klien & proyek pilihan" before work history.

## 7. Plans and team documents

Product specs, OKRs, meeting notes and release notes: structures in [references/work-docs.md](references/work-docs.md). A weekly update is a deck, not a document.

## Check before saving

- Each page has a name, fixed A4 size, padding and the running footer ref.
- Every number is the person's or marked "—"; no invented tax, register, bank or phone numbers.
- Table columns line up; amounts are right-aligned; the total is the strongest figure.
- One accent, used once or twice per page; one or two families.
- Web-facing pages (blog, docs) also pass `taste-check`.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), with html-anything templates (Apache-2.0), tw93/kami (MIT), last30days by Matt Van Horn (MIT) and workflow credit to virattt/dexter. Changed: rewritten for the OpenPencil canvas.
