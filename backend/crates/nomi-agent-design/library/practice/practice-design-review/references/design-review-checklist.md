# Design review checklist: the ten categories

The full per-screen checklist for `practice-design-review`, translated to the canvas: each item is something you can see on your frames at 1440, 768 and 390. Where an item and Taste disagree, Taste wins; pass/fail gates stay in `taste-check` and `slop-audit`.

## 1. Visual hierarchy and composition

- One clear focal point and one main action per screen.
- The eye flows top-left to bottom-right without being pulled sideways by noise.
- Nothing competes with the main element: count the things shouting.
- Density fits the content (a menu can be denser than a hero).
- Nothing overlaps by accident.
- The first screen says what this is in three seconds.
- Squint test: blur your eyes at the frame; the hierarchy still reads.
- White space is placed on purpose, not left over.

## 2. Typography

- At most three families, and only the pairing you dealt (`taste-type` decides which).
- A size scale with a steady ratio (about 1.25 or 1.333 between steps); no random sizes.
- Line height about 1.5 for body, 1.15-1.25 for headings, set on every text node.
- Lines 45-75 characters (about 66 ideal); paragraph text nodes are fixed-width so they wrap.
- Heading levels are not skipped; at least two weights carry hierarchy.
- Curly quotes and a real ellipsis (…) in copy, not straight quotes and three dots.
- Numbers in tables and prices line up (ask Koda for tabular figures in the handoff).
- Body text 16px or more; labels and captions 12px or more.
- No letter-spacing on lowercase body text; negative spacing only on large headings.

## 3. Colour and contrast

