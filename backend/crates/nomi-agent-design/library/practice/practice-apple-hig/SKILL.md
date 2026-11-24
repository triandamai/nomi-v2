---
name: practice-apple-hig
description: "Apple's interface principles for phone apps, translated to the 390 canvas and shadcn parts: navigation, sheets, alerts, controls by job, progress, search and key patterns like undo and sign-in later. Read when an app screen must feel at home on an iPhone."
license: "MIT, Copyright (c) 2025 Raintree Technology (apple-hig-skills skill files; its Apple-owned reference material not used); via open-design (Apache-2.0)"
source: "https://github.com/raintree-technology/apple-hig-skills@a0f1e4e (skills/hig-*/SKILL.md only); open-design@5fdcc3d"
---
# iPhone conventions for app screens

Make app screens that an iPhone user understands without thinking: content over chrome, standard navigation, a clear current state, feedback for every action, and as few interruptions as possible.

Koda builds a web app (SvelteKit), so you aren't designing a native iOS app; you're designing screens that feel right on a phone, which is mostly an iPhone or an Android phone in Indonesia. Android and web rules are in `practice-platform-design`; the general screen rules in `taste-app-screens`. Only the principles from the source's own skill files are used here; Apple's guideline text itself is Apple's and not reproduced.

## Foundations

1. **Content over chrome.** Little visual clutter; subtle separators instead of heavy borders and fills.
2. **Accessibility from the start:** every control has a label, text can grow, motion has a reduced version, contrast holds.
3. **Semantic colours** that work in light and dark (your `$color-*` roles), never hard-coded per screen.
4. **One type hierarchy at readable sizes** (body 16-17, never light weights for body), few typefaces, one icon family at matching weight.
5. **Platform conventions:** standard look and behaviour, direct manipulation, clear feedback.
6. **Privacy:** ask for a permission only when it's needed, explain why, give value before asking for data.
7. **Text grows and varies:** Indonesian strings run longer than English; leave room and let text wrap.
8. **Motion with meaning:** it shows where things come from and go; a crossfade when motion is reduced.

## The phone frame

- 390 wide. Top: the status bar area (about 47 on current iPhones); bottom: the home-indicator area (34). Nothing tappable in either; content may scroll under them.
- One-handed reach: the most used controls in the lower and middle part of the screen; rare or destructive ones up top.
- Side padding 16-20 (`$space-4`/`$space-5`); targets at least 44×44 with 8 between.
- Avoid full-width buttons as a habit on iOS-style screens except the single main action at the bottom of a form or sheet.
- Draw the light and dark versions together, and one frame at large text size for the busiest screen.

## Navigation

| Need | Pattern | shadcn / kit part |
|---|---|---|
| 3-5 peer sections (Beranda, Pesanan, Produk, Akun) | bottom tab bar, icon plus short label, current tab marked | a bottom bar composed from kit parts (icon `path` plus label per item), named in the handoff as composed |
| going deeper (list, then detail) | top bar with the title and a back control on the left; content pushes in from the right | a top bar frame with a ghost icon `Button` for back |
| many sections on a wide screen | sidebar (768 and up) collapsing to the tab bar at 390 | `Sidebar` |
| a flat sequence (onboarding pages, a photo gallery) | page dots, current and total visible | small ellipse dots |

Keep navigation predictable: people always know where they are, how they got there and how to go back. Never hide the only navigation behind a menu icon when there are four or fewer sections. Large titles at the top of a list screen, shrinking to a compact top bar when scrolled, are the familiar iPhone pattern; draw both states.

## Overlays

| Use | When | Kit part |
|---|---|---|
| **Alert** | rarely: an error that needs attention, confirming a destructive action, something the person must acknowledge | `Alert Dialog / Destructive / Default`: short title, a sentence at most, specific verbs ("Hapus pesanan", "Batal"), never "OK" |
| **Sheet** | a focused task that keeps context: create an item, edit, a short multi-step form | `Drawer` or `Sheet` from the bottom, with a grabber, a title and a clear done or cancel |
| **Action sheet** | choosing among several actions, especially when one is destructive | `Drawer` with a list of actions; destructive in `$color-destructive`; cancel at the bottom, separated |
| **Popover** | extra info or options next to a control on wide screens only | `Popover` (at 390, use a sheet instead) |

Before any modal, ask whether it could be inline or undoable instead. Put destructive buttons where they won't be tapped by reflex, and always offer a cancel.

**Digit entry** (OTP, PIN): right-sized boxes, automatic advance, paste and autofill allowed (`practice-login-flow`).

## Controls by job

| Job | Control | Notes |
|---|---|---|
| on or off | `Switch` | in settings, takes effect immediately; in a form, on confirm |
| 2-5 equal, short options | segmented control (`Tabs` or `Toggle Group`) | the active one clearly marked |
| a long list of options, dates, times | picker (`Select`, `Calendar`, a wheel-like list in a sheet) | show the current value |
| a continuous value where precision doesn't matter | `Slider` | min and max labels |
| small exact steps (quantity) | stepper: minus, value, plus | sensible limits; the value always visible |
| short text | `Input` with the right keyboard | multi-line: `Textarea` |
| actions | standard button styles: filled for the main one, tinted or outline for secondary, plain for tertiary | one filled button per screen area |

Always show the current state: what is selected, on, or chosen.

## Status and progress

- Show progress for anything longer than a second or two.
- Known length: a progress bar (preferred: it feels faster and more honest). Unknown: a spinner.
- Put the indicator where the content will appear, not in a modal; never stack several.
- Don't hide the status bar except for full-screen media or games.

## Search

- Where people expect it: at the top of a list or in the top bar, results as they type.
- Scopes (chips or a segmented control under the field) narrow big result sets.
- An empty result suggests a correction or an alternative, never a blank screen.

## Patterns

- **Minimise modality;** use it only when attention is critical, a task must be finished or abandoned, or unsaved work is at risk.
- **Feedback for every action:** visible (and on a phone, a light haptic note for Koda on confirmations).
- **Undo over "Yakin?"**: a toast with "Urungkan" beats a confirmation dialog for reversible actions.
- **Launch fast:** no logo splash screen; open where the person left off.
- **Sign-in later:** let people look around (the menu, the catalogue) before asking for an account; offer the simplest sign-in.
- **Onboarding:** three screens at most, skippable; teach in context instead.
- **Progressive disclosure:** essentials first, details one tap away.
- **Respect attention:** no notifications for promotions; group them; give control.

## Content and system surfaces

- Empty states explain how to fill them, with one action.
- Charts and images get labels and descriptions.
- Notifications stand on their own: enough context to understand and act, with actions where useful ("Pesanan #212 siap diambil · Lihat").
- A home-screen icon (180×180, no transparency, one focal point) and a share card belong to every app (`practice-ui-constraints`).
- Widgets, live activities and watch complications are native-only; mention them only if the brief is a native app.

## Other Apple platforms

iPad: sidebars and multi-column layouts, pointer and keyboard too (your 768 frames). Mac: dense information is fine, menus and shortcuts matter. Watch, TV and Vision Pro have their own paradigms; out of scope for a web app.

Adapted from Raintree Technology's apple-hig-skills (MIT, Copyright (c) 2025 Raintree Technology; skill files only, the Apple HIG material in its references is © Apple Inc. and was not used), via open-design (nexu-io/open-design@5fdcc3d, Apache-2.0). Changed: rewritten for the OpenPencil canvas.
