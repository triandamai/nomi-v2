---
name: practice-ui-ux-pro-max
description: "A prioritised UI/UX rulebook (accessibility and touch first, then layout, type, colour, motion, forms, navigation) as canvas checks, what each trade's screens must contain, and a pre-delivery checklist for app screens. Read when designing or reviewing screens."
license: "MIT, Copyright (c) 2024 Next Level Builder; via open-design (Apache-2.0)"
source: "https://github.com/nextlevelbuilder/ui-ux-pro-max-skill@f2f7266 (SKILL.md, references/quick-reference.md, references/pro-rules.md, data/products.csv); open-design@5fdcc3d"
---
# UI/UX rules by priority

Work through the rules in priority order, critical first, and know what each trade's screens must contain before you style them.

The source is a searchable database (styles, palettes, font pairings, product types) with a search script; you don't run it. What you keep: the priority order, the rules, the per-trade content needs and the checklist. **Taste wins on style:** the database recommends glassmorphism, neumorphism, claymorphism and aurora looks for many trades; ignore them. Style and palette come from Taste's kinds, trades and the client's material.

## Priority table

| # | Category | Impact | Must have | Avoid |
|---|---|---|---|---|
| 1 | Accessibility | critical | 4.5:1 text contrast, visible focus, labels on icon buttons, logical reading order | removing focus rings, icon-only buttons without a label |
| 2 | Touch and interaction | critical | 44×44 targets, 8+ between targets, loading feedback | hover-only actions, state changes with no feedback |
| 3 | Performance (what you control) | high | images at their real ratio, space reserved for late content | layouts that jump when content arrives |
| 4 | Style | high | one style matched to the kind, consistent everywhere, real vector icons | mixing flat and 3D at random, emoji as icons |
| 5 | Layout and responsive | high | phone first (390), 768, 1440; no sideways scroll | fixed widths that break on a phone |
| 6 | Type and colour | medium | body 16, line height 1.5, semantic tokens | body under 12, grey on grey, raw hex in components |
| 7 | Motion | medium | meaning, timing by distance, reduced-motion fallback | one duration for everything, decorative motion |
| 8 | Forms and feedback | medium | visible labels, errors by the field, helper text, step-by-step for long forms | placeholder-only labels, errors only at the top |
| 9 | Navigation | high | predictable back, bottom nav of 5 or fewer, every screen addressable | overloaded nav, broken back |
| 10 | Charts and data | low | legends, labels, accessible colours | colour as the only meaning |

## Rules worth drawing for (by category)

**Accessibility.** Contrast 4.5:1 (3:1 large text and meaningful icons). Focus ring 2-4 thick in `$color-ring` with 3:1 against its surroundings. Headings in order, never skipping a level. Every field has a label. Information never by colour alone: add an icon or text. A cancel or back in every dialog and multi-step flow. Sticky bars and banners never cover the focused control. Text that can grow (system text size) without being cut off: give text frames room and let them wrap.

**Touch.** 44×44 minimum (48 on Android-like designs), extend the tap area past a small icon. 8 between targets. Every pressable thing has a pressed state. No horizontal swipe on main content; vertical scrolling first. Primary targets away from the notch, the home indicator and the screen edges. Swipe actions need a visible hint, and every gesture has a visible button alternative.

**Performance as design.** Draw every image frame at its real ratio so the page doesn't jump; draw skeletons for waits over a second instead of a long spinner; design an offline or slow-network message for apps.

**Style.** One style across all screens; one icon family with one stroke width; real brand logos with correct proportions; hover, pressed and disabled clearly distinct; one elevation scale (`components-kit`); light and dark designed together; native-looking controls unless the brand needs custom; one primary action per screen.

**Layout.** Phone first, then 768 and 1440. Body 16 on phones. Lines of 35-60 characters on phones, 60-75 on desktop (Taste caps at 62). A 4/8 spacing scale (`$space-*`). A consistent content width on desktop. Fixed top and bottom bars reserve space so content isn't hidden under them. Avoid scroll areas inside scroll areas. The core content first on phones; secondary content folds away. Hierarchy by size, space and contrast, not colour alone. Chips wrap to a new line before labels shrink; a "+3" overflow opens the rest.

