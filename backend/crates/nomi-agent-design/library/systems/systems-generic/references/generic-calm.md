# Generic styles: calm and neutral

Calm, neutral and app-like generic packages (cool blue, ink on white, data UIs) with tokens checked and mapped to Sora's names.

Values are from each package's tokens.css, which wins over its DESIGN.md (the DESIGN.md often repeats bergside's original palette). Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule). "Canvas fonts" are real Google Fonts from taste-type: the packages' own Inter, Poppins, Roboto, Montserrat and system faces are banned by Taste.

## Cool white and cobalt (seven packages)

A blue-tinted white floor (#f5f8ff), navy-black ink, a cobalt accent (#2563eb), 10/16/24 radii, a plain sans. The default "clean SaaS" look. It is safe but generic, so give it a real face and a client colour. The red needs darkening.

**Tokens:** background #f5f8ff · foreground #101828 (17) · card #ffffff · muted #eaf1ff · muted-foreground #636d82 (4.9) · border #d7e0ef · primary #2563eb (4.9 on ground) · primary-foreground #ffffff (5.2) · destructive #d72c32 (4.6) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground → #636d82 so it also holds 4.5 on muted; destructive #ef4444 → #d72c32 (3.5:1).

- **bento**: modular bento card grid. Suits saas, product, agency. Canvas fonts Host Grotesk (tokens: Inter). bergside: primary #fad4c0, surface #fff5e6, text #111827; fonts Inter, JetBrains Mono. The bento grid is the lesson; see taste-layout for the canvas version.
- **contemporary**: current minimal with bento. Suits saas, agency. Canvas fonts Instrument Sans (tokens: Inter). bergside: primary #c800df, surface #ffffff, text #111827; fonts Jost, Overpass Mono.
- **corporate**: brand-aligned enterprise grid. Suits law, fintech, saas. Canvas fonts Hanken Grotesk (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Poppins, Open Sans.
- **flat**: two-dimensional, no effects. Suits saas, retail. Canvas fonts Figtree (tokens: Inter). bergside: primary #f2673c, surface #ffffff, text #111827; fonts Inter, JetBrains Mono.
- **perspective**: isometric depth (flattened). Suits saas, hardware. Canvas fonts Epilogue (tokens: Inter). bergside: primary #00bd7d, surface #ffffff, text #111827; fonts Oswald, Poppins.
- **professional**: polished business-ready. Suits law, clinic, fintech. Canvas fonts Red Hat Display + Red Hat Text (tokens: Inter). bergside: primary #fece14, surface #ffffff, text #111827; fonts Poppins, IBM Plex Mono.
- **simple**: plain and readable. Suits clinic, retail. Canvas fonts Albert Sans (tokens: Inter). bergside: removed from its current catalogue.

## Application

Grey floor, white cards, navy ink, cobalt accent, compact 8/12/18 radii; an app shell. The grey needs one step.

**Tokens:** background #f6f7f9 · foreground #172033 (15) · card #ffffff · muted #eef4ff · muted-foreground #657083 (4.7) · border #d8dee8 · primary #2563eb (4.8 on ground) · primary-foreground #ffffff (5.2) · destructive #dc2626 (4.5) · radius 8/12/18 · type 16/18/22/30/42/58 · body line 1.5 · display tracking -0.015em
**Fixed:** muted-foreground #6b7689 → #657083 (4.3:1).

- **application**: app shell, compact. Suits saas. Canvas fonts Onest (tokens: Inter). bergside: removed from its current catalogue.

## Dashboard

Grey-blue floor, white panels, ink, a sky accent (#0ea5e9) with a dark label, 8/12/18 radii. Sky fails 3:1 on the ground (Taste *): ink buttons, sky for chart and status marks.

**Tokens:** background #f4f7fb · foreground #111827 (17) · card #ffffff · muted #eef6ff · muted-foreground #617188 (4.6) · border #d8e2ee · primary #111827* · primary-foreground #f4f7fb · brand #0ea5e9 · destructive #d72c32 (4.5) · radius 8/12/18 · type 15/17/22/30/42/56 · body line 1.48 · display tracking -0.015em
**Fixed:** muted-foreground #64748b → #617188 (4.4:1); primary #0ea5e9 is 2.6:1 on the ground (under 3): buttons and links take the ink, #0ea5e9 becomes $color-brand for small marks and one band (Taste *); destructive #ef4444 → #d72c32 (3.5:1).

- **dashboard**: data panels and charts. Suits saas, fintech. Canvas fonts IBM Plex Sans (tokens: Inter). bergside: removed from its current catalogue.

## Enterprise

Slate floor, white cards, navy ink, a deep blue (#1d4ed8), 6/10/16 radii. Every pair passes on the background; the grey needs one step to hold on muted too.

**Tokens:** background #f8fafc · foreground #0f172a (17) · card #ffffff · muted #eef2f7 · muted-foreground #5e6e85 (5.0) · border #d8dee8 · primary #1d4ed8 (6.4 on ground) · primary-foreground #ffffff (6.7) · destructive #b91c1c (6.2) · radius 6/10/16 · type 16/18/24/34/48/64 · body line 1.52 · display tracking -0.018em
**Fixed:** muted-foreground → #5e6e85 so it also holds 4.5 on muted.

- **enterprise**: cloud-platform data hierarchy. Suits saas, fintech. Canvas fonts IBM Plex Sans (tokens: Inter). bergside: primary #0c5cab, surface #09090b, text #fafafa; fonts IBM Plex Sans. bergside makes it dark (#09090B); open-design made it light.

## Modern

Grey-white floor, ink, an indigo accent (#4f46e5), 10/16/24 radii. The indigo sits at hue 277, so Taste moves it to blue.

**Tokens:** background #f7f8fc · foreground #111827 (17) · card #ffffff · muted #eef1ff · muted-foreground #656c7a (5.0) · border #dfe3ed · primary #0064c4 (5.5 on ground) · primary-foreground #ffffff (5.8) · destructive #d72c32 (4.6) · radius 10/16/24 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground → #656c7a so it also holds 4.5 on muted; primary #4f46e5 is violet (OKLCH hue 277) → #0064c4; destructive #ef4444 → #d72c32 (3.5:1).

- **modern**: clean indigo SaaS. Suits saas, devtool. Canvas fonts Geist (tokens: Inter). bergside: primary #553f83, surface #553f83, text #ffffff; fonts IBM Plex Serif, JetBrains Mono. bergside's Modern is a serif editorial in aubergine.

## Material

Google's grey-white, #202124 ink, blue (#1a73e8), 4/12/24 radii, layered surfaces. It passes, but its faces are on Taste's banned list.

**Tokens:** background #f8fafd · foreground #202124 (15) · card #ffffff · muted #e8f0fe · muted-foreground #5f6368 (5.8) · border #dadce0 · primary #1a73e8 (4.3 on ground) · primary-foreground #ffffff (4.5) · destructive #d93025 (4.6) · radius 4/12/24 · type 16/18/24/32/48/64 · body line 1.5
**Checked:** every pair passes as shipped.

- **material**: Google layered surfaces. Suits saas, school. Canvas fonts Figtree (tokens: Google Sans / Roboto). bergside: primary #6442d6, surface #ffffff, text #111827; fonts Roboto, Inter. Google Sans and Roboto are excluded by Taste.

## Ant

White, #1f1f1f ink, a red accent (#d32029) and compact 6/10/16 radii for data-dense enterprise tables. Passes on the background; the grey needs one step on muted.

**Tokens:** background #ffffff · foreground #1f1f1f (16) · card #f7f8fa · muted #fff1f0 · muted-foreground #667083 (5.0) · border #d9dce3 · primary #d32029 (5.2 on ground) · primary-foreground #ffffff (5.2) · destructive #cf1322 (5.6) · radius 6/10/16 · type 16/18/22/32/48/64 · body line 1.52 · display tracking -0.018em
**Fixed:** muted-foreground → #667083 so it also holds 4.5 on muted.

- **ant**: data-dense enterprise tables. Suits saas, fintech. Canvas fonts Hanken Grotesk (tokens: Ant Sans). bergside: primary #1677ff, surface #ffffff, text #111827; fonts Plus Jakarta Sans, JetBrains Mono. bergside gives it blue (#1677FF), Ant's real primary.

## Shadcn

White, ink, black primary, 6/8/12 radii, Geist: the plain shadcn default. It is Koda's starting point, so Sora always moves it to the client's palette.

**Tokens:** background #ffffff · foreground #111827 (18) · card #ffffff · muted #ffffff · muted-foreground #64748b (4.8) · border #e5e7eb · primary #000000 (21 on ground) · primary-foreground #ffffff (21) · destructive #dc2626 (4.8) · radius 6/8/12 · type 16/20/24/32/40/48 · body line 1.5 · display tracking -0.02em
**Checked:** every pair passes as shipped.

- **shadcn**: monochrome utility components. Suits saas, devtool. Canvas fonts Geist (tokens: Geist). bergside: primary #000000, surface #ffffff, text #111827; fonts Geist, Fira Code.

## Ink on white (four packages)

White, #f7f7f7 surfaces, #111111 ink as both text and primary, 4/8/12 radii (minimal uses 2/4/8). The purest Taste-friendly neutral: add one client accent and it is a full system.

**Tokens:** background #ffffff · foreground #111111 (19) · card #f7f7f7 · muted #eeeeee · muted-foreground #6a6a6a (5.4) · border #d9d9d9 · primary #111111 (19 on ground) · primary-foreground #ffffff (19) · destructive #c53030 (5.5) · radius 4/8/12 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground → #6a6a6a so it also holds 4.5 on muted.

- **bold**: heavyweight type, ink on white. Suits agency, retail. Canvas fonts Archivo (tokens: Arial Black / Inter). bergside: primary #0077bc, surface #111111, text #111827; fonts Archivo Black, JetBrains Mono. bergside: blue #0077BC and green; tokens chose ink.
- **clean**: whitespace and one ink. Suits clinic, law, studio. Canvas fonts Instrument Sans (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Poppins, Roboto.
- **minimal**: stripped back, 2/4/8 radii. Suits studio, agency, law. Canvas fonts Host Grotesk (tokens: Inter). bergside: primary #0c0c09, surface #f4f4f1, text #0c0c09; fonts Inter, Open Sans.
- **mono**: monospace-driven compact. Suits devtool. Canvas fonts Geist + Geist Mono (tokens: IBM Plex Mono). bergside: primary #37f712, surface #e7e5e4, text #78716b; fonts Space Mono, JetBrains Mono. Mono only when the page shows code or technical data.

## Publication

White, black, a news red (#c1121f), zero radius, Franklin-style headlines over a serif body. All pairs pass.

**Tokens:** background #ffffff · foreground #0b0b0b (20) · card #f6f6f6 · muted #fff2f0 · muted-foreground #666666 (5.7) · border #d6d6d6 · primary #c1121f (6.2 on ground) · primary-foreground #ffffff (6.2) · destructive #b00020 (7.3) · radius 0/0/0 · type 17/20/28/42/64/88 · body line 1.58 · display tracking -0.018em
**Checked:** every pair passes as shipped.

- **publication**: news page, square, red. Suits school, community. Canvas fonts Source Serif 4 + Libre Franklin (tokens: Franklin Gothic / Georgia). bergside: removed from its current catalogue.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside; skills authored with typeui.sh). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
