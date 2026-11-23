# Taste: the core rules

Read this first: Taste Skill is your design rulebook, and this file gives its nine house rules, the page kinds and what each may carry, the order you work in, the benchmark and the anti-slop principles.

Taste decides: where an older Sora habit or ban disagrees, Taste wins. Detail is in the other `taste-*` topics; run `taste-check` before every save.

## The goal

Design the site a good company would ship: it says plainly what is sold, shows the real thing on every screen, and looks made for this client, not from a template and not like an art piece. figma.com and x.ai are the bar: the hero fills the first screen alone, every section is a calm, full screen with one idea, the product or place is shown big and real, the copy is plain.

## The nine house rules (they win over everything else)

1. **No caption or description under any image or visual** on the page. Your `Photo · …` text inside a photo placeholder is a canvas note for Koda, never page copy (`taste-photos-motion`).
2. **Every visual is drawn, never an empty ground**, at 390, 768 and 1440: a photo placeholder at its final size, tone and note; UI drawn as real UI with real data; never a blank or grey box.
3. **One section per screen:** the hero alone on the first screen; every section exactly one or two screens tall (900 or 1800 at 1440x900); no second heading peeking in. One exception: a `photo-chapters` moment section, three or four screens.
4. **Equal left and right content edges:** every block on the content edges or the 12-column grid.
5. **Hover micro-interactions** (Koda builds them; you design for them and annotate): a linked card lifts 2px, a photo in a link zooms 1.02-1.03 inside its frame, buttons step up 1px on hover and press on click, underlines draw. No tilt, carousels or timed tabs. Every hero enters once on load; every page has one dealt moment, two on a long page.
6. **More and better real photographs where they fit;** nicer grounds, better heroes, layouts and fonts.
7. **Real full-colour icons** for real products (WhatsApp, Google Sheets, Shopee), never a drawn look-alike; small diagram visuals in cards on tech pages.
8. **Moments on every page** (photo- and type-led off tech); shader effects and scenes only where the trade is technical (software, AI, developer tools, gaming, fintech, hardware), never on a cafe, bakery, guesthouse or shop: those get photographs.
9. **Addresses:** street and district only ("Jl. Malioboro, Gedongtengen"), no house number, postcode or coordinates; no tax or company register numbers; no invented phone or WhatsApp number; stand-in contacts on `.example`.

## The kind decides everything

Write the kind on line 1 of your plan.

| Brief | kind | tech? |
|---|---|---|
| App, SaaS, AI, developer tool, fintech, POS or booking software sold to others | software | tech |
| A game, a studio launching its game | gaming | tech |
| Hardware or a tech device (a drone, a keyboard, a smart lock) | tech | tech |
| Any other physical product or brand sold mostly online (batik, coffee beans, skincare, furniture) | product | other |
| People are hired: clinic, law office, agency, freelancer, photographer, tutor, caterer, contractor | service | other |
| People come to it: cafe, warung, bakery, restaurant, guesthouse, salon, barbershop, laundry, bengkel, shop, gym, local studio | place | other |

A maker that also opens its door or teaches (a pottery studio with a web shop, a bakery that ships) is a **place**; its shop becomes the goods section. It is a product only when nothing happens at the door.

**What each kind may carry**

| | software, gaming | tech hardware | product, service | place |
|---|---|---|---|---|
| Lead picture | its own UI (a game: its key art) | its photographs | its photographs | its photographs, 12-20 on a full page |
| UI fragments in cards | yes (gaming: none) | its app | only an app the client runs | never |
| Diagram cards | 0-2 (pulsing, annotated) | 0-2 | 0-1, still | never |
| Real brand icons | yes | yes | tools really used | never |
| One effect or big scene | yes, annotated | yes | never | never |
| Dark page or dark hero | allowed | allowed | never | never |
| Moments | 1-2, any | 1-2 | 1-2, photo or type | 1-2, photo or type |
| Card loops, counters | yes (annotated) | yes | never | never |
| Grounds | tokens plus up to 2 dealt grounds | same | flat tokens | flat tokens |

On every "other" page: photographs, flat colour, type, real icons, the hero entrance, its moment and the hover layer. Nothing else.

## How you work (Taste's run, translated)

1. **Classify.** Write the kind, what is sold, who buys it, and the **one main action** (the transaction: Pesan kue, Booking kamar, Mulai coba gratis; a place with neither: Petunjuk arah). Write the fact sheet: what the brief really gives (logo, photos, prices, address, hours, people, numbers). Nothing else is invented (`taste-copy`).
2. **Plan as screens.** Line 1 the kind. Line 2 the deal: palette (and one sentence on why it fits the trade), type pairing, hero, nav, footer, close, rhythm, button look, mark, moments. Then one line per section: its id, its one idea, its picture and the picture's kind (object, use, parts, world, type, diagram), its screens (1 or 2, from content height), and which section holds a moment and why it moves. Three "object" lines at most.
3. **Deal palette and type,** never by personal taste: from the trade's palettes and the kind's pairings (`taste-color`, `taste-type`), avoiding the last ones you used in this conversation. The client's own colours (logo, shopfront paint) replace the palette.
4. **Write all the copy first,** every heading, line, button, label and form message, in the person's language, before drawing a frame (Taste writes copy before planning sections; never draw with placeholder text).
5. **Draw.** Tokens first, then the reusable components (buttons, card, nav, form field), then each screen at 1440, 390 and 768, section by section as frames.
6. **Check** your own canvas against `taste-check` and fix every miss before handing off.

"Deal, never choose by taste" means: take options from Taste's lists by the kind and the material the brief really has, vary them from your previous designs, and write the choice and its reason on plan line 2. Never pick a look because it "feels nice".

## The benchmark

- figma.com and x.ai for company sites; Linear, Cursor and Vercel for showing software big in one app window; Figure and 1X for a product shown big; Daunt Books, Prufrock, Bourke Street Bakery, Gail's, Tartine for a place (learn the feel, never copy).
- Put your design beside them: does it read as a professional company's site? Nothing tilted, staggered, cut or overlapping? Is the hero the work itself, not a mood?

## Anti-slop principles

- **Show the real thing on every screen:** the product, the place, the people, the client's real UI. A section with no picture is allowed only for a price list, FAQ, specs table, booking form or one statement moment, and two of those are never neighbours.
- **One object never carries the page:** the product whole leads the hero and at most two more sections; use, parts, people, place and type carry the rest.
- **Plain words a stranger understands in five seconds.** The h1 names the category and the buyer or place. Facts where adjectives want to go. No invented reviews, numbers, customers or logos.
- **One strict system:** one button pair, one card style, one radius scale, one link look, one heading scale, one accent. Two boxes deep at most.
- **The interface stays 90 to 95% neutral;** colour comes from the photographs, the product and real logos. Never violet or lavender.
- **Fewer, bigger, calmer:** one idea a section, one main action a screen, top-aligned content, separation by ground change or space only.
- **Not the template skeleton** (eyebrow, claim heading, two-line lede, three cards, a figure strip) in every section. Shape follows the reader's questions and the material the brief has.
- **Never:** scroll fade-ins, parallax, carousels, popups, chat bubbles, badges, stock passed off as the client's, a drawn copy of a real logo, a copied site.

Adapted from Taste Skill 2.5.1 (MIT, © 2026 Leonxlnx).
