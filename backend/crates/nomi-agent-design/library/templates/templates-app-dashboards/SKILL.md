---
name: templates-app-dashboards
description: "App and dashboard templates (admin dashboard, live team dashboard, kanban, analytics, trading, daily briefing, mobile app archetypes, onboarding, gamified app): regions in order, data density, shadcn mapping, states, and what Taste overrides. Read before drawing a dashboard or app screens."
license: "Apache-2.0, Copyright nexu-io open-design contributors (open-design)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates (dashboard, live-dashboard, live-artifact, github-dashboard, flowai-live-dashboard-template, trading-analysis-dashboard-template, social-media-dashboard, social-media-matrix-tracker-template, kanban-board, mobile-app, mobile-onboarding, gamified-app, dating-web, orbit-general, orbit-github, orbit-gmail, orbit-linear, orbit-notion, worker-visualizer)"
---
# App and dashboard templates

The screen structures open-design's app and dashboard templates use, translated into shadcn components and canvas frames, with Taste's app rules on top.

Read `../../taste/taste-app-screens/SKILL.md` first; it wins on look. These templates give you the regions, their order and the data each must show. Build from kit components (`../../components/components-kit/SKILL.md`): Sidebar, Card, Table, Chart, Tabs, Badge, Avatar, Progress, Empty, Skeleton, Sonner, Dialog, Sheet.

## Data on app screens

An app screen is drawn with realistic content, never `Metric A` or lorem. Use the person's real items (their menu, products, services, staff names they gave) and realistic sample records for transactions (orders, bookings, invoices) that look like their business: `INV-0142 · Bu Sari · Rp 245.000 · Lunas`. Where a real figure is unknown on a report screen, show `—`. Never put invented business results on a marketing page.

## Admin dashboard (the default shape)

**For:** a shop owner's sales overview, a clinic's bookings, a kursus' students, a POS back office.

1. **Sidebar** (240 wide): brand mark at top, 6-8 nav items with icons, the active item highlighted with `$color-sidebar-accent`, the user at the bottom. On phone it becomes a Sheet behind a menu button, or a bottom tab bar of 4-5 items.
2. **Top bar** (56-64 tall): page title left; search, period picker and avatar right.
3. **Row 1, KPI cards (3-4):** label, a big number, the change vs the previous period (`▲ 12% dari minggu lalu`), optional sparkline. Use the reusable `KPI` card.
4. **Row 2, the primary chart:** full width or two-thirds, with its title as the takeaway ("Penjualan 7 hari terakhir"); bars for days, a line for trends, the series that matters in `$color-chart-1`.
5. **Row 3, a table or feed:** recent orders, top products, today's bookings; 5-8 rows, status as Badge, a "Lihat semua" link.

**Draw:** 1440 desktop and 390 phone at least. On phone, KPIs become a 2x2 grid or a horizontal row, the chart full width, the table a list of Items.

## Team workspace dashboard (live, document-like)

A calm, page-like dashboard in the style of a team wiki: a 240 sidebar, a 44 topbar, a page column max ~1100 wide.

- **Classic:** page header (icon, title, meta row: updated time, owner) · a short callout explaining what the page shows · a KPI grid on 1 px hairlines · a two-column block (a 7-day sparkline card, a recent activity feed) · a linked table (tasks of the active sprint) · footer.
- **Kanban flavour:** the table replaced by a 3-4 column board.
- **KPI-only:** header, callout, KPIs, nothing else: for a TV screen in the shop or kitchen.
- **Parts worth reusing:** a small "Live" pill in the topbar (a dot + "Diperbarui 2 menit lalu"), status pills in a fixed five-colour set (grey not started, blue in progress, amber waiting, green done, red blocked), a primary Refresh button and a ghost Auto toggle in the page header, a toast for "Data diperbarui".
- Refresh behaviour, cards moving between columns and accent flashes are build notes.

## Kanban board

Top bar (breadcrumb, sprint or week chip, filters by person/label/status, search) · 4 columns (Backlog or `Masuk`, Dikerjakan, Dicek, Selesai), each with a count Badge and an "+ Tambah" ghost button · 3-6 cards a column (tag chip, title, assignee Avatar, due date or points, progress if any) · an optional side panel: progress bar, top assignees, a blocked-items callout.

For small businesses this fits order or production tracking (catering orders from `Pesanan baru` to `Dikirim`). Draw the card as a reusable component with Default, Hover (lifted, soft shadow) and Selected states; a drag in progress is a note.

## Analytics and social media dashboards

