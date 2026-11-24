---
name: reference-contract
description: "Turn the person's references (their Instagram, a competitor, their old site) into a short contract: evidence labelled observed, provided or inferred; Keep, Change, Do not copy; one frozen direction; what the first preview must prove; a gate Koda can build from."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (skills/reference-design-contract/SKILL.md, skills/reference-design-contract/references/checklist.md)"
---
# The reference contract

"Bikin kayak gini" is not a direction. Before the first preview, turn every reference into explicit decisions so the design, its changes and Koda's build all follow the same rules. This adds the contract to `references-rules` (what you may borrow) and `design-loop` §8 (how to check overlap); it does not repeat them.

## Where it sits in your flow

1. **Collect evidence.** The person's Instagram, shop photos, a competitor's site, their old site, a menu photo. Read pages with `read_web_page` (text only: names, offer, prices, voice) and look at the images they send (colour, photos, layout, type). If you can't see something (Instagram needs a login, a page fails), ask for a screenshot; never describe a page you didn't read.
2. **Split each reference** into Keep, Change, Do not copy (below).
3. **Propose 2–4 directions** with `propose_directions`, built from the Keep columns, and mark the one you'd pick.
4. **Freeze one** when the person picks: write the contract below and pin it with `post_crew_note`. From here every version follows it; a change of direction is a new contract, said out loud.
5. **Carry it into the handoff** as its first section.

## Label every fact

| Label | Means | Example |
|---|---|---|
| provided | The person said it or sent it as theirs | Logo file, prices "Kopi susu Rp 18.000", hours 07.00–21.00 |
| observed | You saw it in a picture or read it on a page | Their Instagram grid is warm, top-down, daylight; the competitor's menu has 24 items in one long list |
| inferred | Your assumption | Most customers order on a phone; green from the shop sign is the brand colour |

Inferred facts are listed as assumptions in the handoff and asked about when they would change the design. Never present an inferred fact as the brand's.

## Keep, Change, Do not copy

Fill one row block per reference:

| Reference | Keep (qualities) | Change | Do not copy |
|---|---|---|---|
| Competitor: kopi shop site (observed) | One big photo per screen, short menu groups, prices visible without a tap | Their pastel pink becomes the client's sign green; their 4-column menu becomes 2 on a phone | Their name, logo, photos, menu names, prices, taglines, section order end to end |
| Client's Instagram (provided) | Their real photos, the green and cream of their cups, their casual voice ("Kak, pesan lewat WA ya") | Text set over busy photos (fails contrast) moves under the photo; the 6 fonts in their posts become one pairing | Nothing is off limits that is theirs; never repost others' photos from their feed as theirs |
| Old site (provided) | The address, hours and price list | The layout and its stock photos | Stock photos passed off as the shop |

Keep lists qualities you can draw: density, photo light and framing, colour temperature, type rhythm, how the main action is placed. Not "the vibe".

## The frozen direction

One paragraph of stance, then binding lines. Concrete beats adjectives: "one green accent from the shop sign, cream grounds, no pink" beats "fresh and premium".

- Kind and main action: place; "Pesan lewat WhatsApp".
- Palette: tokens with hex and source label (`brand-extract`), contrast checked.
- Type: the pairing from `taste-type`, sizes from its table.
- Radius ladder, button look, card style, grounds in order.
- Photos: subjects, framing, light, count.
- Density and rhythm: one idea per screen, menu 8 items per phone screen.
- Never (exact): "no pink, no pill buttons, no photo with text over it, no stock coffee beans".

If two directions truly compete, name the runner-up in one line and why it lost; don't keep a menu of moods.

## What the first preview must prove

Three to five lines, each checkable on the canvas:
- At 390 the hero says what is sold and where, with one price, before any scroll.
- The menu shows all 14 drinks with prices in two screens on a phone.
- Ordering takes 2 taps from the hero to WhatsApp.
- Every colour on screen is a `$color-*` token from the contract.

## Risks and unknowns

List what is missing and what you assumed: "No photos of the interior yet: placeholders with notes." "Prices for pastries not given: left out, not invented."

## The gate

Before you save the first preview and again before `create_handoff`, Koda (or a second designer) must be able to answer, from the contract alone:
- What is the product or surface, and its main action?
- What is kept from each reference?
- What must not be copied?
- Which colours, type, spacing, radius and components are binding?
- What would make the first preview fail review?

Any answer missing: fix the contract first. Keep it short enough to paste whole: 25 lines or fewer.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): skills/reference-design-contract/SKILL.md, skills/reference-design-contract/references/checklist.md. Changed: rewritten for the OpenPencil canvas.
