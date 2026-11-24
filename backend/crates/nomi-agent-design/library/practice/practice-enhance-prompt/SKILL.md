---
name: practice-enhance-prompt
description: "How to turn a rough request or edit (\"tambahin search\", \"bikin halaman login\") into a precise spec in UI vocabulary before you draw: what's missing, the right component words, a numbered structure, and a one-change edit spec. Read when a request is vague or an edit is small."
license: "Apache-2.0 (google-labs-code/stitch-skills); via open-design (Apache-2.0)"
source: "https://github.com/google-labs-code/stitch-skills@62fdcd6 (plugins/stitch-utilities/skills/enhance-prompt + references/KEYWORDS.md), the open-design stub pointed at google-labs-code/skills; open-design@5fdcc3d"
---
# Sharpening a request before you draw

Restate a vague request as a short, structured spec in design vocabulary, so you draw the right thing the first time and the person can see what you understood.

## Step 1: what's missing

| Element | Look for | If missing |
|---|---|---|
| Platform | web page, app screen, phone first | infer from the brief; a shop page is web, a cashier tool is an app screen |
| Screen type | landing, menu, booking form, dashboard, settings | infer from the words |
| Structure | the sections or parts | write a numbered list of them |
| Look | mood words | take them from the design system already on the canvas |
| Colours | roles and values | use the tokens; never new colours for one screen |
| Components | UI names | translate everyday words into component names |

## Step 2: use the design system you have

If the canvas already has a design system frame, the spec reuses its tokens and components, named: "tombol `Button / Primary`, kartu `Card / Default`". If it has none, the first job is the system (`components-kit`), not the screen.

## Step 3: translate everyday words

| They say | Component |
|---|---|
| "menu di atas" | navigation bar with logo, links and the main action |
| "tombol" | primary button (the main action) or secondary button |
| "daftar barang" | card grid with photos, or a list with thumbnails |
| "formulir" | form with labels above fields and a submit button |
| "gambar besar di atas" | hero with a full-width photograph |
| "kotak yang bisa dibuka" | accordion |
| "notif kecil" | toast (Sonner) |
| "pilihan" | select, radio group, or segmented tabs |
| "loading" | skeleton or spinner |

Mood words get concrete too, and Taste decides the values:

| Vague | Concrete |
|---|---|
| "modern" | clean, generous space, one accent, neutral grounds |
| "profesional" | structured, calm, facts first, hairline borders |
| "fun" | rounder corners, warmer photos, livelier type, still one accent |
| "mewah" | big type contrast, a serif display, fewer things per screen |
| "natural" | earthy neutrals, real textures in photographs (not drawn textures) |

Taste wins where the source suggests what it bans: carousels, popups, "soft glows", gradients, or a dark theme on a place or product page.

## Step 4: write the spec

For a new screen, three parts:

1. One line: the screen's purpose and feel. "Halaman pesan kue: cepat, tenang, foto produk besar."
2. The system: platform and width first (390 first for an ordering app), theme, the tokens used.
3. The structure, numbered: `1. Nav: logo, Menu, Lokasi, tombol Pesan kue` · `2. Hero: foto kue utama, h1, harga mulai, tombol` · and so on.

For an edit, a targeted change spec, one change at a time:

> Tambah pencarian di nav.
> - Letak: kanan nav, sebelum tombol Pesan kue
> - Bentuk: `Input / Search`, ikon kaca pembesar di kiri, placeholder "Cari kue"
> - Lebar 240, 320 saat fokus
> - Di 390: ikon saja di nav, membuka kolom penuh saat diketuk
> - Selain ini, tidak ada yang berubah.

## Tips

- The vaguer the request, the more you fill, and the more your spec shows what you assumed.
- Match the intent: a person who wants "simple" gets fewer parts, not a richer design.
- One change per edit; don't bundle unrelated improvements into a small request (say them as suggestions instead).
- Keep the person's own words for their content; your vocabulary is for structure.

Adapted from Google Labs' Stitch enhance-prompt skill (Apache-2.0, google-labs-code/stitch-skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
