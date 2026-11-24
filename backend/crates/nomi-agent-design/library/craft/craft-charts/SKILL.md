---
name: craft-charts
description: "How to design charts, KPI tiles and data reports on the canvas: where Taste lets a page show data, choosing the chart, editorial chart rules, drawing bars, lines and points as shapes, report layout, honest data and motion notes. Read it for dashboards, reports and any chart on a page."
license: "Apache-2.0, nexu-io (html-anything: data-report, frame-data-chart-nyt, after HeyGen HyperFrames' data-chart); MIT, Copyright (c) 2026 Jiannanya (snow-d3, topic only); via open-design (Apache-2.0)"
source: "open-design@5fdcc3d skills data-report and frame-data-chart-nyt (upstream github.com/nexu-io/html-anything), d3-visualization (catalogue stub for github.com/jiannanya/snow-d3)"
---
# Charts, KPI tiles and data reports

How you design data on the canvas so it reads in seconds: when a chart belongs on the page at all, which chart, the editorial rules that make it clear, how to draw it from rectangles, paths and text, and how to lay out a whole report.

A chart as a video frame (a 16:9 newsroom-style chart for a reel) is in `video-frame-templates`; this skill is for charts on websites, in apps and in reports.

## Where data may appear

| Kind (`taste-core`) | Charts |
|---|---|
| Software, tech | Yes: the product's own dashboards and reports drawn as real UI with real-looking data; one diagram or chart card under an h2 |
| Product, service | Rarely: a real figure from the brief as a stat under an h2, a comparison table; a chart only if the brief gives the data |
| Place | Never: no charts, stats strips or counters on a cafe, salon or guesthouse page |
| Apps Nomi builds (owner dashboards, kasir, stock, bookings) | The main use: sales by day, top products, stock levels, booking occupancy |

## Choose the chart by the question

| The question | Chart |
|---|---|
| How does it change over time? | Line (one or two series); columns for few, discrete periods (7 days) |
| Which is biggest? | Horizontal bars, sorted, labels on the left |
| What share of a whole? | One stacked bar or a short sorted bar list; a donut only for 2-3 parts with the total in the middle; never a pie with 6 slices |
| How far from a target? | A progress bar or bullet bar with the target mark |
| Where is the range? | A range band with the median line |
| Is there a relationship? | Scatter, only for analysts |
| One number that matters | A KPI tile, not a chart |

D3-scale visuals (treemaps, sunbursts, force graphs, sankeys, choropleth maps) fit internal analytics tools, not a small business owner's daily screen; offer them only when the person's data really has that shape.

## The headline is the finding

Write the chart's title as the conclusion drawn from the data, not a description of the axes:
- Good: "Penjualan Sabtu dua kali hari kerja", "Stok gula habis dalam 4 hari".
- Bad: "Grafik penjualan mingguan".
A small line above or below names the source and period in sentence case, 13/500: "Data kasir · 1–30 November 2026". (The source's tracked capital kicker is replaced: Taste bans tracked caps.)

## Editorial chart rules

- **One ink, one accent.** All series in `$color-foreground` or `$color-chart-1`; the one bar or line that carries the finding in the accent. Use `$color-chart-1` to `$color-chart-5` in order only when several series must be told apart, and never more than five.
- **Label directly.** Names at the end of each line, values at the top of key bars; avoid legends when four or fewer series.
- **Axes quiet.** A hairline baseline, three or four ticks, tick labels outside in 13px muted, numbers in tabular figures. No full-screen gridlines; if needed, faint horizontal lines at the ticks only.
- **Annotate the point that matters:** a 6px dot and a short label ("Lebaran · Rp 12,4 jt").
- **Start bars at zero.** Lines may zoom in, but say so with the axis.
- **No 3D, shadows, gradients, glows or emoji** on charts.
- **Main line 2.5px, secondary 1.5px** in a muted tone (the canvas has no dashed stroke; use tone, not dashes, and label it).
- **Indonesian formats:** "Rp 1,2 jt" on axes, full value in the tooltip state; dates "Sen", "24 Nov".

## Drawing it on the canvas

The canvas has no chart node; build each chart from primitives inside a frame named for it (`sales-chart`), with `layout: "none"` so you can place by coordinates:
- **Bars:** `rectangle` nodes, one per value, equal widths and gaps, height proportional to the value, top corners radius 2-4 at most. Compute heights from the real numbers: height = value / max x plot height.
- **Lines:** one `path` per series with straight segments through the points (`M x0 y0 L x1 y1 …`), stroke only, round joins.
- **Points:** `ellipse` nodes 6-8px at the key values only.
- **Range band:** a closed `path` filled with a light tone, the median line above it.
- **Labels and ticks:** `text` nodes; tick marks as 1px rectangles.
- **Sparkline in a KPI tile:** a single path, 40px tall, no axes.

Keep a chart under about 40 elements (the preview's limit is 300); for a 30-day series use columns per day only on desktop and a line on the phone.

Draw the states: loading (a skeleton of the plot area and two tick lines), empty ("Belum ada penjualan di periode ini" and the action), error with retry, and the tooltip on one point as a separate frame.

## KPI tiles

Three to five per screen, one row on desktop, two-up or a horizontal list on the phone. Each tile: the label (13/500 muted), the value large in tabular figures, the change against the previous period with its sign and colour (green up, `$color-destructive` down, and the word, not colour alone: "naik 12% dari minggu lalu"), an optional sparkline. Never a tile for a number the person can't act on.

## A data report page

When the person sends a CSV or sheet and asks for a report (`data-report`):
1. **Header:** title, the period, the data source in one line.
2. **KPI row:** three to five tiles.
3. **Main charts:** at least two, each answering a different question, each with its finding as the title.
4. **Table:** an excerpt of the real rows; hairline rows, sticky header, numbers right-aligned.
5. **Insights:** three to five plain sentences, each a finding with its number ("Kue lapis menyumbang 34% pendapatan, naik dari 22% di Oktober"). The source starts them with emoji; Taste doesn't allow emoji as bullets.
6. **Method:** a collapsed section saying how numbers were counted.
Palette restrained: one primary, a neutral scale, the chart tokens.

## Honest data

- Use the person's data. Parse it; never invent numbers to fill a chart (Taste: nothing invented).
- If you only have a sentence ("penjualan naik terus sejak Juli"), draw a schematic chart and label it in the canvas note as `Ilustrasi · bentuk saja, bukan data`; ask for the real numbers before it ships.
- In an app mockup with no data yet, use plausible sample data and list it in the handoff as sample.

## Motion notes

In apps, charts don't animate on every load (they are seen many times a day): at most a 150 ms fade when data changes. On a tech page or a report's first view, a staggered reveal may be one moment: the title, then the source line, then the line draws from left to right, then labels one after another (`craft-motion-notes`; durations from `motion-timing`). Reduced motion: the finished chart.

Adapted from html-anything's data-report and frame-data-chart-nyt templates (Apache-2.0, nexu-io; after HeyGen HyperFrames' data-chart) and the topic of snow-d3 (MIT, Copyright (c) 2026 Jiannanya), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