**Order:** header (brand + section label, period switch) · a one-sentence summary in display type ("Engagement Instagram naik setelah konten resep") · optional ask bar (an input styled as search) · platform switcher (cards per channel the person really uses, with the real brand icon) · KPI row (followers, engagement rate, likes 7 days, reach) · main grid 2/3 + 1/3 (main trend chart; top posts list) · deeper sections (posting heatmap, content-type split).

Multi-platform trackers add a platform x metric matrix table and per-chart insight lines. Keep to one chart per card; hover tooltips, pinning and range selection are notes.

## Finance and trading dashboards

Dense, number-first: an instrument list, a main price chart with crosshair, order book or positions table, a news or signal feed. Use mono tabular figures, green/red only for up/down (with ▲▼ so colour isn't the only signal), dark theme allowed (fintech is a tech kind in Taste). Unknown values are `—`; light/dark, live/demo switch, chart focus and a `/` command palette are notes.

## Team and member dashboards (tabs)

Three tabs (Members, Details, Activity log): a member table (name, role, status, last active), a role distribution bar chart, an activity list with timestamps; an Export CSV button. Draw each tab state as its own frame, same ids for shared parts.

## Repository and project health dashboards

For a software client: stars, forks, contributors, open issues and PRs as KPIs; a growth chart; contributor table; issue and PR activity; every number shows where it came from and when (a small "as of" line). Treat as software kind.

## Daily briefing digest

A one-page "what happened in the last 24 hours" across the tools a person uses (email, calendar, tasks, chat, code):

1. Hero row (~80 tall): greeting, date, one-line summary.
2. KPI strip (5 equal cells): unread, meetings today, tasks due, mentions, alerts.
3. Today's timeline (full width): the day's meetings as blocks on an hour axis.
4. Top 3 (three equal cards): the three things to do first, each with why and a link.
5. Source modules (an adaptive bento): one module per connected tool, each in a shape that fits its data (a list for email, a board for tasks, a mini chart for errors).
6. People waiting on you (a strip of Avatars with what they need).
7. Footer line: sources and refresh time.

Only show sources the person actually connected; an empty module uses Empty with a plain line, not fake rows.

## Mobile app screens: six archetypes

Pick by the screen's main action:

| Brief says | Archetype | Structure |
|---|---|---|
| feed, inbox, list, messages | **Feed** | App bar, optional segmented tabs, a list of Items or cards, bottom tab bar |
| a product, a recipe, an article, a booking | **Detail** | Large picture (placeholder at true ratio), title, key facts row, description, sticky bottom action ("Pesan · Rp 35.000") |
| sign-up, welcome, walkthrough | **Onboarding** | Picture or icon, headline, one paragraph, page dots, full-width primary button, Skip top-right |
| profile, account | **Profile** | Avatar, name, short bio, stats row, tabs, content list |
| checkout, payment, form, settings step | **Checkout / form** | Step indicator, grouped Fields, order summary, total, one primary button |
| timer, map, single big number | **Focus** | One hero element centred (a timer, a map placeholder, a big balance), 1-2 controls |

**Phone frame:** 390 wide, status bar, safe area at the bottom (home indicator), tab bar 4-5 items with `path` icons and labels. Tap targets at least 44.

## Onboarding flow

Three phones side by side (390 each, 120 apart), each one idea: what it does, how it helps, a reason to trust (only true: "Data kamu tersimpan di HP"), and the last one is sign-in (WhatsApp/phone number, Google, email). Same ids for the dots, Skip and button across the three.

## Gamified app (habit, learning, fitness)

Three phones: a cover screen selling the value; a "today" screen with quests or tasks, streak, XP progress bar; a stats or reward screen. Bottom tab bar with a central action. The template's dark stage and neon belong to the template; Taste allows a dark app only for gaming or tech kinds, otherwise use the brand's light theme with one accent for progress.

## Consumer web app shell

A left rail, a strip of community signals, headline KPIs, a 30-day bar chart and a trend block: the same admin shape, in a more editorial type. Use it for a membership or community app.

## Heavy real-time visualisers

Particle or simulation visualisers are engineering showcases; for a tech client draw one frame with the composition in flat shapes and a build note. Not for small-business apps.

## States you always draw

For every data region on the screens you deliver: **loading** (Skeleton blocks at the real sizes), **empty** (Empty: icon, one line, the action: "Belum ada pesanan hari ini · Bagikan link toko"), **error** (Alert destructive with a retry), and the normal state. Put them on one `States` frame per screen type, with ids matching the screen.

## Before you hand over

- Regions in the template's order; one primary action per screen.
- Everything from kit components placed with `ref`; tokens only.
- Realistic, business-specific data; unknowns as `—`.
- Desktop and phone drawn; loading, empty and error states drawn.
- Live behaviour, animations and interactions written as notes.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
