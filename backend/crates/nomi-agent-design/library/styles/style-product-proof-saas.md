# Style: Product Proof SaaS

The product is the evidence: a friction-led headline beside a dense, honest product stage showing one complete workflow, on white-to-pale-blue bands with soft grid lines, near-black type, black primary buttons and one cool signal blue for state.

## The look

Left: one sentence naming the customer's pain, a short line, a black button. Right: a real or clearly labelled sample of the app doing the job, denser than the copy around it: input, processing, draft, edit, approve, done. Thin borders, 12–16px radii, mono labels. Blue appears only where something is active, selected or in progress.

## Suits / avoid

- **Suits:** small SaaS and apps for UMKM (kasir, stok, invoice, booking, WhatsApp order inbox), tools a Nomi user builds and sells, school or clinic admin apps.
- **Avoid for:** services without software, lifestyle brands, anything where a screenshot is not the point.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #F8FBFF | Page |
| `$color-foreground` | #111522 | Text |
| `$color-card` / `-foreground` | #FFFFFF / #111522 | Product stage, cards |
| `$color-muted` | #EEF3FF | Band 2, table header |
| `$color-secondary` | #DFEEFF | Band 3, footer, secondary fill |
| `$color-muted-foreground` | #667085 | Body, labels |
| `$color-primary` / `-foreground` | #111522 / #FFFFFF | Primary actions (black) |
| `$color-accent` / `-foreground` | #E7ECFE / #2A3FBF | Selected row, active tab |
| `$color-border`, `$color-input` | #11152221 | Borders (13%) |
| `$color-ring`, `$color-signal` | #405CF5 | Focus, progress, selection, "processing" |
| `$color-success` / `-foreground` | #E6F6EC / #1B7A43 | Approved, paid |
| `$color-warning` / `-foreground` | #FFF4E0 / #9A5B00 | Quota, stale |
| `$color-destructive` | #D92D20 | Errors |

Radii: `$radius-sm` 6 · `$radius-md` 8 · `$radius-lg` 12 · `$radius-xl` 16 · `$radius-2xl` 20. Fonts: `$font-heading` and `$font-body` Geist (high x-height grotesk), custom `$font-mono` Geist Mono.

## Type

- Hero 56–94px, 800, letter spacing −7%, line height 0.95, max 3 lines; lede 18px, max 44 characters.
- Section 40px/700. Body 16px/24. UI text inside the stage 13–14px.
- Mono labels 10–11px uppercase, tracking 10%.

## Layout signatures

1. **Hero:** 12-column grid; copy columns 1–5, product stage columns 6–12. Stage: `$color-card`, radius 16, stroke 1, drop shadow y 12 blur 32 #1115220F, padding 20.
2. **Stage contents (one complete outcome):** a header with status ("● Siap", `$color-signal` dot), an input area ("Pesanan dari WhatsApp: 2 kopi susu, 1 roti bakar, antar ke Jl. Kaliurang"), a processing step, the result (parsed order card, total "Rp 58.000"), an edit control and an approve button. A mono tag "CONTOH ALUR" in the corner unless it is real data.
3. **Friction proof:** before/after pair or a time breakdown the person gave you ("Rekap harian: 40 mnt → 5 mnt"), never invented.
4. **Workflow strip:** 5–6 states as a horizontal row of small frames with arrows, each with its state name.
5. **Audience tabs:** "Warung · Kafe · Katering", each showing the same stage with different sample data.
6. **Pricing and FAQ:** aligned comparison with clear units, FAQ terms linked from the table.
7. **Background bands:** solid `$color-background`, then `$color-muted`, then `$color-secondary` at the footer. Optional 1px grid lines #11152210 every 80px in the hero only.

Draw the stage's loading, empty, quota, error and success frames too.

## Components

- Primary: fill `$color-primary`, height 44, radius 10, 15px/600. Secondary: stroke 1 `$color-border`.
- Feature module: 16px line icon in flow before a 17px title, 2 lines of body; no tiles.
- Progress: 4px track `$color-muted`, fill `$color-signal`.

## Copy voice

Names the friction, then the mechanism. "Pesanan WhatsApp masuk berantakan? Nomi Kasir merapikannya jadi daftar yang tinggal dicentang." No "AI ajaib", no fake speed.

## Tells

- A dense, working product stage as the hero image.
- Explicit labelled states: input, processing, draft, approved.
- Black buttons; blue reserved for state and selection.
- Mono labels and a "CONTOH ALUR" tag on sample data.
- Pale blue bands that step darker toward the footer.

## Note for Koda

The source fades white to pale blue with soft cloud drift; canvas uses stepped bands. Koda may turn them into one gradient. State changes 420–700ms, no fake typing.

Adapted from mengto/skills: agent-skills/web-design/product-proof-saas (MIT, Copyright (c) 2026 Meng To).
