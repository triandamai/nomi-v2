---
name: ux-laws
description: "The laws of UX turned into composition moves with numbers: spacing for groups, how many options a screen offers, where targets sit, honest progress, the end of a flow. Read when you lay out an app screen, a pricing block, a menu or a multi-step flow."
license: "Apache-2.0, Copyright 2026 Open Design contributors"
source: "https://github.com/nexu-io/open-design at commit 5fdcc3db1e31bd4fa70f673041b78e87977507c3 (craft/laws-of-ux.md)"
---
# Laws of UX as layout moves

Each law below is one move you can see on the canvas. They are guidance, not gates; Taste's rules and numbers win where they speak.

## Grouping

| Law | Move |
|---|---|
| Proximity | 8–12 between things in a group (`$space-2`, `$space-3`), 32–48 between groups (`$space-8`, `$space-12`). Even spacing everywhere reads as nothing grouped. |
| Similarity | Equal things look equal: every list row the same component, every secondary button the same variant. Only the one item meant to stand out differs. |
| Common region | Enclose only when space is not enough: a Card with 16+ padding and a 1px `$color-border` hairline. A screen where every block is boxed loses the signal (Taste: two boxes deep at most). |
| Connectedness | A line or rail ties steps together: the stepper's line, one toolbar holding related actions. Stronger than spacing; use it for flows and comparisons. |
| Good figure | Everything on the grid and the content edges; a break that means nothing looks like a mistake. |

## Attention

- **One strongest contrast per view.** The filled primary button gets it; everything else steps down in weight. Red dots and badges everywhere train people to ignore them.
- **The one that differs is remembered (von Restorff).** The recommended plan, the active nav item, the warning row. Pair the difference with something besides colour: position (the middle plan), the filled button, a plain line ("Kami sarankan untuk toko dengan 1–3 kasir"). No badge (Taste), no invented "Terlaris".
- **Polish buys patience.** Good type, calm space and one accent make small friction forgivable; they never replace the states in `ux-states`.

## Choosing

| Law | Move |
|---|---|
| Hick, choice overload | 3–5 equal options in view; pricing 3–4 plans with exactly one recommended; settings in 5 named groups or fewer; sidebar 4–8 items, phone bottom bar 4–5. Put the rest behind "Lainnya" or a Dropdown Menu, never hidden without a way in. |
| Anchoring | The first number seen sets the scale: show the recommended plan where the eye lands first; give savings in rupiah ("Hemat Rp 120.000 setahun"), not only "-20%"; preselect the safer choice in a Radio Group (pickup, not express shipping, when cost is unknown). |
| Pareto | Name the 2–3 actions most people come for and make them big; the long tail goes to an overflow menu, the footer or Settings. |
| Tesler | Complexity can't vanish, only move. Put help where the hard part is: helper text under the field, a sensible default, a one-line hint in the empty state. |
| Occam | List the elements the task needs; cut any decoration that serves no task. |

## Memory

- **About four chunks at a time.** Group 30 settings as Akun / Toko / Pembayaran / Notifikasi / Hapus akun. Chunk numbers: "0812-3456-7890", "BCA 123 456 7890".
- **Recognise, don't recall.** Show earlier choices on the next step (the order summary on the payment step), keep applied filters visible as chips, keep the person's last selections.
- **Ends are remembered (serial position).** Most-used nav items first and last (Beranda first, Akun last on the bottom bar); utilities in the middle.
- **Peak and end.** The end of a flow is what people remember. Design the success screen as a real screen: what happened, the facts (order number, total Rp 249.000, "Dikirim 26 Jul 2026"), the next step. Middle steps stay calm. No confetti, no "Terima kasih!" modal.
- **Unfinished pulls back (Zeigarnik) and the goal speeds people up (goal gradient).** Show honest progress in multi-step flows: "Langkah 2 dari 4", completed steps ticked only when they are really done (a saved profile counts). Never fake progress, streaks or nag counters.

## Hands and speed

- **Fitts:** big and near is fast. On a phone the main action sits in the thumb zone (the bottom action bar, a full-width button 48 tall); on desktop the page's one primary action on the right of the page head. Destructive actions sit away from the primary one. Targets 44 tall (48 on a place), 8+ apart (`ux-accessibility`).
- **Doherty:** answer within a second. Every press shows its Active state at once; anything slower follows the loading ladder in `ux-states`.
- **Postel:** take input in any reasonable shape (dashes, spaces, +62, "1,5 kg" or "1.5 kg"), show one clean format back.
- **Parkinson:** prefill what you know (last address, today's date, the saved WhatsApp number) so a checkout ends sooner than expected.

## Expectation

- **Jakob:** people expect your app to work like the ones they use daily. Logo top left goes home, cart top right, search with a magnifier, a bottom bar in a phone app; in Indonesia, WhatsApp for contact and QRIS among the payment methods. Be new only where it pays.
- **Mental model:** when the person names an app their customers know (Tokopedia, GoFood), match its interaction grammar, never its look.
- **Active user:** nobody reads a tutorial. Put the guidance where the action is (empty-state line, helper text), no onboarding slides.

## Folklore to ignore

- "44 px is the legal minimum": the AA floor is 24×24; 44 is Taste's and Apple's bar, and yours.
- "Doherty means 400 ms": the paper has no 400.
- "Seven plus or minus two means menus of seven": Miller wrote about chunks; plan for about four.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0, © 2026 Open Design contributors): craft/laws-of-ux.md. Changed: rewritten for the OpenPencil canvas.
