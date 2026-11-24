---
name: style-neoclassical-ledger
description: "The page as an illustrated ledger on warm parchment: one or two giant sans words per chapter, book-serif sentences, mono ledger categories, broken ink rules between entries, and each service paired with one object silhouette you can read at phone size."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Neoclassical Ledger

The page as an illustrated ledger on warm parchment: one or two giant sans words per chapter, book-serif sentences, mono ledger categories, broken ink rules between entries, and each service paired with one object silhouette you can read at phone size.

## The look

Parchment paper with a lighter sheet laid on it. A chapter opens with a huge two-word sans title. Below, a two-column folio of four entries: each has a mono index, a serif title and note on one side, and a cut-out object (a folded invoice, an open journal, a brass ring, a stamp) on the other. Ink is brown-black; nothing is bright. It feels like a careful bookkeeper's studio.

## Suits / avoid

- **Suits:** bookkeeping and tax consultants, notaries, cooperatives (koperasi), jamu and herbal makers, tailors, printers (percetakan), stationery and bookbinding, antique and heritage shops, letterpress wedding invitations.
- **Avoid for:** fast food, tech startups, youth brands, dark-mode apps.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #EFE2C8 | Paper |
| `$color-card` / `-foreground` | #F9EED7 / #2C241B | Lighter sheet |
| `$color-foreground` | #2C241B | Ink: titles |
| `$color-secondary-foreground` | #514537 | Body ink |
| `$color-muted-foreground` | #695841 | Captions, mono categories |
| `$color-border` | #CDBB98 | Rules |
| `$color-primary` / `-foreground` | #2C241B / #F9EED7 | Primary button |
| `$color-accent` / `-foreground` | #E8D7B4 / #2C241B | Selected entry, hover |
| `$color-signal` | #9C8764 | Brass: index numbers, small marks |
| `$color-ring` | #4B3929 | Focus |

Radii: `$radius-sm` 2; everything else square. Fonts: `$font-heading` Geist 600 (creator's Geist Sans), `$font-body` Newsreader 400 (the creator used Geist Serif; Newsreader is the closest Google Fonts book serif), custom `$font-mono` Geist Mono.

## Type

- Chapter title: one or two words, 62–156px, 600, letter spacing −4%. On the phone frame, check the longest word fits 358px; drop size before breaking a word.
- Entry title: serif 28–32px. Notes: serif 17–19px/28, `$color-secondary-foreground`.
- Categories and indices: mono 11px uppercase, tracking 10%, `$color-muted-foreground` ("LEDGER 02 · PAJAK").
- Nav 14–17px sans.

## Layout signatures

1. **Chapter head:** giant title left-aligned on paper; a mono line below with chapter number and date "26 Jul 2026".
2. **Folio:** sheet frame (`$color-card`, padding 48) with two columns and one thin vertical rule (1px `$color-border`) at the split.
3. **Entry:** horizontal, gap 32: object placeholder 40% (sized to its visible ink, open paper around it, caption "Ilustrasi: faktur terlipat, tinta cokelat") and text 60% (mono index "01", serif title "Pembukuan bulanan", 3-line note, price "mulai Rp 750.000 / bulan").
4. **Broken rule between entries:** stroke 1 `$color-border` with dash pattern (6 on, 4 off) or two segments with a gap, never a solid line.
5. **Footer:** mono address "Jl. Malioboro No. 12, Yogyakarta · buku@kantorsiti.example".

Phone: entries stack, object above text, never overlapping body copy.

## Components

- Primary: fill `$color-primary`, height 46, radius 2, serif 16px label "Jadwalkan konsultasi".
- Secondary: text link with 1px underline in `$color-foreground`.
- Category tag: mono 11px, stroke 1 `$color-border`, padding 4 8, radius 2.

## Copy voice

Precise, old-fashioned courtesy, short. "Kami catat, Anda tenang." Services named like ledger lines.

## Tells

- Giant one- or two-word sans chapter titles on parchment.
- Serif notes with mono ledger categories and brass index numbers.
- One object silhouette per service, given room to breathe.
- Broken ink rules between entries; one vertical rule at the folio split.
- Brown-black ink only; no bright colour anywhere.

## Note for Koda

The source tiles a low-alpha paper grain and faint engraved marks; the canvas uses flat paper. Koda may add grain only if text reads equally well with it off. The object illustrations need real drawn or commissioned art; until then they are captioned placeholders.

Adapted from mengto/skills: agent-skills/web-design/neoclassical-ledger-ui (MIT, Copyright (c) 2026 Meng To).
