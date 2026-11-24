---
name: practice-color-expert
description: "Colour science for building tokens: plan in OKLCH for even ramps, map reference values to shadcn roles, check contrast in light and dark, harmonise by character not hue angles, and avoid clipped, muddy picks. Read when you build or fix a palette."
license: "CC-BY-4.0, by meodai (github.com/meodai), original materials of skill.color-expert (SKILL.md); via open-design (Apache-2.0)"
source: "https://github.com/meodai/skill.color-expert@5bf7c77 (SKILL.md only; its third-party reference corpus not used); open-design@5fdcc3d"
---
# Colour expert: palettes that hold up

Build colour as a small system, reference values mapped to roles, with even lightness steps, checked contrast and a clear character, so every screen reads well in light and dark.

Taste deals the palette (by trade and kind, `taste-color`) and the client's own colours come first. This skill is how you make those colours into good tokens. Where it suggests more colour than Taste allows (the 60-30-10 rule, full palettes), **Taste wins**: the interface stays 90-95% neutral and colour comes from the photographs and the product.

## Ask first

For a concrete project: the medium (screen, print on a menu or banner, both), the brand or mood, the audience, accessibility needs, and the colours that already exist (logo, shopfront paint, packaging, uniform). Harmonise with those; don't replace them.

## Think in OKLCH, write hex

The canvas takes hex. Plan in OKLCH anyway (lightness L, chroma C, hue H), because it is perceptually even:

- **HSL lies about lightness.** Full yellow and full blue share "50% lightness" in HSL but look very different in brightness. Two buttons with the "same lightness" in HSL won't look equally strong.
- **HSL hue spacing is uneven:** a small shift near red is dramatic, the same shift near green barely shows.
- **HSL saturation isn't perceived saturation:** a dark blue at 100% can still look muted.
- In OKLCH, equal L means equal perceived lightness across hues, so a ramp built at fixed L steps has no muddy middle and no sudden jumps.

Terms worth keeping straight: **chroma** is colourfulness against a neutral of the same lightness; **saturation** is colourfulness relative to the colour's own brightness; **lightness** is perceived reflectance; **brightness** is perceived light intensity. Same chroma does not mean same saturation.

## Ramps

- Build a neutral ramp (about 9-11 steps, background to foreground) and, if needed, one accent ramp, at even OKLCH lightness steps.
- **The test of a good ramp:** the perceived step between neighbours is the same all the way along, in colour and in greyscale. A bump means two steps look nearly identical or one jumps.
- Tint the neutrals slightly towards the brand hue (very low chroma) so they feel related; pure grey next to a warm photo looks cold.
- **Gamut:** a high chroma at some lightness may not exist on screens; it gets clipped to something duller and hue-shifted. When a colour is too intense, **reduce chroma, keep lightness and hue**: that preserves its identity.

## Reference values, then roles

Keep three layers, the same as `components-kit`:

1. **Reference values:** the ramp steps (krem-50 … cokelat-900, hijau-600).
2. **Roles:** `$color-background`, `$color-foreground`, `$color-card`, `$color-primary`, `$color-primary-foreground`, `$color-muted-foreground`, `$color-border`, `$color-ring`, `$color-destructive`, and any extra pair (`$color-success` / `$color-success-foreground`).
3. **Components** use only roles, never raw hex.

Decide roles by rule, not by eye, and write the rule in the handoff so Koda can recompute it:

- foreground on a surface = the ramp step with the best contrast against it;
- primary = the most vivid brand step that still reaches 4.5:1 with its label, never the destructive or success hue;
- background and foreground pinned to the ramp's ends, so they survive a ramp change;
- hover and pressed = the base mixed about 10-12% towards the foreground (darker on light, lighter on dark), not a new hand-picked hex.

## Contrast

- Check **every** text and background pair in both themes: 4.5:1 for body text, 3:1 for large text (about 24 regular or 19 bold) and for icons and field borders that carry meaning; 7:1 if you want comfortable long reading.
- Passing pairs are rare: only a small fraction of random colour pairs reach 4.5:1, and far fewer reach comfortable reading contrast. Expect to adjust.
- Greyscale is a quick check of lightness separation, not proof: still measure, and think about size, weight, dark-on-light versus light-on-dark, and colour blindness (never red versus green alone for meaning: add a label or icon).
- Legibility comes from lightness difference. Same character with varied lightness is the safest way to build hierarchy.

## Harmony

- **Hue-angle rules (complementary, triadic) are weak on their own:** each hue has a different shape in perceptual space, so equal angles don't give balanced results.
- **Harmonise by character:** pale, muted, deep, vivid, dark. Chroma and lightness predict the feeling more than hue does: a muted palette reads calm in almost any hue; a vivid one reads loud. Pick the character for the trade (a clinic: pale and muted; a kids' class: brighter, still one accent).
- **Colour meaning is per project, not a symbolism table:** don't pick green "because it means growth"; pick it because it's the shopfront paint.
- **One colour dominates.** The source's 60-30-10 becomes, under Taste: neutrals carry the screen, one accent marks the main action, photographs bring the rest.

## Temperature

Temperature isn't just hue: warming or cooling shifts hue and saturation together, differently per starting hue. Warm daylight photos want warm-leaning neutrals; cool, blue-shadowed photos want cooler ones. Greens and purples sit ambiguously between warm and cool; judge them in context.

## Dark theme

Not an inversion: build its own values. Surfaces step lighter as they rise (card one step above background); borders become low-alpha white; the primary usually needs to be lighter to keep its contrast; the destructive gets lighter too. Re-check every pair.

## Naming colours

Give each reference colour a name a person can say: by character and source ("hijau daun pisang", "krem roti", "cokelat kerak"), with the hex beside it (`practice-design-md`). Plain descriptive registers (pale, deep, warm grey) work better than poetic invented names, and the role name is what Koda uses.

## Mistakes to avoid

- Building scales in HSL and getting uneven, muddy steps.
- Picking a chroma the screen can't show, then fixing it by changing the hue.
- Freezing hand-picked hex values into components instead of roles.
- Checking contrast only in the light theme.
- Using colour as the only signal for status.
- Violet or lavender accents (Taste bans them) and gradients (the canvas can't draw them; note any intent for Koda instead).

Adapted from Color Expert by meodai (CC-BY-4.0, by meodai, https://github.com/meodai/skill.color-expert; only its original SKILL.md, not its third-party reference material), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
