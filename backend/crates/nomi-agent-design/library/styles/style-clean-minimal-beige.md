# Style: Clean Minimal Beige

Calm, warm, process-led light mode: layered beige and stone surfaces, hairline borders, one centred master shell, an even grid of process columns and a single quiet accent, for services that want to feel organised and trustworthy rather than loud.

## The look

Everything sits on warm neutrals that differ by only a few steps, so hierarchy comes from tone, thin strokes and spacing, not shadows. A simple centred hero (small badge, heading, one or two buttons) leads into a framed shell holding equal process columns or modules. One dark, deep-green inset card appears once or twice as a contrast moment. It reads premium and quiet, like a well-run clinic reception.

## Suits / avoid

- **Suits:** tutors and mentoring, clinics and therapists, consultants, kos and guesthouses explaining a process, cafés with a calm brand, course sign-ups, appointment flows.
- **Avoid for:** loud promos, street food, nightlife, data-heavy dashboards (it turns sleepy), brands that need high energy.

## Tokens

| Token | Light | Dark | Role |
|---|---|---|---|
| `$color-background` | #F3EEE6 | #191612 | Page |
| `$color-foreground` | #1F1B16 | #F3EEE6 | Headings, primary text |
| `$color-card` / `-foreground` | #FBF7F0 / #1F1B16 | #2A251F / #F3EEE6 | Cards inside the shell |
| `$color-secondary` | #F8F4ED | #211D18 | Master shell, secondary button |
| `$color-muted` | #E4DBCD | #332D26 | Inner dividers, progress track |
| `$color-muted-foreground` | #62574B | #C9BFB2 | Body copy, labels |
| `$color-primary` / `-foreground` | #1F1B16 / #F8F4ED | #F3EEE6 / #1F1B16 | Primary button |
| `$color-accent` / `-foreground` | #ECE3D6 / #1F1B16 | #3B342C / #F3EEE6 | Selected row, hover |
| `$color-border`, `$color-input` | #D8CEBE | #3B342C | Hairlines, inputs |
| `$color-ring`, `$color-signal` | #5C3822 | #C08A62 | Focus, active dot, progress, one badge |
| `$color-inset` / `-foreground` | #2E3A2F / #F3EEE6 | same | Rare dark inset card |
| `$color-destructive` | #A33A2B | #E0705F | Errors |

Radii: `$radius-sm` 8 · `$radius-md` 12 · `$radius-lg` 16 · `$radius-xl` 24 · `$radius-2xl` 28; buttons and pills 999. Fonts: `$font-heading` and `$font-body` = Geist (or Figtree).

## Type

- Creator used Inter; Geist or Figtree keeps the weights without the default look.
- Display 48–64px, weight 500, letter spacing −2%, line height 1.05.
- Section heading 28–32px, 500. Body 16px/26, 400, `$color-muted-foreground`.
- Labels 11px uppercase, tracking 18%, 500, `$color-muted-foreground`. Use them sparingly: one per section at most.

## Shape, spacing, depth

- Master shell cornerRadius 28, stroke 1 `$color-border`, padding 32 (16 on phone).
- Cards cornerRadius 24 (large) or 16 (small); pills and buttons radius 999.
- Inner highlight instead of shadow: inner shadow y 1, blur 0, #FFFFFFBF on shells and buttons. Optional outer shadow y 1, blur 0, #725D480A.
- Gap 12 inside groups, 24 between modules, 96 between sections (64 on phone).
- No drop shadows heavier than that.

## Layout signatures

1. **Centred hero:** frame vertical, alignItems center, gap 20. Badge pill (`$color-card`, stroke `$color-border`, 6px `$color-signal` dot + "Kelas dibuka 26 Jul 2026"), heading max 14 words, body max 2 lines, button row gap 12.
2. **Framed shell below the hero:** one frame width fill_container, max 1200, holding a horizontal row of 3–4 equal columns (each fill_container), separated by 1px vertical `$color-muted` rectangles instead of gaps between cards.
3. **Process column:** number or small label at top, 20px heading, 2-line body, then a tiny functional mock: a progress bar (track `$color-muted` 4px tall, fill `$color-signal`), a status row with a dot, or a mini approval card.
4. **Dark inset moment:** one `$color-inset` card (radius 24, text #F3EEE6) inside the light shell, e.g. the next available slot.

On phone: shell becomes a vertical stack; column dividers turn into horizontal 1px lines.

## Components

- Primary button: fill `$color-primary`, text `$color-primary-foreground` 15px/500, height 44, padding 0 20, radius 999.
- Secondary: fill `$color-card`, stroke 1 `$color-border`, text `$color-foreground`.
- Status dot: 6px ellipse `$color-signal`, paired with 13px text.
- Input: fill #FFFFFF99, stroke `$color-border`, radius 12, height 44; focus stroke `$color-signal`.

## Copy voice

Plain, reassuring, step-by-step. "Cari mentor yang cocok, lalu mulai sesi 1:1." Steps named as verbs: "Pilih bidang", "Cari mentor", "Mulai belajar". Prices with unit: "Rp 249.000 / sesi".

## Tells

- Three or four tones of beige within 6% lightness of each other carrying all the hierarchy.
- Equal process columns divided by hairlines inside one big rounded shell.
- A single deep-green dark card in an otherwise light page.
- Accent used only as a 6px dot, a progress fill or one badge.
- Inner 1px white highlight on raised surfaces instead of drop shadows.

## Note for Koda

The source puts a soft painted wash behind the hero; on canvas it is flat `$color-background`. Koda may add a very low-contrast radial wash and masked text fade-ins; nothing more.

Adapted from mengto/skills: agent-skills/web-design/clean-minimal-beige-light-mode (MIT, Copyright (c) 2026 Meng To).
