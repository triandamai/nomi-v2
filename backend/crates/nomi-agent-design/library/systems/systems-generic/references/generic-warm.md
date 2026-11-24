# Generic styles: warm

Warm generic style packages (cream, paper, clay, retro) with tokens checked and mapped to Sora's names, plus what bergside's upstream file said.

Values are from each package's tokens.css, which wins over its DESIGN.md (the DESIGN.md often repeats bergside's original palette). Token names are Sora's `$color-<name>`; the number after a colour is its contrast with `background`, and after primary-foreground its contrast with primary. A value marked * means the accent is under 3:1 on the ground, so buttons and links take the ink and the accent becomes `$color-brand` (Taste's * rule). "Canvas fonts" are real Google Fonts from taste-type: the packages' own Inter, Poppins, Roboto, Montserrat and system faces are banned by Taste.

## Cream paper and walnut (five packages share these tokens)

A warm cream floor with a white card, espresso ink, a walnut-brown accent, soft 10/16/24 radii, a serif display over a sans body, a 1.62 body line and roomy 112px sections. Calm hospitality: the closest generic match to Taste's warm-paper-walnut palette.

**Tokens:** background #fbf6ee · foreground #201914 (16) · card #fffdf8 · muted #f1e3cf · muted-foreground #71645a (5.3) · border #ded2c3 · primary #9b5b32 (5.0 on ground) · primary-foreground #ffffff (5.3) · destructive #b33a3a (5.4) · radius 10/16/24 · type 17/20/28/42/64/88 · body line 1.62 · display tracking -0.025em
**Fixed:** muted-foreground → #71645a so it also holds 4.5 on muted.

