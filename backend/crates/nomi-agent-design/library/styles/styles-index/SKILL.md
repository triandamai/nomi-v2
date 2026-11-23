---
name: styles-index
description: "The named visual styles you can offer as directions, one line each (style → suits → look), plus how to pick 2–4 that are clearly different for a brief; read this before proposing directions, then read the chosen style files."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Styles Index

The named visual styles you can offer as directions, one line each (style → suits → look), plus how to pick 2–4 that are clearly different for a brief; read this before proposing directions, then read the chosen style files.

## The styles

| Topic | Suits | Look in one line |
|---|---|---|
| `style-clean-minimal-beige` | Tutors, clinics, consultants, kos, calm cafés | Warm beige tones, hairline shell, equal process columns, one dark green inset |
| `style-editorial-service-booking` | Salons, barbers, spas, clinics, homestays | Ivory paper, huge tight serif, hairline grid, ruled treatment list, no cards |
| `style-orange-paper-saas` | Small SaaS, UMKM apps, sign-up and onboarding | Cream paper, white rounded cards, split form + product slices, orange signal |
| `style-book-serif-index` | Menu books, craft catalogues, heritage, writers, course notes | Dark catalogue shell around an open paper book, serif pages, mono index |
| `style-image-first-grid` | Villas, homestays, tours, venues, property | Full-bleed photo stage, scrim, rails over the image, copy anchored low |
| `style-portfolio-chapters` | Photographers, designers, videographers, MUAs | Near-black frame, one muted colour field per project, billboard grotesk |
| `style-split-technical` | Kos rooms, used vehicles, product and event detail | 50/50 framed panels: photo window vs mono spec rows |
| `style-framed-grid` | Price lists, bengkel and laundry menus, schedules, reports | 1px frames on a strict 12-col grid, L-brackets, light display type, one red dot |
| `style-nested-agency` | Agencies, consultants, WO and photo packages | White shell with rails, deep-navy rounded block, inset package cards |
| `style-agency-grid-minimal` | Architects, interiors, contractors, furniture | Huge tight headlines, tiny caps lanes, panoramic image, ruled service rows |
| `style-documentary-brutalist` | Barbershops, streetwear, tattoo, events, venues | Compressed billboard caps, hard black/white cuts, documentary collage |
| `style-tactile-hardware` | Kasir/POS, calculators, queue screens, IoT panels | Moulded surfaces lit from above, inset wells, one status light |
| `style-neoclassical-ledger` | Bookkeeping, notaries, koperasi, jamu, printers | Parchment ledger, giant sans chapter words, serif notes, object per service |
| `style-field-guide` | Nurseries, farms, eco-tours, coffee origin, honey | Pale sage paper, thin big Lexend, vertical guides, specimen-captioned figures |
| `style-product-proof-saas` | SaaS landing pages, admin apps, Nomi-built tools | Friction headline + dense honest product stage, black buttons, blue state |
| `style-tech-green-dark` | Small-team dashboards, monitoring, IT services | Three-step black panels, mono codes, green only for live and active |

Companion files: `style-details` (numbering, container lines, brackets, chamfers, shadow recipes, icons), `pattern-landing-pricing` (page structure and pricing), `slop-audit`, `design-loop`.

## Picking 2–4 directions

1. Shortlist from the **Suits** column for the trade, then add one wildcard that fits the customer rather than the trade (a barbershop can be `style-editorial-service-booking` or `style-documentary-brutalist`).
2. Make the set differ on at least two of these axes, or it is one direction twice:
   - **Value:** light / warm paper / dark.
   - **Type voice:** serif-led / grotesk-led / compressed caps / thin geometric / mono-technical.
   - **Structure:** hairline grid / nested containers / full-bleed image / framed modules / book spread.
   - **Energy:** quiet / confident / loud.
3. Good contrasting sets:
   - Salon: editorial-service-booking · clean-minimal-beige · documentary-brutalist.
   - Café or warung: framed-grid (menu) · book-serif-index (menu as a book) · image-first-grid.
   - Kos or homestay: split-technical · image-first-grid · clean-minimal-beige.
   - Small SaaS or app: product-proof-saas · orange-paper-saas · tech-green-dark (for the app inside).
   - Portfolio: portfolio-chapters · agency-grid-minimal · documentary-brutalist.
   - Consultant or accountant: nested-agency · neoclassical-ledger · clean-minimal-beige.
   - Farm or nature: field-guide · image-first-grid · book-serif-index.
4. Do not offer a style whose needs the person cannot meet: image-first and portfolio need real photos; ledger needs object illustrations; dashboards need real data fields.
5. Present each direction as a drawn first viewport (desktop and phone), its name, one line on why it fits this customer, and its palette chips. Let the person pick; then commit to one style and do not blend two.

## Shared rules for every style

- Colour tokens use shadcn roles with a `color-` prefix (`$color-background`, `$color-foreground`, `$color-card`, `$color-primary`, `$color-muted-foreground`, `$color-border`, `$color-ring`, …). Style files add a few custom roles, always as pairs where text sits on them: `$color-signal` (the one accent that is not the primary button), `$color-chapter` / `-foreground` (a contrasting full-width band), `$color-placeholder` (photo placeholder tone), `$color-sidebar*` (an outer shell).
- Spacing comes from `$space-1`…`$space-12` (4, 8, 12, 16, 24, 32, 48, 64, 96, 128…). Round source values to that scale.
- Each style sets its own `$radius-sm`…`$radius-2xl` and `$font-heading` / `$font-body`; all fonts named are on Google Fonts.
- Fills are solid. Where a style's source relied on gradients, glass or grain, its file says what the canvas draws instead and what Koda may add.
- Photos are tone-filled placeholder frames with a caption naming the photo.
- Run `slop-audit` and score with `design-loop` before saving any version.

Adapted from mengto/skills: agent-skills/web-design (MIT, Copyright (c) 2026 Meng To).