- A coherent palette: few non-grey colours (Taste: 90-95% neutral, one accent).
- Text people read passes its contrast (numbers in `taste-color`); field borders, focus rings and icons 3:1.
- Semantic colours consistent: success green, error `$color-destructive`, warning amber.
- Colour never carries meaning alone: add a label, icon or pattern.
- No red/green-only pairs (about 8% of men can't tell them apart).
- Neutrals are all warm or all cool, never mixed.
- Dark theme (app screens, tech pages): surfaces step lighter for elevation instead of inverting; body text off-white rather than pure white; the accent slightly less saturated than in light.

## 4. Spacing and layout

- One grid at every width; every block on the content edges or the columns.
- Spacing from the `$space-n` scale only; no odd values.
- Related things closer together, separate things further apart.
- A radius hierarchy from the `$radius-*` ladder, not one bubbly radius on everything.
- Nested corners: inner radius = outer radius minus the padding between them.
- Body text never runs full-bleed; a max content width is set.
- Phone frames leave room for the notch and home bar at top and bottom.
- Frames exist at 390, 768 and 1440 (Taste's widths).

## 5. States

- Every interactive component has hover, focus, pressed and disabled variants drawn on the design system frame (`<Component> / <Variant> / <State>`).
- Focus is a visible ring in `$color-ring`, never removed.
- Disabled is clearly dimmed.
- Loading skeletons match the real layout's shapes.
- Empty states: a short message, one action and a reason to act, never just "Tidak ada data."
- Errors are specific and say how to fix: "Nomor WhatsApp harus diawali 08 atau +62", not "Input tidak valid".
- Success is confirmed visibly.
- Tap targets 44px or more on phone frames.
- Mindless choices: at every button, link, menu or dialog choice it is obvious what happens. A choice that needs thought is a high finding.

## 6. Responsive

- The 390 frame makes design sense on its own, not a squeezed desktop stack.
- Nothing runs wider than the frame.
- Photos keep their ratio and crop sensibly at each width (say the phone crop in the placeholder note).
- Navigation collapses on purpose (a menu sheet or a bottom bar), and the main action stays reachable.
- Forms are usable one-handed: one column, right keyboard noted (phone, email, number).

## 7. Motion notes (you annotate, Koda builds)

- Every motion has a job: state change, attention or where something came from.
- Entrances ease out; moves ease in and out; durations 50-700ms.
- Only position, scale and opacity animate; nothing grows from zero (start at 85% or more).
- One authored moment per page (Taste: a hero entrance plus one or two dealt moments), not the same entrance on every section.
- Content is visible by default; nothing important waits on an animation; reduced-motion shows the end state.

## 8. Copy

- Button labels name the outcome ("Simpan alamat", "Booking kamar"), never "Lanjut" or "Kirim" alone where something more specific fits.
- No placeholder or lorem text anywhere.
- Long names and long prices are drawn truncated or wrapped on purpose.
- Active voice; loading labels end with an ellipsis ("Menyimpan…").
- Destructive actions have a confirm dialog or an undo.
- Happy talk ("Selamat datang di...", self-praise) is flagged: report "This screen has X words, Y of them happy talk."
- Instructions longer than one sentence mean the design failed: flag the instructions and the interaction they cover for.
- Sentence-case headings; errors say what happened and what to do, never "Oops!"; no exclamation marks on success.
- Sample data in drawn UI looks real: local names, uneven numbers, varied dates (`taste-copy`).

## 9. AI slop

The test: would a good studio ship this? Common tells (see `slop-audit` for the full audit):

- Violet, indigo or blue-to-purple grounds or gradients.
- The three-column feature grid: icon in a coloured circle, bold title, two lines, three times.
- Icons in coloured circles as decoration; an icon tile above every heading.
- Everything centred.
- One large radius on everything; cards nested in cards; stacked cards instead of layout in app screens.
- Decorative blobs, wavy dividers, glows, halos, grid-paper grounds.
- Emoji as bullets or decoration.
- A coloured left border on cards.
- Generic hero copy ("Solusi all-in-one untuk...", "Wujudkan..."); marketing buzzwords.
- The template rhythm: hero, three features, testimonials, pricing, CTA, all the same height.
- A hero chip or kicker above every heading.
- A testimonial row with stars and quotes nobody said; three big round numbers under the hero ("10rb+ pengguna").
- "Mulai" and "Pelajari lebih lanjut" as the only actions.
- A grid of identical cards for content of unequal weight.
- Sparklines, rings and fake avatars filling space where real data should be.
- Only the happy path drawn.
- **The three looks.** Generated work lands on one of three looks whatever the business: cream ground with a high-contrast serif and a terracotta accent; near-black with one neon accent and glowing edges; broadsheet hairlines with an italic serif and tiny tracked mono labels. Each is fine when the brief asks for it. If you landed there without being asked, you stopped looking. Could someone guess your look from the trade alone? Then deal again from `taste-color` and `taste-type`.

## 10. Performance as design (what you control)

- Photo placeholders state the real crop and size so Koda can export the right image.
- Skeleton screens for slow lists are drawn and match the real layout.
- No layout that shifts when a photo or font arrives: give every picture a fixed frame.
- Fonts: at most the families you dealt, and only the weights you use (name them in the handoff).

## Persuade pages: five checks (a failed one is a high finding, usually deferred to the person)

1. **One main action,** named; nothing competes with it on the first screen, and the close repeats it word for word.
2. **Named objections:** the visitor's top worries, each answered where it comes up (a real FAQ, a comparison, the guarantee).
3. **Proof beside each claim:** a real customer, quote or figure from the brief next to the claim it supports, never a metric row. An unsupported claim is cut.
4. **A risk reversal the business really offers** (free trial, cancel anytime, uang kembali, gratis ongkir): never invented.
5. **Benefit first and specific:** "Laporan mingguan jadi 15 menit, bukan 4 jam", never "hemat waktu". A page behind an ad repeats the ad's promise in its hero. A high-commitment offer moves the FAQ up.

## Universal rules

- Never body text under 16px or under its contrast number.
- Never a label only inside the field (placeholder as label): the label stays visible above.
- A heading sits closer to the section it introduces than to the one before it: more space above a heading than below.
- Depth has an offset: a shadow is offset plus soft blur; a zero-offset coloured glow is decoration.
- Secondary text on a coloured ground is tinted from that ground's hue, never plain grey.
- If deleting 30% of the copy improves the screen, keep deleting.
- Cards earn their place: a card only when the card is the thing you tap.

Adapted from garrytan/gstack (MIT, Copyright (c) 2026 Garry Tan), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
