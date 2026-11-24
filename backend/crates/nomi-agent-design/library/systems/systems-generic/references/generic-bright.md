# Generic styles: bright and playful

Bright, loud and game-flavoured generic packages with tokens checked, violet swapped and the effects the canvas can't draw flattened.

Values are from each package's tokens.css, which wins over its DESIGN.md (the DESIGN.md often repeats bergside's original palette). Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule). "Canvas fonts" are real Google Fonts from taste-type: the packages' own Inter, Poppins, Roboto, Montserrat and system faces are banned by Taste.

## Butter yellow and hot orange (six packages)

A pale yellow floor (#fff8d7), navy-violet ink, a hot orange (#ff6b00), 10/16/24 radii. Playful and loud. The orange fails under a white label and on the ground, so it is darkened to a burnt orange. The ink leans violet, so Taste moves it to navy.

**Tokens:** background #fff8d7 · foreground #091e37 (16) · card #ffffff · muted #ffef9f · muted-foreground #706688 (5.0) · border #eadfba · primary #c55409 (4.2 on ground) · primary-foreground #ffffff (4.5) · destructive #d1343e (4.6) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** foreground #1d1836 is violet (hue 289) → #091e37; muted-foreground #796f91 → #766c8e (4.4:1); muted-foreground → #706688 so it also holds 4.5 on muted; primary #ff6b00 → #c55409 so its label #ffffff holds 4.5 (2.9:1); destructive #e5484d → #d1343e (3.7:1).

- **colorful**: vibrant multicolour. Suits product, studio. Canvas fonts Funnel Display + Funnel Sans (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Inter, JetBrains Mono.
- **creative**: character-driven, big type. Suits agency, studio. Canvas fonts Bricolage Grotesque + Figtree (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Bangers, IBM Plex Mono.
- **doodle**: hand-drawn and informal. Suits school, cafe. Canvas fonts Gabarito + Figtree (tokens: Comic Sans MS). bergside: primary #49b6e5, surface #ffffff, text #111827; fonts Delius Swash Caps, JetBrains Mono. Comic Sans is in the tokens; never on a client page.
- **energetic**: high-energy sport and promo. Suits wellness, retail. Canvas fonts Archivo (tokens: Inter). bergside: removed from its current catalogue.
- **friendly**: approachable, rounded. Suits school, cafe, wellness. Canvas fonts Nunito (tokens: Inter). bergside: primary #f2d9dc, surface #ffffff, text #111827; fonts Noto Serif Display, Space Mono. bergside means soft pastels (#F2D9DC); the tokens are the loud set.
- **vibrant**: lively bold colour. Suits product, studio. Canvas fonts Funnel Display + Funnel Sans (tokens: Inter). bergside: primary #7c61d4, surface #ffffff, text #2f281d; fonts Fascinate, Noto Sans.

## Brutalism

Raw cream (#f5f1e8), pure black ink and borders, a lemon fill (#ffef5a) with a black label, zero radius, heavy display. The lemon is a fill, never a link (Taste *).

**Tokens:** background #f5f1e8 · foreground #000000 (19) · card #ffffff · muted #ffef5a · muted-foreground #555555 (6.6) · border #000000 · primary #000000* · primary-foreground #f5f1e8 · brand #ffef5a · destructive #db0014 (4.6) · radius 0/0/0 · type 17/20/28/42/64/88 · body line 1.35
**Fixed:** primary #ffef5a is 1.0:1 on the ground (under 3): buttons and links take the ink, #ffef5a becomes $color-brand for small marks and one band (Taste *); destructive #ff2b2b → #db0014 (3.3:1).

- **brutalism**: raw concrete, black borders, zero radius. Suits agency, studio. Canvas fonts Archivo (tokens: Arial Black / Arial). bergside: primary #dd614c, surface #ffffff, text #111827; fonts Darker Grotesque, JetBrains Mono.

## Pacman

Black ground, cream text, arcade yellow (#ffcc00) with a black label, maze-blue borders, pixel display face. Game reference only.

**Tokens:** background #050505 · foreground #fff7d6 (19) · card #101014 · muted #1f1b08 · muted-foreground #b9a85d (8.6) · border #2338ff · primary #ffcc00 (13 on ground) · primary-foreground #050505 (13) · destructive #ff3b3b (5.8) · radius 10/18/9999 · type 14/16/20/30/44/64 · body line 1.6
**Checked:** every pair passes as shipped.

- **pacman**: arcade maze, pixel type. Suits game. Canvas fonts Anybody + Onest (tokens: Press Start 2P / Inter). bergside: primary #2a3fe5, surface #000000, text #111827; fonts Press Start 2P, Space Mono. Press Start 2P only as a game logo.

## Tetris

Navy-black ground, white text, cyan (#00f0f0) with a dark label, 0/0/4 radii, pixel display face. Game reference only.

**Tokens:** background #050816 · foreground #f8fafc (19) · card #10162a · muted #17203a · muted-foreground #94a3b8 (7.8) · border #26324f · primary #00f0f0 (14 on ground) · primary-foreground #061018 (13) · destructive #f40e08 (4.7) · radius 0/0/4 · type 14/16/20/28/40/56 · body line 1.6
**Fixed:** destructive #f00000 → #f40e08 (4.5:1).

- **tetris**: block-game neon on navy. Suits game. Canvas fonts Anybody + Onest (tokens: Press Start 2P / Inter). bergside: primary #1c202b, surface #dfe7ff, text #1c398e; fonts Bangers, JetBrains Mono.

## Lingo

Lilac-white floor, ink, an electric violet (#6d4aff) and 10/16/24 radii. A language-app look. Taste swaps the violet to blue.

**Tokens:** background #fbf9ff · foreground #1d1b2a (16) · card #ffffff · muted #f1ecff · muted-foreground #6f6686 (5.1) · border #ded7f0 · primary #0172dd (4.5 on ground) · primary-foreground #ffffff (4.7) · destructive #d1343e (4.7) · radius 10/16/24 · type 16/18/24/34/50/70 · body line 1.55 · display tracking -0.025em
**Fixed:** muted-foreground #786f8f → #756c8c (4.5:1); muted-foreground → #6f6686 so it also holds 4.5 on muted; primary #6d4aff is violet (OKLCH hue 284) → #0172dd; destructive #e5484d → #d1343e (3.7:1).

- **lingo**: playful language-app. Suits school. Canvas fonts Nunito (tokens: Inter). bergside: primary #58cc02, surface #ffffff, text #3c3c3c; fonts Nunito, JetBrains Mono. bergside's own Lingo is owl green (#58CC02), close to the Duolingo card.

## Expressive

Lilac-white floor, violet-ink, a vivid violet (#7c3aed). Taste swaps both to blue-navy; the lesson is the big personality type, not the hue.

**Tokens:** background #fbf8ff · foreground #0c1d32 (16) · card #ffffff · muted #efe8ff · muted-foreground #716582 (5.1) · border #ded4ee · primary #0d6dd0 (4.9 on ground) · primary-foreground #ffffff (5.1) · destructive #d72c32 (4.6) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** foreground #1f1730 is violet (hue 297) → #0c1d32; muted-foreground → #716582 so it also holds 4.5 on muted; primary #7c3aed is violet (OKLCH hue 293) → #0d6dd0; destructive #ef4444 → #d72c32 (3.6:1).

- **expressive**: personality-driven bold colour. Suits agency, studio. Canvas fonts Bricolage Grotesque + Figtree (tokens: Inter). bergside: primary #db2777, surface #ffffff, text #111827; fonts IBM Plex Mono.

## Gradient

Lilac floor, violet accent and round 12/20/32 radii. Gradients can't be drawn on the canvas: use the solid accent and note the intended gradient for Koda only on tech.

**Tokens:** background #f7f3ff · foreground #191225 (17) · card #ffffff · muted #efe7ff · muted-foreground #6e637f (5.1) · border #ddd2f2 · primary #0d6dd0 (4.7 on ground) · primary-foreground #ffffff (5.1) · destructive #d3272f (4.7) · radius 12/20/32 · type 16/19/26/40/62/86 · body line 1.52 · display tracking -0.03em
**Fixed:** muted-foreground → #6e637f so it also holds 4.5 on muted; primary #7c3aed is violet (OKLCH hue 293) → #0d6dd0; destructive #ef4444 → #d3272f (3.4:1).

- **gradient**: gradient-rich surfaces (flattened). Suits ai, product. Canvas fonts Funnel Display + Funnel Sans (tokens: Inter). bergside: primary #990ffa, surface #ffffff, text #111827; fonts Space Grotesk, Montserrat.

## Glassmorphism

Ice-blue floor (#eef6ff), navy ink, sky-blue accent (#4f8cff) and very round 16/24/36 radii. Blur and glass don't exist on the canvas: draw solid white cards with a white hairline. The accent needs darkening.

**Tokens:** background #eef6ff · foreground #102033 (15) · card #ffffff · muted #eef6ff · muted-foreground #60708a (4.6) · border #ffffff · primary #3471e4 (4.2 on ground) · primary-foreground #ffffff (4.5) · destructive #d3272f (4.7) · radius 16/24/36 · type 16/18/24/36/54/76 · body line 1.55 · display tracking -0.025em
**Fixed:** primary #4f8cff → #3471e4 so its label #ffffff holds 4.5 (3.2:1); destructive #ef4444 → #d3272f (3.5:1).

- **glassmorphism**: frosted layers (flattened). Suits saas, ai. Canvas fonts Instrument Sans (tokens: Inter). bergside: primary #1856ff, surface #ffffff, text #141414; fonts Plus Jakarta Sans, JetBrains Mono.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside; skills authored with typeui.sh). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
