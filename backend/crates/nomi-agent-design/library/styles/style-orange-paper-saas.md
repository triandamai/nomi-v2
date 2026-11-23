# Style: Orange Paper SaaS

Welcoming product style on warm paper tones: cream surfaces, white rounded cards with an inner highlight, a split form-plus-product panel, and orange used strictly as the signal for steps, focus and the primary action.

## The look

A large rounded master container sits on a slightly darker paper page. Inside, one side carries a clean form or onboarding step; the other shows the product as a composed visual: a stack of white UI cards, a stat chip, a mini invoice. Depth comes from soft, warm, very light shadows and a 1px white top highlight. Orange appears in small doses, so the page stays calm but alive.

## Suits / avoid

- **Suits:** small SaaS and apps for UMKM (kasir, stok, invoice, booking), sign-up and onboarding screens, membership and course enrolment, laundry or catering order apps, Nomi-built apps' own landing pages.
- **Avoid for:** luxury, editorial, heritage or nightlife brands; anything that needs dark or serious tone.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F7F4EB | Page |
| `$color-foreground` | #1F1A14 | Headings |
| `$color-card` / `-foreground` | #FFFFFF / #1F1A14 | Cards, product panels |
| `$color-secondary` | #FDFBF7 | Master container, input fill |
| `$color-muted` | #FFF5E6 | Illustration panel background |
| `$color-muted-foreground` | #6B645A | Body copy |
| `$color-primary` / `-foreground` | #E65C00 / #FFFFFF | Primary button, step marker, links |
| `$color-primary-pressed` | #CC5200 | Pressed state |
| `$color-accent` / `-foreground` | #FCE8D5 / #8A3A00 | Badge fill, selected row |
| `$color-border` | #EFECE5 | Borders, dividers |
| `$color-input` | #E2DDD2 | Input borders |
| `$color-ring` | #E65C00 | Focus |
| `$color-success` / `-foreground` | #E7F4EA / #1E6B3A | "Lunas", saved |
| `$color-destructive` | #C2361B | Errors |

Radii: `$radius-sm` 6 · `$radius-md` 8 · `$radius-lg` 12 · `$radius-xl` 16 · `$radius-2xl` 32; buttons 999. Fonts: `$font-heading` and `$font-body` = Plus Jakarta Sans (or Inter, as the creator used).

## Type

- **Inter** 400/500/600 as the creator did, or **Plus Jakarta Sans** for a local, slightly warmer voice (designed in Jakarta; a good default for Indonesian SaaS).
- Display 44–56px, 600, letter spacing −3%. Section 28px, 600.
- Body 15–16px/24, 400, `$color-muted-foreground`. Micro labels 10–11px, 500, tracking 6%, `$color-primary` or `$color-muted-foreground`.

## Shape, spacing, depth

- Master container cornerRadius 32, padding 40 (20 phone), stroke 1 `$color-border`.
- Cards radius 16; inputs radius 12; buttons radius 999.
- Card depth: inner shadow y 1, blur 0, #FFFFFFCC + drop shadow y 4, blur 20, #00000005.
- Featured card: drop shadow y 8, blur 40, #E65C0008 + y 1, blur 3, #00000005.
- Primary button glow: drop shadow y 4, blur 14, #E65C004D.
- Gap 16 in forms, 24 between cards, 80–96 between sections.

## Layout signatures

1. **Split onboarding card:** master container horizontal, two children fill_container, gap 40. Left: step marker ("Langkah 2 dari 3" with three 4px bars, filled ones `$color-primary`), heading, form fields, primary button. Right: `$color-muted` panel radius 24 holding the product visual.
2. **Product visual:** 2–3 white cards offset vertically (absolute positions, 16–24px overlap), each a real slice of the product: "Pesanan hari ini: 18", a mini invoice "INV-0726 · Rp 249.000 · Lunas", a chip "Stok kopi susu tinggal 6". Labelled "Contoh tampilan" if not real data.
3. **Feature row:** 3 columns, each with a 20px orange icon inline before a 17px heading (no icon tiles), 2-line body.
4. **Pricing:** 2–3 cards, recommended one with stroke 1.5 `$color-primary` and an `$color-accent` badge "Paling sering dipilih".

On phone: the product visual moves below the form and shrinks to one card.

## Components

- Primary: fill `$color-primary`, text #FFFFFF 15px/600, height 46, padding 0 22, radius 999, orange glow shadow. Pressed `$color-primary-pressed`.
- Secondary: no fill, text `$color-foreground`, stroke 1 `$color-input`.
- Input: fill `$color-secondary`, stroke `$color-input`, radius 12, height 46, label 13px/500 above; focus stroke `$color-primary` plus drop shadow 0 0 blur 0 spread 3 #E65C0026.
- Step marker: 24px circle, `$color-primary` fill with white number for current, `$color-accent` for done.

## Copy voice

Friendly, concrete, second person. "Catat pesanan, stok ikut berkurang sendiri." CTA verbs: "Coba gratis 14 hari", "Buat toko". Never "Revolusi bisnis Anda".

## Tells

- Cream page, white cards, nothing pure-white on pure-white.
- Orange appears only on the step marker, focus ring, primary button and a few icons.
- Split card: real form left, composed product slices right.
- Soft orange-tinted glow under the primary button only.
- Generous radius (16–32) but crisp hairline borders keep it from going bubbly.

## Note for Koda

The source uses a gradient border on the master container and gentle floating of the product cards. Canvas uses a 1px `$color-border` stroke and static offsets; Koda may add a 1px warm gradient border and a slow 4–6px float on the product cards.

Adapted from mengto/skills: agent-skills/web-design/orange-clean-paper-saas (MIT, Copyright (c) 2026 Meng To).
