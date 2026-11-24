---
name: huashu-directions
description: "The three-direction gate: brief, spec, six candidates from three logics, direction cards, picking three that differ, borrowing one layer from a reference, presenting real drafts and stopping for the choice. Read it before any new visual design: site, app, deck, video or image."
license: "MIT, Copyright (c) 2026 alchaincyf (花叔 · 花生)"
source: "https://github.com/alchaincyf/huashu-design @59065698ad63f87ea1532c126c87f488d4f93fd6"
---
# Three directions: three good designs, not three skins

The three-direction gate: brief, spec, six candidates from three logics, direction cards, picking three that differ, borrowing one layer from a reference, presenting real drafts and stopping for the choice.

**The gate:** every task that produces a new visual design first shows three real drafts in three directions and waits for the person to pick. A named style or brand does not exempt it ("gaya Apple" means three readings inside Apple's world). The only exemptions, each written in your notes with the person's words: they said in this conversation to skip it; you are iterating inside a project whose direction is already chosen; the job is mechanical (export, fix a typo, resize).

This extends `design-loop` §8 (references without copying) and Taste's "deal, never choose by taste". For sites and apps, Taste's dealt palettes and pairings still supply each direction's colour and type, and Taste's house rules bind all three; the six-candidate logic below decides structure, hero material and where the boldness goes. For decks, video and images, this logic leads.

## Why it works this way (three rounds of blind tests)

1. **Three versions that only changed skin** (same skeleton, new colours) were recognised at once. Differences must sit in structure, not palette.
2. **Forcing a big metaphor on every direction** turned pages into props (a filing cabinet, a library card, a subway map) and lost three of four blind picks. The same subway map even appeared in two unrelated briefs.
3. **Piling on bans** to dodge the "AI default look" pushed all three into pale, cool, regular grids: "terlalu rapi, terlalu membosankan". It lost again; the old versions the person liked were exactly dark-with-one-accent and warm-paper-serif.

So: **add options, not bans.** Six candidates, pick the three strongest and most different. Look families (warm paper serif, dark with one accent, hairline newspaper, card kit) are not faults; three directions crowding into one family, or repeating your last project, is.

## The steps

**1. Clarify.** Your one round of questions (`huashu-workflow` §2), asking for all reference material at once: the name, logo, colours, any site or photo with "that feeling". A named brand or product adds `huashu-brand-asset-protocol`; all three directions use the same real assets and differ in interpretation. A plain theme (coffee, history, fitness) is not a brand: don't hunt for "the coffee logo".

**2. Restate like a consultant.** In your own words: the real need, audience, setting, emotional tone, the unspoken expectation. End with "saya buatkan tiga arah nyata untuk Anda lihat", never with "arah mana yang Anda mau?": choosing before seeing anything is worthless.

**3. Freeze a spec** shared by all three: what it is, audience and setting, the key messages, the tone, **the output format and size (identical for all three, or they can't be compared)**, constraints, image needs. The spec says what to say, never what it looks like.

**4. Gather material first** (`huashu-image-briefs`, asset protocol): the content's essential pictures and every named product's logo, shared by all three. One direction with real photos against two with flat boxes is not a design comparison.

**5. Six candidates, two from each logic:**

- **Best designer ×2.** Breathe and think: with an unlimited budget, which studio or designer in the world fits this person and subject best? Pick two with different temperaments (one bold, one restrained; one Western, one Asian). Think the way they would, from scratch; don't imitate one of their works. Vary the names beyond Pentagram and Kenya Hara: Irma Boom, Experimental Jetset, Collins, Studio Dumbar, Ikko Tanaka, Sugiura Kohei, Lü Jingren, and Indonesian or Southeast Asian studios you can verify.
- **Real-world reference ×2.** Two verified excellent works **in the same medium**: an app for an app, a website for a site, a publication or proposal for a deck, newspaper graphics for an infographic. Cross-medium references (a deck styled as a 1935 paperback cover) tested as either overdone or boring. Look at the actual work, not your memory of its name; measure it (grid columns, headline to body ratio, share of the main colour, picture to text relation). If you can't see it, write "not seen" on the card and don't pretend.
- **Seconds roulette ×2.** Take the current second S. From the matching section of the style index (`huashu-design-styles`) pick entry S mod (entries in section) + 1, and from the East Asian section S mod 12 + 1. Study what the style looks like done well before writing the card. If it repeats your last three deliveries, take the next one. The roulette takes you where you wouldn't go yourself.

**6. Borrow one layer, never the whole body.** The clearer you see a reference, the easier it is to make its remake. Split it into three layers and take only the middle one:

| Layer | What it is | Do |
|---|---|---|
| Problem | What it solves for its user | Anyone may solve the same problem |
| Mechanism | How: the main screen's form, the shape the data is drawn in, the order information appears, the grid relation, one interaction | **Borrow this** |
| Expression | Its face: palette, typefaces, shape language, illustration, copy voice, metaphor | **Never.** Grow it again from this subject |

Borrow one mechanism per source, written as a sentence without its looks ("progress is a solid bar with ticks; done is solid" is a mechanism; "blue ground, yellow pill" is expression). Test: someone who never saw the original couldn't draw it from your sentence. Borrow across categories where you can (a bookkeeping app borrowing a tide app's curve). The most visible thing on the screen must be the subject's own; the borrowed mechanism is bone, the subject is the face. Name the source on the card and tell the person what you borrowed: what you can say out loud is borrowing; what you'd hide is copying.

**7. Write six direction cards** (template and an Indonesian example in `references/direction-cards.md`): logic and source, what is borrowed, what is not borrowed and where colour and type come from instead, the hero material, the skeleton in one sentence, where the boldness goes (one place), and where the form comes from in the content. A card you can't make concrete is a candidate not yet thought through: rewrite it from another source; don't sketch thumbnails to rescue it.

**8. Pick three, in this order:**

1. **Good enough alone.** A card that can't name its hero and skeleton won't be saved by the build.
2. **Different from each other:** different skeletons, different temperaments (not all the same temperature), not crowded in one look family.
3. **Not your last project:** avoid looks and references you just delivered.
4. **Not overdone:** at most one of the three turns the whole page into a prop or a universal metaphor (subway map, terminal, newspaper front page, dashboard), and only when the content needs it. Test: remove the device; is the message clearer or muddier? Clearer means it was overdone.

Three from the same logic is fine if they are the strongest.

**9. Build the three drafts**, each from the spec plus its own card only, never looking at the other drafts (if you can't work in parallel, build them one after another and forget the previous one). Same real content in all three; readability floors apply (`huashu-workflow` references). Then one polish round each (`huashu-polish-loop`): a first screen that doesn't stand, a missing hero material or identical cards all the way down must be fixed before showing. Then the **recognition test** for each draft with a real reference: put it beside the reference, ignore all text, and ask whether someone who knows the original would call yours its new version. If yes, keep the mechanism, redo the expression.

What a draft is, by medium:

| Medium | Each direction shows |
|---|---|
| Site or landing page | First screen at 1440 and 390 on the canvas, plus a one-line outline of the sections below |
| App | Three to four key screens on a composed board (`huashu-app-paradigms`) |
| Deck | Two representative slides, the two most different page types (cover and a data page) |
| Video | One direction board: one or two real rendered keyframes, a palette strip, one positioning sentence, the reference's name |
| Single image or cover | One real generated image |
| Social carousel | The cover plus the hardest inner page |

On the canvas, keep each direction in its own top-level frames, named "Arah 1 · …", and stay inside the 300-element budget of `pen-format`: draft the first screen, not the whole site.

**10. Present, then stop.** Show the three drafts together, each with its card summary (logic and source, what it borrows, hero material) and one line on where the form came from. Then one or two sentences on each of the three unbuilt candidates: "tiga ini cadangan; kalau ada yang menarik, saya kembangkan". End your turn and wait, even in an unattended session.

**11. After the choice.** Write in your notes which drafts were shown and the person's words when choosing. Mixed choices ("tata letak A, warna C") are fine: build them, and note which columns of which cards were mixed; that teaches more than the colour they liked. Record taste statements verbatim with scope ("terlalu polos", "ini nggak kelihatan AI").

## Template chrome vs look families

Huashu splits the 2026 "AI default look" in two, which extends `slop-audit`:

- **Look families** (fine to use, never all three directions in one): warm cream ground with high-contrast serif and terracotta accent (your own house look); near-black with one bright accent; hairline newspaper columns with zero radius; the SaaS card kit with identical radii and one grey shadow.
- **Template chrome** (delete unless the brief asks): a tracked all-caps label over every heading; "A · B · C" chains joined by middle dots; "WORD — fragment" labels; #111 instead of black; monospace for small data labels; a trailing arrow on links and buttons; one recoloured or italic word per headline; 01/02/03 on content that isn't a sequence. Swapping the dot for another symbol is not a fix; remove the structure.

Adapted from huashu-design (MIT). Changed: translated to English and rewritten for Nomi's canvas, decks and video.
