# Video styles, palettes and transitions

Named looks for a short video, the palette families to deal colours from, and the transition menu by energy, mood and position in the story.

Pick by what the viewer should feel first, the content second. Write the style's name and one line on why it fits on line 2 of your storyboard plan. Taste rule 8 still holds: glitch, iridescent particles, warps and dark dramatic looks are for tech trades only; a cafe, salon or batik shop gets photographs, type and calm transitions.

## Eight named styles

| Style | Mood | Fits | Palette | Type | Motion |
|---|---|---|---|---|---|
| **Swiss Pulse** | Clinical, precise | Software, dashboards, metrics, fintech | Near-black #1A1A1A, white, one accent (electric blue #0066FF or amber #FFB300) | Bold grotesk headlines, regular labels, numbers 80-120 px | Grid-locked; counters count up from zero; hard cuts; entries snap fast and stop |
| **Velvet Standard** | Premium, timeless | Luxury, keynotes, investor pieces, a fine-dining or hotel brand | Black, white, one rich accent (navy #1A237E or gold #C9A84C) | Thin sans in caps, wide tracking | Slow sequential reveals with long holds; everything glides |
| **Deconstructed** | Industrial, raw | Tech news, security, developer launches | Dark grey #1A1A1A, rust #D4501E, raw white #F0F0F0 | Heavy industrial type, at angles | Text slams and scrambles into place (tech only) |
| **Maximalist Type** | Loud, kinetic | Big launches, milestones, a sale countdown reel | Red #E63946, yellow #FFD60A, black, white | Type is the picture, 50-80% of the frame | 2-3 s rapid scenes; fast arrivals, hard stops; type over footage |
| **Data Drift** | Futuristic | AI and data products only | Black #0A0A0A with cyan #06B6D4 (its purple stays out of Nomi work) | Thin floating sans | Smooth continuous scale shifts; particles become numbers |
| **Soft Signal** | Intimate, warm | Wellness, personal stories, lifestyle, a home bakery | Amber #F5A623, cream #FFF8EC, dusty rose #C4A3A3, sage #8FAF8C | Humanist serif or handwriting, often lowercase | One element fills the frame; slow drifts, never snaps |
| **Folk Frequency** | Cultural, vivid | Food, community, festive launches (Lebaran, 17 Agustus, Imlek) | Hot pink #FF1493, cobalt #0047AB, sun yellow #FFE000, emerald #009B77 | Bold rounded type, pattern and repetition | Elements bounce and pop in with intentional overshoot |
| **Shadow Cut** | Dark, cinematic | Security, dramatic reveals, investigative pieces | Black #0A0A0A, cold grey #3A3A3A, white, one of blood red #C1121F or toxic green | Sharp angular titles like noir cards | Slow push-ins; elements emerge from darkness; silence before the hit |

**Your own style:** name a mood, pick ground + ink + one accent, one display face and one text face, one primary transition, one easing character (snappy, gliding, bouncy). Write it as a row like the ones above.

## Palette families

Deal three to five colours from one family; map them to tokens (ground `$color-background`, text `$color-foreground`, accent `$color-primary`, labels `$color-muted-foreground`, data `$color-chart-1…5`). The client's colours replace the family.

| Family | Fits | Anchor colours to start from |
|---|---|---|
| **Warm editorial** | Stories, documentaries, case studies, a kopi origin film | Deep teal #264653, sea green #2A9D8F, saffron #E9C46A, sandy orange #F4A261, terracotta #E76F51, cream #F4F1DE |
| **Clean corporate** | Explainers, tutorials, presentations | Ivory #FFFCF2, stone #CCC5B9, charcoal #403D39 / #252422, one accent (orange #EB5E28 or slate blue #3D5A80) |
| **Nature and earth** | Sustainability, outdoor, organic, wellness, farm produce | Olive #606C38, forest #283618, parchment #FEFAE0, caramel #DDA15E, rust #BC6C25 |
| **Dark premium** | Tech, finance, luxury, cinematic (tech or luxury only) | Black #000000, ink navy #14213D, amber #FCA311, light grey #E5E5E5 |
| **Jewel rich** | Events, weddings, high-end venues | Wine #5F0F40, crimson #9A031E, tangerine #FB8B24, deep teal #0F4C5C, cream #FDF0D5 |
| **Monochrome** | Typography-led, serious content | A grey ladder #F8F9FA → #212529, optionally one blue #0466C8 |
| **Pastel soft** | Fashion, beauty, lifestyle | Blush #FFC8DD, pink #FFAFCC, baby blue #BDE0FE, sage #CCD5AE, cream #FEFAE0, tan #D4A373 (skip its lilacs) |
| **Bold energetic** | Launches, social promos, announcements | Amber #FFBE0B, orange #FB5607, magenta #FF006E, blue #3A86FF (skip its violets) |
| **Neon electric** | Gaming, nightlife, Gen Z tech only | Navy #0B132B, teal #5BC0BE, aqua #6FFFE9, pink #F72585 |

On a dark ground, light text reads heavier: drop body weight a step and open the line height slightly.

## Transitions

### By energy

| Energy | Primary | Accent | Length |
|---|---|---|---|
| **Calm** (wellness, brand story, luxury) | Blur crossfade, focus pull | Light leak, circle iris | 0.5-0.8 s |
| **Medium** (corporate, explainer, product) | Push slide, staggered blocks | Squeeze, vertical push | 0.3-0.5 s |
| **High** (promo, sport, music, launch) | Zoom through, overexposure flash | Staggered blocks, gravity drop | 0.15-0.3 s |

### By mood

- **Warm, inviting:** light leak, blur crossfade, focus pull, film burn.
- **Cold, clinical:** squeeze, zoom out, blinds, shutter, grid dissolve.
- **Editorial, magazine:** push slide, vertical push, diagonal split, shutter.
- **Tech:** grid dissolve, staggered blocks, blinds; glitch and chromatic split for tech subjects only.
- **Playful:** elastic push, flip, circle iris, clock wipe.
- **Dramatic:** zoom through, gravity drop, dip to black.
- **Premium:** focus pull, blur crossfade, dip to black.
- **Retro, analog:** film burn, light leak, clock wipe.

### By position in the story

| Position | Use |
|---|---|
| **Opening** | Your most distinctive transition, matching the mood; 0.4-0.6 s |
| **Between related points** | The primary transition, the same each time; about 0.3 s |
| **Topic change** | Something different from the primary (staggered blocks, shutter, squeeze) so the viewer resets |
| **The reveal** | The boldest accent, fastest or most dramatic: spend it once |
| **Wind-down** | Back to gentle: blur crossfade, 0.5-0.7 s |
| **Outro** | Slowest and simplest: crossfade or dip to the ground colour, 0.6-1.0 s |

On the canvas, write the transition at the bottom of each scene frame: `→ next: push left, 0.3 s`.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0), plus HeyGen's HyperFrames (heygen-com/hyperframes, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
