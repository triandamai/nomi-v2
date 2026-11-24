---
name: huashu-app-paradigms
description: "Eight interface paradigms distilled from two years of App Store Awards finalists, the five things every app direction must decide, the presentation board instead of a row of phones, real-image honesty and information density. Read it when designing an app's screens or app store images."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# App paradigms

Eight interface paradigms distilled from two years of App Store Awards finalists, the five things every app direction must decide, the presentation board instead of a row of phones, real-image honesty and information density.

This extends `taste-app-screens` (Taste's system for dashboards, lists, forms and states stays in force) and feeds the six candidates in `huashu-directions`. It is a source of ideas, not a quota: no screen must use a paradigm.

**Where it comes from:** the 90 finalists of the 2025 and 2026 App Store Awards, 1572 store screenshots, all looked at (November 2026). Only what is visible in store images: main screen form, visual system, how data is drawn, copy voice. Flows, empty states, gestures and real motion are not covered; don't infer them from here. A finalist is not automatically a visual benchmark. Borrow the mechanism, never the named app's palette, type, shapes or illustration (`huashu-directions`, "borrow one layer").

## First ask one question

The excellent apps almost never open on "nav bar, card list, five tabs". Each first answered: **what does the thing this app manages look like in real life?** A day is a timeline with lengths, a tide is a curve, a camera is a machine with levers, drinking water is a level in a jug. The interface grows from that shape; controls come later. A warung's daily cash is a drawer filling up; a laundry order is a basket moving through stations; a kos (boarding house) is a floor plan of rooms.

## The eight paradigms

1. **The interface is a world.** The screen is one place or object; functions are things in it, not floating controls. (A camera app whose screen is the camera's back; a focus timer that is a small isometric room.) Ask: can the main screen be a place or a thing? Not for: scanning and comparing many rows (bookkeeping lists, mail). On the web: the first screen shows the product working in its scene.
2. **A signature object.** One self-made control or graphic found nowhere else, recognisable in a screenshot; everything else can be plain. (A progress bar that is a solid pill followed by fine ticks; a water jug with the percentage on it.) Ask: what does the user touch most each day? Make that 120%. Not for: copying it onto every screen. On the web: one signature interaction beats an effect on every section.
3. **Time has a shape.** A day, a week, a trip is a graphic with lengths and rises, not a list. (Tasks as capsules on a vertical timeline, length = duration; the moon moving along a dotted orbit.) Not for: places where exact numbers are checked (reconciliation, expense claims): shape gives feeling, tables give accuracy. On the web: pricing, process and roadmap, think shape before table.
4. **One semantic colour set.** Colour answers exactly one question everywhere. (Each health metric its own colour, the whole screen turning that colour inside it; captions tagged blue for "asked you", pink for "said your name".) Not for: giving every category a colour no decision depends on. On the web: colour has one job sitewide.
5. **Content is the hero.** The interface has almost no colour or shape of its own. (A search that turns the whole page the colour you searched; a painting filling the screen with one small pill.) Not for: products whose content isn't beautiful (forms, settings). On the web: portfolios, media, product detail pages take ground and accent from the content.
6. **AI lands on the object.** The result appears on the thing the user was already looking at, not in a separate chat. (Photograph an item and the listing form fills itself; a spoken sentence becomes a task card on the timeline.) Ask: on which existing thing should this result grow, and how does the user spot and fix an uncertain one in one tap? Not for: showing "we have AI" with sparkle icons, gradient halos or a half-screen chat box; almost no finalist made chat the main screen. On the web: show the thing after the AI changed it, before and after.
7. **A character carries the state.** A face shows the state; numbers step back. (Stress as a face whose skin you can swap; a bean knitting while you focus.) Each state is followed by "so what to do". Not for: a mascot pasted in a corner that never changes; serious, high-risk contexts (medical readings, money). On the web: empty, loading, success and error are the four best places.
8. **Accessibility decides the form.** Design first for the user who struggles most (low vision, no hearing, one hand, harsh sun, running); the layout follows and everyone gains. (One screen that is one button.) Always applies; it pushes to big type, high contrast, one thing per screen.

Three more, for every direction:

- **Type is the voice** (`huashu-typography` §4). No excellent finalist used the system font throughout.
- **Copy takes a stance.** Write what only this product would say, on buttons, empty states and hints too. "Bosan aplikasi bahasa yang cuma untuk pemula? Kami juga." beats "Selamat datang". Never "Mulai sekarang" when "Catat penjualan pertama" exists.
- **Plain words first, then the number.** "Di bawah biasanya" next to the metric, the range below; "sisi kanan" rather than a bearing; one big number per screen. Ask beside each number: does the user know if it's good, and what to do?

**Almost never seen in those finalists** (a reverse checklist): violet-blue "AI" gradients, a chat box as main screen, a coloured icon on every card, five tabs with equal weight, one system font everywhere, frosted glass everywhere. Glass appears only on floating toolbars over content, never as the whole screen's material.

## Five things every app direction decides

At least three must differ between the three directions:

| Decide | Ask |
|---|---|
| Main screen form | A list, a scene, one big number, a content wall, a timeline? |
| The signature | What the user touches most, in a form only this app has |
| The shape of the data | Pill, ring, curve, water level, or really a table? |
| What colour answers | The one thing colour separates here |
| Voice | Who is speaking in type and copy? |

## The board, not a row of phones

In blind tests, five or six equal phones in an even row read as "terlalu rapi, membosankan", whatever the screens looked like. The board is each direction's first poster:

| Move | How | Good for |
|---|---|---|
| Primary and secondary | The most representative screen at 1.2 to 1.4 times, centred; the rest smaller, staggered at different heights | Almost always; the default |
| Flow | Screens in the order of a real task, thin lines from "tapped here" to "went there", notes on the lines | Apps with one main task (ordering, recording) |
| Close-up | Beside one screen, a 2 to 3 times close-up of the signature control or data graphic | A detail worth remembering |
| Context ground | The board's ground is the app's context: a scene colour, its material, not a white canvas | Strong settings (kitchen, outdoors, night) |
| Board typography | The board has its own title area (app name, one-line position, screen labels) in the direction's own system | Always |

Rules: the board's ground and type follow the direction (a dark direction gets a dark board); a phone screen never shrinks below 300 wide; the three directions use different board compositions. On the canvas, build a board as one top-level frame per direction with the 390 screens placed inside it as `ref` instances of the screen frames, so the screens stay editable.

## Real pictures and density

- Where an interface needs a picture (avatar, dish, product, cover, place) it gets a real one; on the canvas a sized, captioned placeholder (`pen-format`), never an emoji or a drawn person. **Honesty test:** remove the picture; is information lost? A cover image on an essay list, a scenic header on a profile, a decorative banner in settings: decoration, leave it out. A dish photo on a menu, the item in a product detail, the place on a map card: content, required.
- **Restrained density is the default:** one container, border and decorative icon fewer. **Dense is the exception** when the product's selling point is intelligence, data or context (AI tools, dashboards, trackers, health, bookkeeping): at least three pieces of real, differentiating information per screen (non-decorative data, a reasoning snippet, an inferred state, a linked context). A screen with one button and a clock expresses no intelligence.
- Taste is one detail at 120%, the rest at 80%: leave one place worth a screenshot.

## App store images

They are an ad read in sequence, not a gallery; in search results only the first three show, each about a quarter of the screen, for a second or two. Write a one-sentence-per-image storyboard first (1 what it is and for whom, 2 the strongest feature, 3 what differs from the rest, then the other features in the order users care, last a stance or real social proof); **the first three must stand alone**. One sentence per image, two or three short lines, readable at search size; say what the user gets or ask a question. Only the app's real interface (crop, enlarge, break it out of the phone, but never draw a feature the app lacks). Awards, ratings, reviews, press logos and download counts only if real and traceable. No other brands' devices or apps, no "terbaik" or "nomor satu", no prices (they expire). Current sizes: iPhone 6.9-inch 1320 × 2868, iPad 13-inch 2064 × 2752, no transparency, up to ten per device. Moves seen among finalists: images joined into a panorama with the subject straddling the seam; a sentence split across two images; an interface fragment breaking out of the phone; no phone at all; a macro close-up of one control; one ground throughout, or one colour per image; one inverted image to break the rhythm; a closing image with words only.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