- **artistic**: high-contrast expressive type on cream. Suits studio, agency. Canvas fonts Fraunces + Figtree (tokens: Georgia / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Limelight, JetBrains Mono. bergside styles it high-contrast with a display face; the tokens are the shared cream set.
- **cafe**: cozy cream, espresso ink, walnut. Suits cafe, bakery, resto. Canvas fonts Besley + Albert Sans (tokens: Georgia / Inter). bergside: primary #5d4432, surface #f9f7f5, text #3e2b1e; fonts Poppins, JetBrains Mono. The DESIGN.md says Poppins and #5D4432 on #F9F7F5; trust the tokens.
- **elegant**: refined cream and serif. Suits wedding, hotel, studio. Canvas fonts EB Garamond + Jost (tokens: Georgia / Inter). bergside: removed from its current catalogue.
- **refined**: understated serif minimal. Suits law, studio, hotel. Canvas fonts Newsreader + Hanken Grotesk (tokens: Georgia / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Playfair Display, JetBrains Mono.
- **storytelling**: narrative long scroll on cream. Suits studio, wedding, school. Canvas fonts Literata + Figtree (tokens: Georgia / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Abril Fatface, Inter.

## Aged paper and tobacco brown (three packages)

A yellowed paper floor (#faf3df), deep brown ink, a tobacco-brown accent, 10/16/24 radii. Old print, archive and almanac moods. The grey needs one step darker.

**Tokens:** background #faf3df · foreground #211b12 (15) · card #fffaf0 · muted #efe0bd · muted-foreground #6d6251 (5.4) · border #ddd0b4 · primary #8f5d24 (5.0 on ground) · primary-foreground #ffffff (5.6) · destructive #a64036 (5.6) · radius 10/16/24 · type 17/20/28/42/64/88 · body line 1.62 · display tracking -0.025em
**Fixed:** muted-foreground #7c7160 → #796e5d (4.3:1); muted-foreground → #6d6251 so it also holds 4.5 on muted.

- **paper**: print-textured paper (texture dropped). Suits bakery, studio, school. Canvas fonts Vollkorn + Work Sans (tokens: Georgia / Inter). bergside: primary #111111, surface #ffffff, text #111827; fonts Montserrat, Roboto.
- **spacious**: generous whitespace on paper. Suits hotel, wellness, studio. Canvas fonts Newsreader + Hanken Grotesk (tokens: Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Montserrat, Open Sans.
- **vintage**: old-print nostalgia. Suits cafe, workshop, retail. Canvas fonts Zilla Slab + Rethink Sans (tokens: Georgia / Inter). bergside: primary #008080, surface #c0c0c0, text #000000; fonts Silkscreen, JetBrains Mono. bergside means a 90s teal-and-silver desktop; open-design's tokens make it aged paper.

## Editorial cream

Cream floor, warm black ink, a walnut accent, small 4/8/12 radii and a serif body face: a reading page, not an app.

**Tokens:** background #fbf7f0 · foreground #1f1a16 (16) · card #fffdf8 · muted #f1e6d6 · muted-foreground #71655c (5.3) · border #ded3c5 · primary #9a5a2f (5.1 on ground) · primary-foreground #ffffff (5.4) · destructive #b33a3a (5.5) · radius 4/8/12 · type 18/21/30/44/66/92 · body line 1.65 · display tracking -0.02em
**Fixed:** muted-foreground #7d7168 → #7a6e65 (4.4:1); muted-foreground → #71655c so it also holds 4.5 on muted.

- **editorial**: magazine reading page, serif body. Suits studio, school, law. Canvas fonts Source Serif 4 + Libre Franklin (tokens: Georgia / Source Serif Pro). bergside: primary #111111, surface #ffffff, text #111827; fonts Gelasio, Ubuntu Mono.

## Premium ivory

Ivory floor, warm black ink, a bronze accent (#a06a3b) and larger 10/16/28 radii; a quiet luxury page set in a high-contrast serif.

**Tokens:** background #faf8f4 · foreground #1c1b19 (16) · card #ffffff · muted #f0e7d8 · muted-foreground #6e675d (5.3) · border #ded6c9 · primary #a06a3b (4.3 on ground) · primary-foreground #ffffff (4.5) · destructive #b84a4a (4.8) · radius 10/16/28 · type 16/19/26/40/60/84 · body line 1.58 · display tracking -0.02em
**Fixed:** muted-foreground → #6e675d so it also holds 4.5 on muted.

- **premium**: ivory and bronze quiet luxury. Suits hotel, wedding, retail. Canvas fonts Instrument Serif + Instrument Sans (tokens: Canela / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Inter, JetBrains Mono. bergside calls it "Apple-inspired"; the tokens are warmer.

## Soft clay (three packages)

A peach-clay floor, brown ink, a clay accent and very round 14/22/34 radii. The look of these styles (puffy clay, extruded neumorphic, textured skeuomorphic) relies on shadows and textures the canvas can't draw. Keep the palette and the round radii, drop the effects. Grey, accent and red all need fixing.

**Tokens:** background #f7eee6 · foreground #2b211c (14) · card #fff8f1 · muted #ead6c7 · muted-foreground #6c5c52 (5.6) · border #dac8b9 · primary #ab613d (4.1 on ground) · primary-foreground #ffffff (4.7) · destructive #b54949 (4.6) · radius 14/22/34 · type 16/18/24/36/54/76 · body line 1.52 · display tracking -0.025em
**Fixed:** muted-foreground #8a7a70 → #78685e (3.6:1); muted-foreground → #6c5c52 so it also holds 4.5 on muted; primary #b46a46 → #ab613d so its label #ffffff holds 4.5 (4.1:1); destructive #b84c4c → #b54949 (4.4:1).

- **claymorphism**: puffy clay shapes (effects dropped). Suits school, cafe, product. Canvas fonts Nunito (tokens: Inter). bergside: primary #3b82f6, surface #ffffff, text #1c398e; fonts Poppins, Montserrat.
- **neumorphism**: extruded soft UI (effects dropped). Suits wellness, product. Canvas fonts Outfit (tokens: Inter). bergside: primary #006666, surface #e7e5e4, text #1e2938; fonts Space Mono, JetBrains Mono.
- **skeumorphism**: real-world textures (effects dropped). Suits workshop, retail. Canvas fonts Zilla Slab + Rethink Sans (tokens: Georgia / Inter). bergside: primary #fa3c00, surface #ffffff, text #111827; fonts Germania One, Roboto.

## Sunny retro (three packages)

A butter-yellow floor (#fff4cf), brown-black ink, a burnt-orange accent, small 4/8/12 radii and a typewriter or heavy display face. 70s print and zine moods. The accent needs a small darkening for its white label.

**Tokens:** background #fff4cf · foreground #2a1810 (15) · card #fffaf0 · muted #ffdca8 · muted-foreground #7e5a46 (5.6) · border #d9aa7a · primary #ce481b (4.2 on ground) · primary-foreground #ffffff (4.6) · destructive #b83a2f (5.2) · radius 4/8/12 · type 14/16/20/28/40/56 · body line 1.45
**Fixed:** muted-foreground → #7e5a46 so it also holds 4.5 on muted; primary #d24b1f → #ce481b so its label #ffffff holds 4.5 (4.4:1).

- **dithered**: dot-shaded retro print. Suits studio, game. Canvas fonts Archivo + Work Sans (tokens: Courier New / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Space Grotesk, Open Sans.
- **neobrutalism**: thick borders, vivid fills. Suits agency, studio, product. Canvas fonts Bricolage Grotesque + Figtree (tokens: Arial Black / Inter). bergside: primary #fdc800, surface #fbfbf9, text #1c293c; fonts Inter, JetBrains Mono.
- **retro**: 70s print and typewriter. Suits cafe, workshop, retail. Canvas fonts Zilla Slab + Rethink Sans (tokens: Courier New / Inter). bergside: primary #3b82f6 (its placeholder pair), surface #ffffff, text #111827; fonts Macondo, JetBrains Mono.

## Levels

Warm white, green-black ink, a leaf green (#2f8f46) and round 12/18/28 radii; a conversion page that guides with one green. The green and the grey need darkening.

**Tokens:** background #fbf7ef · foreground #1f2a24 (14) · card #ffffff · muted #eef7ed · muted-foreground #697367 (4.6) · border #dbe3d7 · primary #23863d (4.3 on ground) · primary-foreground #ffffff (4.6) · destructive #dc2626 (4.5) · radius 12/18/28 · type 16/18/24/34/48/66 · body line 1.56 · display tracking -0.02em
**Fixed:** muted-foreground #788276 → #697367 (3.7:1); primary #2f8f46 → #23863d so its label #ffffff holds 4.5 (4.1:1).

- **levels**: conversion page guided by one green. Suits saas, clinic, product. Canvas fonts Red Hat Display + Red Hat Text (tokens: Inter). bergside: primary #27272a, surface #ffffff, text #111827; fonts Inter, JetBrains Mono.

Adapted from open-design design-systems (nexu-io/open-design@5fdcc3d, Apache-2.0) and bergside/awesome-design-skills (@f631a09, MIT, Copyright (c) 2026 Bergside; skills authored with typeui.sh). Changed: rewritten for the OpenPencil canvas; colours checked for contrast.
