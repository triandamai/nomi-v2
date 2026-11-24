---
name: practice-design-brief
description: "How to turn a vague request (\"bikin yang profesional\") into eight explicit design decisions before you draw: map the person's words to dimensions, resolve each, flag what they never said and report the defaults you chose and why. Read at the start of every new design."
license: "Apache-2.0, Copyright nexu-io (open-design); via open-design (Apache-2.0)"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3db1e31bd4fa70f673041b78e87977507c3/skills/design-brief; open-design@5fdcc3d"
---
# Design brief: eight decisions before any frame

Turn the person's words into eight explicit decisions, say which ones you assumed, and write them on plan line 2 (the deal) before you draw a single frame.

"Bikin yang bersih dan profesional" means something different to every person. Most of the arguing after a first version comes from a dimension nobody decided. This skill makes every dimension a stated decision, so the person can correct an assumption in one line instead of rejecting a whole design.

## The eight dimensions

| # | Dimension | What you decide | Where it lands on the canvas |
|---|---|---|---|
| 1 | Palette | ground, surface, text, secondary text | `$color-background`, `$color-card`, `$color-foreground`, `$color-muted-foreground` |
| 2 | Accent | the one action colour and its pressed step | `$color-primary`, `$color-primary-foreground`, `$color-ring` |
| 3 | Body type | family, size, line height | `$font-body`, body text style |
| 4 | Display type | heading family and weight, or the body family | `$font-heading`, display/h1/h2 styles |
| 5 | Layout model | single column, two column, asymmetric | the 12-column grid use per section |
| 6 | Mood | professional-minimal, playful, bold/raw, editorial | radius ladder, button look, photo treatment |
| 7 | Density | compact, balanced, spacious | section padding, gaps, `$space-*` scale |
| 8 | Constraints | what is excluded (dark mode, animation, stock photos, carousels) | your never-list on plan line 2 |

Animation timing, breakpoints and contrast are not brief-level choices: Taste and `taste-check` already fix them (390, 768, 1440; 4.5:1 body contrast; the hover layer).

**Taste wins on values.** The source skill ships a closed table of hex values and fonts (navy-and-white, electric blue, Inter as the default body). Don't use it. Taste deals palette and type by the trade and the kind (`taste-color`, `taste-type`), and the client's own colours replace the palette. What you keep from this skill is the method: every dimension decided, every default explained.

## Step 1: read the person's words into dimensions

Go sentence by sentence and tag each phrase with the dimension it touches. One phrase can touch several ("landing gelap yang bersih" sets palette, mood and layout); resolve each separately.

| They say | Dimension | Read it as |
|---|---|---|
| "gelap", "dark mode" | palette | dark ground: only on software, gaming or tech (Taste: never on a place, product or service) |
| "terang", "putih", "bersih" | palette + mood | light neutral ground, professional-minimal |
| "hangat", "earthy", "natural" | palette | warm neutrals from the trade's palettes |
| "ada warna yang nendang", "vibrant" | accent | one saturated accent, still 90-95% neutral interface |
| "kalem", "subtle" | accent | a muted accent close to the neutrals |
| "simpel", "minimalis", "rapi" | mood | professional-minimal |
| "fun", "ceria", "ramah anak" | mood | playful: rounder radius ladder, warmer photos |
| "berani", "beda", "raw" | mood | bold: big type contrast, sharp ladder |
| "kayak majalah", "editorial" | mood | editorial: a serif display, generous margins |
| "lega", "banyak ruang" | density | spacious |
| "padat", "banyak info" | density | compact |
| "pakai font X" | type | that family, if it is licensed for the web |
| "klasik", "elegan" | display type | a serif display |
| "nggak usah animasi" | constraints | no hero entrance beyond a fade; still the hover layer |
| "jangan pakai foto stok" | constraints | only the client's photos; ask for them |
| "satu halaman aja" | layout | single page, single column sections |
| "ada sidebar" | layout | app shell with sidebar (an app screen, `taste-app-screens`) |

When a phrase is ambiguous between two readings, take the first one in the table unless the rest of the brief clearly points to the second. When a word maps to nothing ("pokoknya yang wow"), don't guess: ask one short question with two or three concrete options and a recommended default, or show them as directions.

## Step 2: resolve every dimension

Every one of the eight gets a value. For an unspecified one, apply a default by rule, not by mood of the moment:

| Unspecified | Rule |
|---|---|
| Palette | deal from the trade's palettes (`taste-color`); client colours first |
| Accent | from the logo or shopfront; otherwise the trade palette's accent |
| Body type | the kind's pairing in `taste-type` |
| Display type | the kind's pairing; editorial mood favours a serif display |
| Layout | single column sections on the 12-column grid |
| Mood | follows the kind and price: a clinic is facts-only and calm, a kids' class is playful |
| Density | balanced (Taste's one section per screen already sets the rhythm) |
| Constraints | Taste's never-list only (no carousels, popups, parallax, scroll fade-ins) |

If the mood itself is missing, every other default falls back to the calm, neutral set for the kind.

## Step 3: show the decisions on the canvas

Your design system frame is the brief made visible. Draw it before the screens, in this order:

1. **Swatches:** one rectangle per colour token, labelled with its role and hex (Background #F7F4EE).
2. **Type specimens:** display, h1, h2, body, small, each at its real size, set in real copy from this project, never "The quick brown fox".
3. **Spacing ruler:** stacked bars for section padding, content gap and card padding, each labelled in px.
4. **Components:** a primary button, a card with a title and body, a text field, all as `reusable` frames named `Button / Primary / Default` and so on (`components-kit`).

## Step 4: report what you assumed

In the message that goes with the first version, list every dimension you set by default, with its rule, in the person's language:

> Yang saya tentukan sendiri (silakan koreksi):
> - Huruf judul: Fraunces (aturan: toko roti, pasangan editorial hangat)
> - Kerapatan: seimbang (Anda belum menyebut ingin lega atau padat)
> - Larangan: tidak ada carousel atau pop-up (aturan dasar)

This one list prevents silent assumptions from travelling into version three.

## Rules worth keeping

- Use only the declared tokens; a screen that needs a raw hex means the token set is incomplete.
- Keep section spacing consistent across the page.
- Two typefaces at most for display and body; a mono face for figures or code doesn't count.
- The accent appears at most three times per screen (Taste: one main action a screen).
- Shadows only where your elevation scale allows them.

Adapted from open-design's design-brief skill (Apache-2.0, nexu-io/open-design), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
