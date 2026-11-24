---
name: style-documentary-brutalist
description: "Hard, honest and loud: compressed billboard headlines, black and warm-white chapters cut hard against each other, square corners, exposed hairline grid, and real documentary photos as the proof, with pale mint and pink only on verified quotes."
license: "MIT, Copyright (c) 2026 Meng To"
source: "https://github.com/mengto/skills"
---
# Style: Documentary Brutalist

Hard, honest and loud: compressed billboard headlines, black and warm-white chapters cut hard against each other, square corners, exposed hairline grid, and real documentary photos as the proof, with pale mint and pink only on verified quotes.

## The look

The first screen is a black field with two lines of compressed type at billboard scale and a few documentary photo frames layered through the middle. Then a hard cut to warm white for a short philosophy chapter. Work sits on black. Testimonials sit on pale mint or pink cards. The page ends with a giant wordmark. No glow, no gradients, no rounded SaaS cards.

## Suits / avoid

- **Suits:** barbershops, tattoo studios, streetwear and distro brands, event organisers, music venues, skate and bike shops, coffee roasters with attitude, community collectives, documentary photographers.
- **Avoid for:** clinics, finance, older or cautious audiences, anything that needs softness or reassurance.

## Tokens

| Token | Hex | Role |
|---|---|---|
| `$color-background` | #0B0B0A | Black chapters |
| `$color-foreground` | #F2EFE6 | Text on black |
| `$color-card` / `-foreground` | #F2EFE6 / #0B0B0A | Warm white chapters |
| `$color-muted-foreground` | #9A968C | Secondary text on black |
| `$color-primary` / `-foreground` | #F2EFE6 / #0B0B0A | Buttons on black (inverted on white) |
| `$color-border` | #F2EFE647 | Hairlines on black (on white use #0B0B0A33) |
| `$color-proof-1` | #C8DFCF | Mint proof card |
| `$color-proof-2` | #E8BBC4 | Pink proof card |
| `$color-ring` | #F2EFE6 | Focus |
| `$color-placeholder` | #3A3630 | Photo placeholder |

Radii: all 0. Only the round project action (52px) is a circle. Fonts: `$font-heading` Anton (creator used Impact; Bebas Neue also works), `$font-body` Archivo 400/500.

## Type

- Hero: 96–206px, uppercase, line height 0.86, letter spacing −2%, exactly two lines; test the break on the phone frame (64–72px there).
- Chapter titles 50–104px. Work titles 40–64px.
- Labels 10–11px uppercase, tracking 13%. Body 15px/22; philosophy paragraph 22px/32, max 3 sentences.

## Layout signatures

1. **Header:** compact wordmark left, current section centre ("KARYA"), "MENU" right. Bottom hairline.
2. **Hero:** black, exposed vertical hairlines every 3 columns. Headline across the width; 3 documentary photo placeholders overlapping its middle at different sizes and offsets, captioned ("Foto: potong rambut jam 9 malam, Kemang"); headline stays readable. Small positioning line and a 52px circle action at the bottom.
3. **Hard cut:** warm-white philosophy chapter, one large paragraph, lots of space, no images.
4. **Work on black:** large title, role label, documentary media; rows divided by hairlines.
5. **Collage:** irregular grid of photo frames, varied scale, keeping reading order left to right, top to bottom.
6. **Proof:** mint or pink cards (radius 0, padding 32) with real quotes and full attribution only.
7. **FAQ:** hard-ruled rows with + marks. **Footer:** wordmark at about 25% of frame width tall, contact line "halo@cukurkeras.example".

## Components

- Button: height 48, radius 0, stroke 1, uppercase 12px/500 tracking 10%.
- Menu: full-screen black overlay frame with 64px uppercase links.
- Circle action: 52px ellipse, stroke 1 `$color-foreground`, arrow icon.

## Copy voice

Short, declarative, a bit blunt, always true. "POTONG RAPI. NGOBROL JUJUR." Dates and places on everything.

## Tells

- Two lines of compressed uppercase type at billboard size.
- Hard black-to-warm-white cuts with no transition.
- Exposed hairline grid and square corners throughout.
- Documentary photo collage as proof, not decoration.
- Pale mint and pink appear only behind real testimonials.

## Note for Koda

Collage parallax under 5%; sticky contrast changes at chapter edges; 500–760ms masks. No floating, no smooth-scroll theatre.

Adapted from mengto/skills: agent-skills/web-design/documentary-brutalist-agency (MIT, Copyright (c) 2026 Meng To).