**Type and colour.** Body line height 1.5-1.75; a consistent scale (12, 14, 16, 18, 24, 32 or Taste's); weights 600-700 headings, 400 body, 500 labels. Semantic tokens only. Dark mode uses lighter, less saturated tonal values, not inverted ones, checked separately. Prefer wrapping to truncation; if truncating, an ellipsis and a way to see the full text. No tight letter-spacing on body text. Tabular figures for prices, tables and timers.

**Motion (annotate for Koda).** One or two animated elements per view. Ease out when arriving, ease in when leaving; exits about 60-70% of the enter time. Motion shows cause and effect; dialogs and sheets grow from what opened them; forward navigation moves left or up, back moves right or down; press feedback scales 0.95-1.05 at most. Never block input during an animation. A reduced-motion version. Taste wins: no parallax or scroll fade-ins, even "subtle" ones.

**Forms.** A visible label per field; a specific error under the field; loading then success or error on submit; required fields marked; helpful empty states with an action; toasts that leave after 3-5 seconds; a confirm step before anything destructive; helper text under complex fields; disabled at 38-50% opacity; validate when the user leaves a field, not on every keystroke; the right keyboard per field (`practice-login-flow`); show/hide on passwords.

**Navigation.** Bottom nav with 5 items at most, each with an icon and a label; a predictable back; the current place always marked; no hamburger as the only nav on a phone app with few sections.

## What each trade's screens must contain

From the source's product data, adapted (social proof only when real, per Taste):

| Trade | Must contain |
|---|---|
| Bakery, cafe | menu with prices, ordering, location and hours prominent, catering or big orders, seasonal specials, appetising photographs |
| Restaurant | menu, ordering or reservation, food photography, location and hours on the first screen |
| Beauty, spa, salon | service menu with prices and durations, booking, calm photographs, before/after only with real photos |
| Clinic, dental, vet | services, the practitioners, online booking, emergency info (vet), what to bring, insurance or payment info |
| Hotel, guesthouse | rooms with prices, booking, amenities, location map, seasonal prices |
| Photographer | portfolio full-bleed, packages, booking |
| Wedding or event planner | portfolio, packages, a timeline of the process, enquiry form |
| Law office | practice areas, the lawyers, contact form; credibility through facts |
| Florist | catalogue by occasion, delivery date and time choice, care guides |
| Real estate | listings with photos and key facts, map, agent contact, a cost calculator if useful |
| Car dealer or rental | vehicles with photos, comparison, financing or rental terms, test-drive or booking |
| Coworking | space photos, membership plans, booking, amenities |
| Place of worship, community | service or meeting times, events, location, giving |
| Booking app | date strip or calendar, time-slot grid, service and staff choice, confirmation summary, reschedule and cancel |
| Food delivery app | restaurant list, menu categories, cart sheet, order status steps, live tracking |
| Shop (e-commerce) | product grid, product page with real photos and price, cart, checkout, delivery options |

## Pre-delivery checklist for app screens

- Checked at 390 and in landscape where it matters.
- Reduced-motion and the largest text size considered (text frames wrap, nothing cut off).
- Dark theme contrast checked on its own.
- Every target 44+ and nothing under the notch or home indicator.
- No emoji icons; one icon family; official logos with their clear space.
- Pressed states don't change the element's size.
- Tokens everywhere, no per-screen hex.
- Every tappable element has pressed feedback; disabled states look and act disabled.
- Reading order matches the visual order; labels are descriptive.
- No conflicting gestures (a swipe row inside a swipe-back screen).
- Primary and secondary text at 4.5:1 in both themes; dividers and states visible in both.

## If you're unsure

Don't present a rule as data you looked up. If no rule here or in Taste covers the case, say you're applying general defaults and why.

Adapted from UI/UX Pro Max by Next Level Builder (MIT, Copyright (c) 2024 Next Level Builder), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
