---
name: craft-interface-guidelines
description: "Vercel's Web Interface Guidelines turned into design rules you can draw: accessibility, focus, forms, content handling, navigation and state, touch, dark mode, Indonesian number and date formats, and what to annotate for Koda. Read it when drawing app screens, forms and flows."
license: "MIT, Copyright (c) 2025 Vercel Labs (web-interface-guidelines), via open-design (Apache-2.0)"
source: "https://github.com/vercel-labs/web-interface-guidelines @4e799d4 (pinned in open-design@5fdcc3d skills/web-design-guidelines)"
---
# Interface guidelines for the canvas

The interface rules every screen should meet, as things you draw or annotate rather than code: what each control needs, how forms behave, how screens survive real content, and which details Koda must build that the canvas can't show.

The source is a code review list. Koda enforces the code half; you own the half that is decided in the design. For each group: **Draw** is what must exist on the canvas, **Note** is what goes in the handoff.

## Accessibility and focus

- **Draw** a visible focus state for every control in the design system: `$color-ring`, 2px, offset 2 outside the control. Never a design where focus is invisible.
- **Draw** hover, focus and pressed states that are each more prominent than rest, not less.
- **Draw** headings in a clear h1-h3 order on every screen; one h1.
- **Draw** a visible label for every field (or a clear icon button whose name you note).
- **Note** the accessible name of every icon-only button ("Tutup", "Hapus item", "Buka menu").
- **Note** which updates are announced (toasts, inline validation, cart count).
- **Note** decorative icons as hidden from screen readers; meaningful photos get a description (your `Photo ·` note is a good start).
- **Note** a "Langsung ke konten" skip link on pages with a long nav.

## Forms

- **Draw** labels clickable with the control: a checkbox's label and box are one 44-tall hit area with no dead gap.
- **Draw** errors inline under the field that caused them, with the fix, and the field border in `$color-destructive`. On submit, the first error is the one in view.
- **Draw** the submit button enabled until the request starts, then a loading state ("Menyimpan…") on the button itself; not a disabled button the person can't explain.
- **Draw** placeholders as an example of the pattern ending in an ellipsis ("08xx xxxx xxxx…"), never as the label.
- **Draw** a warning before leaving with unsaved changes when the form is long ("Perubahan belum disimpan. Tinggalkan halaman?").
- **Note** the field type for each input so phones show the right keyboard: phone (WhatsApp numbers), email, number (quantities), postal code is not needed (Taste: no postcodes).
- **Note** that paste is never blocked, and that email, codes and usernames have spellcheck off.
- **Note** autofill names (name, email, tel, address) for checkout and booking forms.

## Content handling

- **Draw** every text container for long, average and very short content: truncate with an ellipsis, clamp to two lines, or wrap; decide per element and draw the long case.
- **Draw** the empty state of every list and table: never a broken layout for an empty array.
- **Draw** numbers that are compared or stacked in tabular figures.
- **Draw** headings balanced (no single word on the last line).
- **Note** image sizes are fixed (your placeholder's size and ratio) so nothing jumps while loading; the hero photo loads first, photos below the fold later.

## Navigation and state

- **Draw** the current page in the nav as a distinct state.
- **Draw** a way back from every screen (a back link, a breadcrumb, the logo to home).
- **Draw** destructive actions with a confirmation dialog or an undo toast ("Pesanan dihapus. Batalkan"), never immediate.
- **Note** which states belong in the URL so a link reopens the same view: filters, tabs, page number, an open product.
- **Note** that links are real links (open in a new tab works) and actions are buttons.

## Touch and layout

- **Draw** tap targets 44 tall and 8 apart on phones.
- **Draw** the phone frame with the safe areas in mind: nothing important in the top 44 (status bar) or bottom 34 (home bar) of a full-bleed 390 screen; the phone action bar sits above the home bar.
- **Draw** sheets, drawers and dialogs that scroll inside themselves; the page behind does not scroll.
- **Note** no zoom blocking, no double-tap delay, and autofocus only on a desktop screen with one main input.
- **Note** no horizontal scroll at 390 except a deliberate row you drew as scrolling.

## Dark mode and theme

- **Draw** both themes for every screen you mark as themed; native controls (select, date picker) styled in both, not left white in dark mode.
- **Note** the browser bar colour matches `$color-background` of the active theme.

## Locale: Indonesian formats

- **Money:** "Rp 18.000" (non-breaking space, dot for thousands, no decimals for rupiah); "Rp 1,2 jt" only in tight charts with the full value in the tooltip.
- **Dates:** "Senin, 24 November 2026"; short "24 Nov 2026"; times "14.00 WIB" (state the zone: WIB, WITA, WIT).
- **Numbers:** "1.250 pesanan", decimals with a comma "4,8".
- **Phone:** show as the person types it, grouped "0812 3456 7890"; never invent one (Taste house rule 9).
- **Note** that Koda formats these with the locale tools, never hand-built strings; brand names and codes are marked not-to-translate.

## Content and copy (UI words)

- Active voice, second person ("Simpan perubahan", "Kamu bisa ubah nanti" or "Anda", whichever voice the brand chose; never both).
- Numerals for counts: "8 pesanan", not "delapan pesanan".
- Specific button labels: "Simpan alamat" not "Lanjut"; "Kirim pesanan" not "OK".
- Error messages say what happened and what to do next.
- **Taste wins:** the source asks for Title Case on headings and buttons; Taste and Indonesian usage both use sentence case.

## Anti-patterns to catch on the canvas

- An icon button with no name in the notes.
- A field with only a placeholder and no label.
- An error shown only as a red border, or only as a toast far from the field.
- A destructive button with no confirm or undo.
- A table without its empty state; a card that breaks with a long name.
- A disabled primary button with no reason shown.
- Focus states missing from the design system frame.

Adapted from Vercel's Web Interface Guidelines (MIT, Copyright (c) 2025 Vercel Labs), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
