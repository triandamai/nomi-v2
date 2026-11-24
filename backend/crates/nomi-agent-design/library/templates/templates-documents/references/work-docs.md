# Work documents

Structures for the working documents a team keeps (help and docs pages, a service runbook, a new-staff onboarding plan, a feature spec, OKRs, meeting notes and release notes), with how to draw each.

These are calm, functional pages: one sans for text, a mono for codes and times, one accent for status and links, hairline dividers, status shown with Badges (kit components). Draw them as a 1440 web page when they live online (then Taste's app rules apply) or as A4 pages (794x1123) when printed or sent as PDF. Use the person's real names, roles and dates; stand-in contacts on `.example`.

## Help or docs page

**For:** a help centre for an app, "Cara pesan" instructions, a POS manual.

- **Left nav** (240-280): 3-5 groups of 4-8 links, the current page in bold with a short accent stripe on its edge.
- **Article** (text column ~720 wide, centred): h1, a lede paragraph, h2 sections, numbered steps, a note callout and a warning callout, inline links, a screenshot placeholder of the real screen at true size. Code or keys in mono on a soft surface, only for technical products.
- **Right "Di halaman ini" contents** (200-240): the h2/h3 list, current section highlighted.
- Lines 60-75 characters long; accent only on the active nav item, links and one callout.
- On phone the contents rail drops and the nav becomes a Sheet.

## Service runbook (technical clients)

Header (service, owner team, severity tier, version) · a summary paragraph and dependency list · alerts table (alert / severity / what it means / first response) · dashboards and links · 3-4 common procedures (deploy, roll back, rotate keys) as numbered steps · on-call rotation table (week / primary / secondary / backup) · a five-step incident checklist. Dense and plain; severity in Badges (red, amber, grey) with words, not colour alone.

## New-staff onboarding plan

**For:** a cafe's new barista, a clinic's new front desk, an agency hire.

1. **Cover banner:** name, role, start date, manager and buddy.
2. **Hari pertama:** the literal schedule (`08.00 Perkenalan tim`, `12.00 Makan siang bareng`, `14.00 Latihan kasir`).
3. **Minggu pertama:** Monday to Friday (or Saturday), two activities a day, as a five- or six-column row.
4. **30 / 60 / 90 hari:** three cards, three concrete outcomes each ("Bisa buka toko sendiri").
5. **Resources:** handbook, group chats, the SOP sheet, payroll setup.
6. **"Kamu siap kalau…" checklist:** five outcomes with checkboxes.

Draw it as one long A4 sequence (two pages) or a 1440 page; the day columns and milestone cards as reusable components.

## Feature spec (one page)

Header strip (title, status Badge: Draft / Review / Disetujui, date, owner) · three-line summary: what, who, why now · problem (one paragraph and a real customer quote if there is one) · goals and non-goals side by side · success metrics table (metric / target / how measured) · user stories ("Sebagai kasir, saya ingin…, supaya…") · scope as 3-4 phases · open questions with an owner chip each.

## OKRs (quarter plan)

Quarter banner (quarter, dates, overall progress chip) · three objective cards, each with its title, owner Avatar, status Badge (On track / Berisiko / Tertinggal) and three key results as rows (metric, current → target, Progress bar) · a right column: headline numbers, biggest movers, a blockers callout. One accent for progress; red only for "Tertinggal".

## Meeting notes

Header (title, date, time, place or call link, attendee Avatars) · agenda as a checklist of 4-6 items · decisions in a strong panel (the part people look for) · action items table (what / owner / due / status) · open questions and the next meeting in a footer. Subdued colours, clear hierarchy.

## Release notes (one pager)

**For:** an app update, a new menu season ("Yang baru di bulan ini"), a website update.

1. **Hero:** version or season, date, one-sentence summary.
2. **Baru:** a list of additions, each a short title and one line.
3. **Diperbaiki:** fixes.
4. **Perubahan penting:** breaking changes, or one row that says "Tidak ada".
5. **Masalah yang diketahui:** known issues with a workaround.
6. **Yang perlu kamu lakukan:** short upgrade steps, or a plain "Tidak perlu apa-apa".

A top nav jumps to each section; a call-to-action button only if a real destination exists. Each list row is a reusable component (date or tag in mono, title, one line).

## Drawing notes for all of them

- Running header and footer (document name, page number `2 / 4`, date) as reusable frames placed with `ref`.
- Tables as a reusable row component; at most ~12 rows a page, then continue on the next page.
- Status always as text in a Badge, never colour alone.
- Long documents: draw the first page, one typical inside page and any special page (a table, a checklist); list the rest in a `Pages` note.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
