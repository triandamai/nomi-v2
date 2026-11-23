---
name: style-split-technical
description: "A disciplined two-panel page: one half is the visual (photo or product), the other is the facts, set in a calm serif with mono spec rows, thin frames and corner squares, so detail pages feel precise and trustworthy."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Split Technical

A disciplined two-panel page: one half is the visual (photo or product), the other is the facts, set in a calm serif with mono spec rows, thin frames and corner squares, so detail pages feel precise and trustworthy.

## The look

Desktop splits 50/50. The media half is atmospheric: a framed photo window on a dark or tinted panel. The info half is dense and ordered: a mono top rail, a clean headline, then a spec table of label/value rows separated by hairlines, then the action. Each half has its own inset frame and tiny corner squares, like two display surfaces side by side.

## Suits / avoid

- **Suits:** kos and rental rooms, used cars and motorbikes, furniture and appliances, property listings, course syllabi, event detail pages, menu item detail, product detail in small shops.
- **Avoid for:** emotional storytelling homepages, playful brands, long marketing pages.

## Tokens

| Token | Light | Dark panel | Role |
|---|---|---|---|
| `$color-background` | #F6F5F2 | — | Info half |
| `$color-foreground` | #1C201C | — | Text |
| `$color-card` / `-foreground` | #FFFFFF / #1C201C | — | Embedded surfaces |
| `$color-chapter` / `-foreground` | #1B241A / #F6F5F2 | — | Media half |
| `$color-muted` | #ECEAE5 | — | Spec row zebra, inactive tab |
| `$color-muted-foreground` | #767873 | — | Labels, captions |
| `$color-primary` / `-foreground` | #1C201C / #F6F5F2 | — | Primary action |
| `$color-border` | #1C201C1F | #FFFFFF1F | Frames, rows |
| `$color-ring`, `$color-signal` | #B5532E | — | One accent: availability, active tab |
| `$color-success` / `-foreground` | #E5EFE3 / #2F5D2A | — | "Tersedia" |

Radii: `$radius-sm` 2 · `$radius-md` 4 · `$radius-lg` 8 (media window). Fonts: `$font-heading` Newsreader 400, `$font-body` Inter 400/500, custom `$font-mono` JetBrains Mono.

## Type

- Headline 40–56px serif, 400, letter spacing −1.5%, max 2 lines.
- Spec labels: mono 11px uppercase, tracking 12%, `$color-muted-foreground`. Values: sans 15px/500.
- Body 16px/26. Rails: mono 10px.

## Layout signatures

1. **Split:** horizontal frame, two children fill_container (shift to 55/45 if the photo must dominate). Phone: media on top (4:3), facts below, each still framed.
2. **Panel frame:** each half padding 32, an inset 1px `$color-border` rectangle 16px in from the edges, 6px squares at its corners.
3. **Top rail:** mono row space-between: "KOS PUTRI · KAMAR 03" left, "DIPERBARUI 26 JUL 2026" right.
4. **Media window:** photo placeholder with stroke 1 and a mono caption ("Foto: kamar 3×4 m, jendela timur, kasur 120"); thumbnails row of 4 below, the active one with a 2px `$color-signal` bottom stroke.
5. **Spec block:** vertical list, each row horizontal space-between, padding 12 0, bottom stroke: Ukuran / 3 × 4 m; Harga / Rp 1.350.000 per bulan; Listrik / Token, terpisah; Kamar mandi / Dalam; Lokasi / Jl. Kaliurang km 5.
6. **Action foot:** primary "Jadwalkan survei", secondary "Tanya lewat WhatsApp", availability pill.

## Components

- Buttons: height 46, radius 2, primary filled, secondary stroke 1 `$color-foreground`.
- Status pill: `$color-success` fill, 12px, radius 999, only where status is real.
- Tabs (Detail · Fasilitas · Aturan): mono 11px, active underline `$color-signal`.

## Copy voice

Factual and complete. Every value has a unit. "Termasuk Wi-Fi dan air. Listrik bayar sendiri." No adjectives a spec row could replace.

## Tells

- Two framed halves with distinct jobs: one visual, one factual.
- Inset frame lines with corner squares inside each panel.
- Mono label / sans value rows separated by hairlines.
- A mono metadata rail on top.
- One accent only on availability and the active tab.

Adapted from mengto/skills: agent-skills/web-design/split-layout-technical (MIT, Copyright (c) 2026 Meng To).
