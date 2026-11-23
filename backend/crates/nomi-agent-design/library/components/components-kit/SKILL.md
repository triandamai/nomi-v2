---
name: components-kit
description: "How you build the design system frame the shadcn way: semantic `$token` variables with light and dark values, a radius scale, reusable component frames named `<Component> / <Variant> / <State>`, a pinned state matrix, and instances placed with `ref`."
license: "MIT, Copyright (c) 2023 shadcn; MIT, Copyright (c) 2023 Hunter Johnston, CokaKoala"
source: "https://github.com/shadcn-ui/ui (skills/shadcn), https://www.shadcn-svelte.com"
---
# Components Kit

How you build the design system frame the shadcn way: semantic `$token` variables with light and dark values, a radius scale, reusable component frames named `<Component> / <Variant> / <State>`, a pinned state matrix, and instances placed with `ref`.

## Why the shadcn way

Koda builds with shadcn-svelte (shadcn/ui on React). Every shadcn component reads the same small set of semantic tokens and exposes the same variant names. If your canvas uses those exact token roles and variant names, Koda can map each frame to a real component without guessing. Draw nothing that has no shadcn home unless the brief needs it; when it does, compose it from kit parts and say so in the handoff.

Order of work: foundations first (tokens, type, radius, elevation), then primitives, then composed components, then screens. A screen that needs a raw hex value means the token set is incomplete: add the token, do not hard-code.

## 1. Semantic tokens

Define each as a `$variable` with one value per theme (light, dark). Name them exactly as shadcn does, so `$color-primary` becomes `--primary` in code. Every surface token has a `-foreground` partner for the text and icons that sit on it.

| Token | Role on the canvas |
|---|---|
| `$color-background` / `$color-foreground` | Page floor and default text |
| `$color-card` / `$color-card-foreground` | Card surfaces |
| `$color-popover` / `$color-popover-foreground` | Floating layers: menus, dialogs, popovers |
| `$color-primary` / `$color-primary-foreground` | Main action fill and its label |
| `$color-secondary` / `$color-secondary-foreground` | Tinted secondary action |
| `$color-muted` / `$color-muted-foreground` | Soft bands, placeholders, helper text |
| `$color-accent` / `$color-accent-foreground` | Hover wash, active tab, highlighted menu row |
| `$color-destructive` | Errors, delete actions, invalid borders |
| `$color-border` | Hairlines and card outlines |
| `$color-input` | Field borders |
| `$color-ring` | Focus ring |
| `$color-chart-1` … `$color-chart-5` | Data series, in that order |
| `$color-sidebar`, `$color-sidebar-foreground`, `$color-sidebar-primary`, `$color-sidebar-primary-foreground`, `$color-sidebar-accent`, `$color-sidebar-accent-foreground`, `$color-sidebar-border`, `$color-sidebar-ring` | App sidebar rail |

Extra roles the brief needs (a warning yellow, a success green, a brand accent) get their own pair, named the same way: `$color-warning` / `$color-warning-foreground`, `$color-success` / `$color-success-foreground`. Tell Koda in the handoff; these are custom tokens, not shadcn defaults.

Rules:
- Pick values for the person's brand, not shadcn's grey defaults. Start neutral, then set `$color-primary` (and `$color-ring` near it) from the brand colour.
- Check every surface/foreground pair for at least 4.5:1 contrast at body size. A brand primary that fails behind 14 px labels is a real problem: darken it or switch the label colour.
- Dark theme is its own set of values, not an inversion: `$color-card` sits one step lighter than `$color-background`, `$color-border` and `$color-input` are low-alpha white, `$color-destructive` gets lighter to stay readable.
- Solid fills only. Hex with alpha is fine for `$color-border`, `$color-input` and overlays.

## 2. Radius, elevation, type

**Radius.** One base value, `$radius`, plus a ladder. Use each step for the same jobs every time:

| Step | Typical value | Used by |
|---|---|---|
| `$radius-sm` | 4 | Small accents, checkbox |
| `$radius-md` | 6 | Menu, select and command rows |
| `$radius-lg` | 8 | Buttons, inputs, selects, tabs, popovers |
| `$radius-xl` | 12 | Inner panels, dialogs |
| `$radius-2xl` | 16 | Cards, frames, photos, toasts |
| pill | 9999 | Badges, pill buttons if the brand wants them |

