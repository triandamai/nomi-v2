---
name: practice-design-md
description: "How to describe a design system in words a person and Koda can both use: atmosphere, colours by descriptive name, hex and role, type, component shapes, layout and depth. Read when you extract a system from the person's current site or screenshot, or write your own system down."
license: "Apache-2.0 (google-labs-code/stitch-skills); via open-design (Apache-2.0)"
source: "https://github.com/google-labs-code/stitch-skills@62fdcd6 (plugins/stitch-utilities/skills/design-md), the open-design stub pointed at google-labs-code/skills; open-design@5fdcc3d"
---
# Writing a design system down

Describe a visual system in plain design language with exact values in brackets, so every new screen, and Koda, can follow it without seeing the first one.

Two jobs use this:

- **Extracting:** the person shows their current website, Instagram grid, menu board or a screenshot, and says "yang seperti ini tapi lebih rapi". You read the system out of it before redesigning.
- **Recording:** you've drawn the design system frame; you write a short description of it into the handoff and the frame's note, so later screens (or a later conversation) stay consistent.

## The five parts

Write them in this order, each a few lines.

### 1. Atmosphere

The mood, density and philosophy in a sentence or two, with concrete adjectives: "Lapang dan tenang: latar krem, banyak ruang, foto besar, satu warna aksi hijau tua." Avoid vague words ("modern", "clean") without saying what makes it so.

### 2. Colours by name, value and role

Each colour as **a descriptive name, the hex in brackets, its job**, and the token it becomes:

- Krem roti (#F7F1E6): latar halaman, `$color-background`
- Cokelat kerak (#3B2A1E): teks utama, `$color-foreground`
- Hijau daun pisang (#2F5D3A): tombol utama dan tautan, `$color-primary`
- Abu hangat (#7A6F64): teks pendukung, `$color-muted-foreground`

Name by character and purpose, not "blue 2". A name helps the person remember and argue about it; the hex keeps it exact.

### 3. Type

The families, the weight for headings versus body, size steps and the letter-spacing character: "Judul Fraunces 600, rapat (-0.5 px di 48); teks Instrument Sans 400, 16/1.6."

### 4. Component shapes

Translate radius and depth into words, with the number:

| Value | Say |
|---|---|
| 0 | sharp, squared-off edges |
| 2-4 | slightly softened corners |
| 6-8 | gently rounded corners |
| 12-16 | generously rounded |
| 20+ | very round, pillow-like |
| 9999 | pill-shaped |

Then per component: **buttons** (shape, fill, label colour, hover), **cards** (radius, ground, border or shadow), **fields** (stroke, ground, label position).

### 5. Layout and depth

Margins, grid, section spacing ("tepi 80 di desktop, 20 di HP; setiap section satu layar"), and how layers are shown: flat, a hairline, whisper-soft shadows, or hard shadows.

## When extracting from someone else's design

1. Look at the big picture first: what makes it feel the way it does.
2. Collect repeated values (the same grey three times is a token; a one-off is noise).
3. Name colours by role, not just appearance.
4. Note the hierarchy: how importance is shown (size, weight, colour, space).
5. Don't copy what is wrong: where the old system breaks Taste (violet accents, five fonts, centred everything), record it, then say which parts you keep (usually the logo colour and the photos) and which Taste replaces.

## Rules

- Exact values always follow the descriptive name; a name alone or a hex alone is half a description.
- Explain why when it matters ("hijau tua karena warna cat toko").
- Use one set of words throughout: if it's "gently rounded" once, it isn't "soft corners" later.
- Don't use code words in the person-facing text (`rounded-xl`, `px-4`); tokens and values are for Koda's part of the handoff.
- Include subtle details that hold the look together: border colour, spacing rhythm, shadow quality.

Adapted from Google Labs' Stitch design-md skill (Apache-2.0, google-labs-code/stitch-skills), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
