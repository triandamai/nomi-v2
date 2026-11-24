---
name: practice-platform-design
description: "How one app design works on Android, iPhone and the browser: Material 3 colour roles, navigation by screen size, type scale and components, WCAG 2.2 web rules, and a cross-platform choice table. Read when an app must work on Android and the web."
license: "MIT, Copyright (c) 2026 (ehmo/platform-design-skills); via open-design (Apache-2.0)"
source: "https://github.com/ehmo/platform-design-skills@dc2be82 (skills/android, skills/web; skills/ios read for comparison); open-design@5fdcc3d"
---
# One design, three platforms

Draw one app that feels at home on an Android phone (most Indonesian users), an iPhone and a desktop browser: shared structure, platform-aware details, and the accessibility floor that all three require.

Koda builds a web app, so the browser rules are the law and the phone rules shape how it feels. iPhone details are in `practice-apple-hig`; the general app-screen rules in `taste-app-screens`.

## Cross-platform choices

| Need | Android (Material 3) | iPhone | Your 390 design |
|---|---|---|---|
| 3-5 top sections | navigation bar, always labelled, filled icon when selected | tab bar | one bottom bar, icon plus label, active item marked by shape and colour, not colour alone |
| more than 5 sections | modal drawer plus the bar | "More" tab | bar with 4 plus "Lainnya", or a drawer (`Sheet` from the left) |
| tablet and up (600+) | navigation rail | sidebar | `Sidebar` at 768 and 1440 |
| main action on a screen | one FAB, bottom-end, above the bar; extended with a label preferred | a button in the top bar or a bottom button | one main `Button`; a floating labelled button only on list screens where adding is the main job ("+ Produk") |
| brief feedback | snackbar at the bottom, above the bar, one action | toast or banner | Sonner toast above the bottom bar, auto-dismissing in 3-5 s, with "Urungkan" when reversible |
| critical decision | dialog | alert | `Alert Dialog` |
| supplementary content | bottom sheet with a drag handle | sheet | `Drawer` with a grabber; content scrolls if long |
| filters | filter chips in a row | segmented control or chips | `Toggle Group` chips in a horizontally scrolling row or wrapping, never stacked vertically |
| back | system back gesture or button; "up" arrow in the top bar | back control top left, edge swipe | a back icon top left; never trap back with "Yakin?" unless there's unsaved input |

## Material 3 colour roles, mapped to your tokens

| Material role | Your token |
|---|---|
| primary / on-primary | `$color-primary` / `$color-primary-foreground` |
| primary container / on | a soft tint of primary for selected chips and the nav active indicator: `$color-secondary` / `$color-secondary-foreground` |
| surface / on-surface | `$color-background`, `$color-card` / `$color-foreground` |
| surface variant, outline | `$color-muted`, `$color-border` |
| error / on-error | `$color-destructive` |
| inverse surface | the toast ground (dark on light theme) |

Rules: every foreground uses the matching "on" colour for its ground; primary is never a large background area; light and dark both exist (dark uses tonal surfaces, never pure black); people can choose light, dark or system in settings.

## Size classes and grid

| Class | Width | Margin | Gutter | Columns | Your frame |
|---|---|---|---|---|---|
| Compact | under 600 | 16 | 8 | 4 | 390 |
| Medium | 600-839 | 24 | 16 | 8 | 768 |
| Expanded | 840+ | 24 | 24 | 12 | 1440 |

Draw edge to edge (content may run under the status and navigation bars, controls may not).

## Type scale (Material defaults, for app screens)

| Role | Size / weight | Use |
|---|---|---|
| Display | 57, 45, 36 / 400 | onboarding, a big figure |
| Headline | 32, 28, 24 / 400 | screen titles, section headers, card titles |
| Title | 22 / 400; 16, 14 / 500 | top bar title; tabs and list titles |
| Body | 16, 14, 12 / 400 | text; 12 only for captions |
| Label | 14, 12, 11 / 500 | buttons; chips; timestamps |

Never body text under 12; labels no smaller than 11. Line height about 1.2-1.5 times the size on app screens (body text on reading pages 1.5+). Test at 200% text size: nothing clipped or overlapping. Taste's type scale and pairing replace Material's font; keep the role structure.

## Components

- **Top bar:** small for most screens, medium or large for a key screen's title, collapsing on scroll; at most 2-3 action icons, the rest in a "more" menu.
- **Bottom sheets:** visible drag handle; modal for supplementary content, persistent for content that stays.
- **Dialogs:** only for critical decisions; a concise question or statement as the title; dismiss on the left, confirm on the right as text buttons (on the web, `Alert Dialog` with outline and filled buttons is fine; keep the order consistent).
- **Snackbars and toasts:** brief and non-critical; never the only place critical information appears; one action at most.
- **Chips:** filter, input (tags the user typed), suggestion; in a scrolling row or a wrapping flow.
- **Waiting states:** acknowledge every action immediately (inline change, progress, a busy button), never silence.

## Accessibility floor (all platforms)

- **Targets:** 48×48 on Android, 44×44 on iPhone and the web, with spacing (WCAG's AA minimum is 24×24; aim for 44-48 anyway). Enlarge the tap area with padding rather than shrinking the visual.
- **Contrast:** 4.5:1 for normal text, 3:1 for large text (24+, or about 19 bold) and for meaningful icons, borders and focus rings.
- **Never colour alone** for meaning: add an icon or text.
- **Focus:** a visible focus indicator on every control; focus follows reading order; after a dialog closes, focus returns to what opened it; modals trap focus.
- **Names:** a visible label is part of the control's accessible name (voice control users say what they see); icon-only buttons get a label in the note.
- **Gesture zones:** no controls in the bottom 20 or the side 24 edges on Android (system gestures); every swipe action has a button alternative and swipe-to-delete is undoable.
- **Every control has a pressed state** (ripple on Android, highlight on iPhone; one pressed style on your canvas).
- **No flashing** more than three times a second.

## Web specifics

- **Forms:** a label for every field; the right input type and autofill hint (`practice-login-flow`); instructions next to the field; required fields marked; validate when the user leaves a field, and on submit. The source prefers **not disabling the submit button** (validate on submit and show errors); if you do disable it, say why next to it (`practice-ui-constraints`).
- **Typography:** body line height at least 1.5, paragraph spacing at least twice the font size, lines about 75 characters at most (Taste: 62), real quotes and dashes, tabular figures in data.
- **Navigation and state:** the address reflects the screen and filters; browser back works; the active nav item is marked; breadcrumbs on deep hierarchies; scroll position restored when going back.
- **Hover and focus parity:** anything shown on hover is also shown on focus and on tap; no hover-only disclosure.
- **Motion:** reduced-motion respected; only movement and opacity animate; transitions for state changes; no motion without meaning.
- **Dark mode:** follows the system preference, with contrast checked in both.
- **Locale:** dates, times and money formatted for Indonesia ("24 Nov 2026", "Rp 1.250.000", 24-hour time "09.30"); no text baked into images.
- **Installable app (PWA):** a name, icons, theme and background colours; standalone display for app-like tools, fullscreen only for games.

## Anti-patterns

Placeholder as label; focus outline removed with no replacement; zoom disabled; text under 12; a disabled submit with no explanation; colour alone for status; images without set dimensions; hover-only menus; icon-only navigation bars; more than one FAB; primary colour as a big background; a "Yakin keluar?" on back with nothing unsaved.

Adapted from ehmo's platform-design-skills (MIT, Copyright (c) 2026), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