The radius ladder per trade comes from taste-cards-icons; where this file and a taste-* skill differ, Taste wins. Rounder brand (friendly cafe, kids' class): raise every step together. Sharper brand (law office, workshop): lower every step together. Never mix ladders.

**Elevation.** Four levels, drop shadow only: flat (page, bands), hairline (cards: `$color-border` stroke, no shadow or a faint one), soft (popovers, menus, toasts), deep (dialogs, sheets). Overlays also get a scrim: a full-frame rectangle in black at about 50% alpha behind the panel.

**Type.** A short scale named by job (display, h1, h2, h3, body, small, caption, mono). Control labels and card titles sit at body size. Set line height on every text style.

**Control height.** Buttons, inputs, selects, toggles and tab lists share one height per size (44 default and 48 on a place page, as taste-cards-icons and taste-check require; 36 sm only for dense desktop tables and toolbars; the tap area stays 44 on a phone). If the button height changes, change the fields too, or forms misalign.

## 3. What to draw as reusable frames

Draw each as a frame with `reusable: true`, auto layout, and token fills only. Group them on the design system frame by job, in this order:

| Group | Components |
|---|---|
| Actions | Button, Button Group, Toggle, Toggle Group, Badge |
| Inputs | Field (label + control + description + error), Input, Input Group, Textarea, Select, Native Select, Combobox, Checkbox, Radio Group, Switch, Slider, Input OTP, Calendar |
| Navigation | Tabs, Breadcrumb, Pagination, Navigation Menu, Sidebar |
| Data display | Card, Item, Table, Avatar, Accordion, Carousel, Chart, Separator |
| Feedback | Alert, Sonner toast, Progress, Spinner, Skeleton, Empty |
| Overlays | Dialog, Alert Dialog, Sheet, Drawer, Popover, Tooltip, Dropdown Menu, Command |

Draw only what the app uses, plus Button, Input, Card and one overlay so the person sees the system. See `components-catalog` for each component's parts and variants.

Build composed components from primitives: a Card's footer holds Button instances, a Field holds an Input instance. That way one change to `Button / Default / Default` updates every card.

## 4. Naming

Name every reusable frame `<Component> / <Variant> / <State>`, using shadcn's words in Title Case:

- `Button / Default / Default`, `Button / Destructive / Disabled`, `Button / Outline / Hover`, `Button / Ghost / Focus`
- `Input / Default / Error`, `Input / Default / Disabled`
- `Badge / Secondary / Default`
- `Tabs / Line / Default`, `Alert / Destructive / Default`

Size, when it matters, goes after the variant: `Button / Default / Sm / Default`, `Button / Outline / Icon / Hover`. Component names are shadcn's (Sheet, not "Side panel"; Sonner, not "Snackbar"). States use these words only: Default, Hover, Focus, Active, Disabled, Error, Loading, Selected, Open. Each frame keeps its lowercase slug id as well (`button-destructive-disabled`).

## 5. The state matrix

Every interactive primitive shows its states side by side, pinned, with a caption row above (never write the state inside the component).

| State | How to draw it |
|---|---|
| Default | Resting tokens |
| Hover | Fill one step darker or `$color-accent` wash (ghost, outline, menu rows) |
| Focus | Focus-visible: 3 px `$color-ring` outline at about 50% alpha outside the control, plus a `$color-ring` border on fields |
| Active | Pressed: slightly darker than hover; no scale on canvas (note "press scale" for Koda if wanted) |
| Disabled | Whole control at about 50% opacity, no hover |
| Error | Fields only: `$color-destructive` border, `$color-destructive` ring at low alpha, message below in `$color-destructive` |
| Loading | Buttons: Spinner icon before the label, control disabled, label kept ("Menyimpan…") |
| Selected / Open | Toggles, tabs, radios, checkboxes, select triggers: the on state |

Matrix shapes:
- Button: rows = default, secondary, outline, ghost, destructive, link; columns = Default, Hover, Focus, Active, Disabled, Loading. Then a size ladder (sm, default, lg, icon) and an icon row (leading, trailing, icon-only).
- Fields (Input, Textarea, Select trigger, Combobox, Input OTP): columns = Default, Hover, Focus, Disabled, Error, plus Filled.
- Checkbox, Radio, Switch, Toggle: off and on, each with Hover, Focus, Disabled.
- If a state is meant to look the same (a brand that darkens only on press), keep that column identical. It shows the choice was made.

Use real content with awkward cases: a long label ("Konfirmasi pesanan & kirim invoice"), an empty table that falls back to Empty, a price that wraps (Rp 1.249.000), an error ("Nomor WhatsApp belum lengkap").

## 6. Placing instances

On screens, place components with `ref` to the reusable frame's id, then change only what differs through `descendants`, keyed by the child's id: the label text, an icon glyph, a fill token for one state. Rules:
- Swap state by pointing `ref` at the other state's frame (`Button / Default / Disabled`), not by recolouring a Default instance.
- Override content (text, icon), never tokens of the brand. If you are overriding colours on many instances, you need a new variant frame.
- Keep the instance id a lowercase slug that is the same on every screen showing that element (`checkout-pay-button`).
- One primary Button per group. Secondary actions are outline or ghost.

## 7. Before you save

- Every fill and stroke is a `$token` (no raw hex on screens).
- Light and dark both render; flip one screen to dark and look.
- Contrast pairs pass 4.5:1 for body text.
- Every interactive primitive used on a screen has its matrix on the design system frame.
- Each frame name follows `<Component> / <Variant> / <State>`.
- Motion is a one-line note, not drawn (see `components-handoff`).

Sources: shadcn/ui skill: SKILL.md, design-system.md, design-system-page.md, customization.md (MIT, Copyright (c) 2023 shadcn); shadcn-svelte theming docs (MIT, Copyright (c) 2023 Hunter Johnston and CokaKoala).
