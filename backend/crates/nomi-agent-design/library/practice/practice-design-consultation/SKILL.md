---
name: practice-design-consultation
description: "How a senior designer runs a brand-from-zero consultation: confirm context in one message, find the one thing to remember, read the category in three layers, propose a system with safe choices and two risks. Read when a person has no brand yet."
license: "MIT, Copyright (c) 2026 Garry Tan (gstack; its font procedure and three-looks calibration derived from pbakaus/impeccable, Apache-2.0); via open-design (Apache-2.0)"
source: "https://github.com/garrytan/gstack/tree/a3ecfd530df3c78c2d60e92b6488586a75e0ff91/design-consultation (SKILL.md.tmpl, sections/proposal-and-preview.md.tmpl); open-design@5fdcc3d"
---
# Design consultation: a system from zero

Listen, read the category, then propose one coherent design system with a reason for every choice, safe where buyers expect it and bold in two places, and show it on the person's real screens before you record it.

Use it when the person has no brand, no site and no style ("baru mulai jualan, belum punya logo"), or asks to start fresh. Taste still decides the values (palette and type are dealt by trade and kind, `taste-color`, `taste-type`); this skill is the conversation and the structure around those choices.

## Phase 0: what already exists

Check for an earlier design in this conversation or the files: a logo, a design system frame, an old site. If there is one, ask once: update it, start fresh, or stop. Updating keeps every decision they don't want changed.

## Phase 1: context, confirmed in one message

Don't ask a string of questions. Write your read of the business, its buyers, its space and its kind in two or three sentences, and let the person confirm or correct it in one reply. Say plainly that they can talk it through instead of answering a form.

Then ask one separate question:

> "Satu hal apa yang ingin orang ingat setelah pertama kali melihat usaha Anda?"

Record the answer in one sentence (a feeling, a picture, a claim, a stance): "Rotinya dipanggang sendiri tiap subuh." Every later choice must serve it.

## Phase 2: read the category in three layers

From what you know and what the person shows you (competitor sites, Instagram accounts; `practice-competitive-ads`):

1. **Tried and true:** what buyers in this category expect (a bakery shows bread big, prices clearly, opening hours on the first screen).
2. **New and popular:** current patterns in the category, noted, not obeyed.
3. **First principles:** test the conventions against this business's buyers. Where a convention fails them, name it: "Semua toko roti di kota ini pakai krem dan cokelat karena dianggap 'hangat'. Pembeli Anda anak muda yang pesan lewat HP: warna dari kemasan Anda sendiri lebih kuat."

Never present remembered patterns as things you observed. If you saw nothing, say you are working from design knowledge.

## Phase 3: the proposal

Draft it yourself, completely, before discussing it. Every line is "X because Y".

| Part | Decide |
|---|---|
| Direction | one of: brutally minimal, luxury/refined, playful, editorial, organic/natural, industrial/utilitarian, retro, art deco, raw (pick by trade; Taste's kinds limit it: no dark or effect-heavy direction on a place or product) |
| Decoration | minimal (type does the work) or intentional (photo and type moments). Taste wins: no grain, texture fills or patterns on the canvas; texture comes from real photographs |
| Layout | grid-disciplined, or hybrid (grid for app screens, more editorial for the landing page). Taste wins on overlap and grid-breaking: equal edges, nothing tilted or overlapping |
| Colour approach | restrained (one accent and neutrals: Taste's default), committed (one hue leads, neutrals derived from it), full palette (only for apps with real status colours) |
| Type | by the reader's world and mode: persuade (marketing), operate (tasks), read (long text). Shortlist three per role, pick, and only name faces you know are licensed for the web |
| Spacing | base unit and density |
| Motion | minimal-functional or intentional (Taste: one hero entrance, one or two moments, the hover layer) |

**Avoid the three predictable looks** unless the brief asks for one: cream, serif and terracotta; near-black with neon and glowing edges; broadsheet hairlines with italic serif and tiny tracked mono. Draw the colours from the business's real material instead (its packaging, shopfront, uniforms, product).

**Vary between projects.** Don't land on the same direction, faces and palette as your last design without a reason. Light versus dark is not a style dial: it follows where and when people use it.

**Coherence check after any change the person asks for:** flag mismatches gently and offer an alternative (minimal direction with lively motion, a committed colour with no supporting moments, an editorial layout on a dense data screen), then accept their final choice.

### Present it with safe choices and risks

> **Arah:** organik dan hangat, karena roti dipanggang sendiri tiap subuh.
> **Warna:** netral terang, satu aksen hijau dari kemasan Anda (#2F5D3A).
> **Huruf:** judul Fraunces, teks Instrument Sans, karena ...
> **Ruang, gerak:** lega; satu gerakan masuk di hero.
> Sistem ini menyatu karena ...
> **Pilihan aman** (yang diharapkan pembeli): foto roti besar di layar pertama; jam buka dan harga mudah ditemukan.
> **Risiko yang disengaja** (yang membuat Anda diingat): (1) judul sangat besar dengan foto subuh di dapur; untung: langsung terasa buatan sendiri; biaya: lebih sedikit produk terlihat di layar pertama. (2) ...

Propose at least two creative risks (type, accent, spacing, layout or a moment), each with what it is, why it works, what it gains and what it costs. Coherence alone looks generic.

Then the person picks: proceed, adjust a part, try different risks, or start over.

## Phase 4: drill down only on request

One focused question per area they want to change: 3-5 font candidates with roles and a reason; 2-3 palettes with hex values; 2-3 directions that fit the business.

## Phase 5: preview on real screens

Draw the design system frame (`components-kit`, `practice-design-brief` step 3), then two or three realistic screens for this business, with its real name and real copy, never lorem ipsum:

- a landing page first screen and one more section,
- for an app: a list or dashboard screen and a form with validation states,
- a sign-in screen if accounts exist (`practice-login-flow`).

Gate yourself before showing: would a human designer sign this? Hard rejects: purple gradient hero, three-column feature grid, everything centred, an overused display face, stock photos posing as the client's, gradient buttons.

## Phase 6: record and confirm

After approval, write the system down (`practice-design-md`): atmosphere, colours by name, hex and role, type, components, layout, depth, motion, and the "one thing to remember". Any later change to a token, face or direction updates the record and the preview, and is confirmed again.

## Rules

1. Propose with reasons; explain every choice.
2. Keep the system coherent: its parts reinforce each other.
3. The first preview has to be beautiful; it sets the tone.
4. Stay conversational and accept the person's final choice after naming the tradeoff once.

Adapted from gstack's design-consultation by Garry Tan (MIT, Copyright (c) 2026 Garry Tan; font procedure and three-looks calibration derived there from pbakaus/impeccable, Apache-2.0), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
