# Components Catalog: Actions, Inputs, Navigation

The shadcn-svelte components for actions, forms and navigation: what each is for, its parts, variants, sizes and states, and when to pick it over a neighbour; overlays, feedback, data display and layout are in `components-catalog-display`.

Names are shadcn-svelte's. Parts are the pieces you draw as child frames, named the same way (`Card.Header`, `Dialog.Footer`) so Koda sees the structure. "States" lists what you show in the matrix (see `components-kit`).

## Actions

**Button**: any click that does something (save, pay, open a dialog). Also renders as a link when it navigates.
- Variants: default (primary fill), secondary (tinted), outline (border on background), ghost (no fill until hover), destructive, link (text with underline on hover).
- Sizes: xs, sm, default, lg; icon-only: icon-xs, icon-sm, icon, icon-lg.
- Parts: optional leading or trailing icon, label.
- States: Default, Hover, Focus, Active, Disabled, Loading (a Spinner in place of the leading icon, button disabled; there is no loading variant), Error (invalid outline, rare).
- Pick: one default per group ("Bayar Rp 249.000"); outline or ghost for "Batal"; destructive only for irreversible actions; link for inline text actions.

**Button Group**: buttons joined into one control (split "Simpan" + dropdown arrow, or "Hari ini | Minggu | Bulan" when each is an action).

**Toggle**: one on/off button (Bold, "Tampilkan harga"). Variants: default, outline. Sizes: sm, default, lg. States: off, on, Hover, Focus, Disabled.

**Toggle Group**: 2 to 7 options where one (single) or several (multiple) can be on. Variants: default, outline. Sizes: sm, default, lg. Spacing: joined or gapped. Orientation: horizontal, vertical.
- Pick over a row of Buttons with a fake active state, always. Pick over Tabs when it filters or sets a value rather than switching a whole panel.

**Badge**: a small status or count label ("Lunas", "Baru", "3"). Variants: default, secondary, destructive, outline, ghost, link. No sizes. Can hold an icon or a Spinner.
- Pick for status on rows and cards. Custom roles (success green, warning yellow) need a token pair and a new variant name in the handoff.

**Kbd**: a keyboard key hint ("Ctrl K"). Only on desktop-heavy tools.

## Inputs and forms

Lay every form out as **Field** groups. A Field is the wrapper: `Field.Label`, the control, `Field.Description` (helper text), `Field.Error` (message). Related checkboxes or radios go in `Field.Set` with a `Field.Legend`. Orientation: vertical (default), horizontal (label left, for settings rows), responsive. Field states: Default, Disabled (whole field dimmed), Error (control gets the `$color-destructive` border, Field.Error shows below).

**Input**: single-line text, number, email, phone. States: Default, Hover, Focus, Filled, Disabled, Error, Read-only.

**Input Group**: an Input or Textarea with attached extras: a prefix text ("Rp", "+62"), an icon, a button ("Cek"), a Spinner, a Kbd. Addon positions: inline-start, inline-end, block-start, block-end. Buttons inside use the Input Group button (default, secondary, ghost; xs, sm, icon-xs, icon-sm).
- Pick whenever something sits inside the field's border: search with an icon, price with "Rp", phone with "+62", password with an eye toggle.

**Textarea**: multi-line text (address, order notes). Same states as Input.

**Select**: pick one value from a short fixed list (6 to 15 options). Parts: Trigger (shows Value or placeholder), Content, Group, Group Heading, Item (with check mark when selected), Separator, scroll buttons. States: trigger Default, Hover, Focus, Open, Disabled, Error; item Default, Highlighted, Selected, Disabled.

**Native Select**: the browser's own select. Pick on very simple mobile forms where the system picker is friendlier.

**Combobox**: a searchable select, composed from Popover + Command + Button. Parts: trigger Button showing the choice, Popover content with Command.Input (search), Command.List, Command.Empty ("Tidak ditemukan"), Command.Group, Command.Item. States: closed, open with results, open with no match, Disabled, Error.
- Pick for long lists: city, product SKU, customer name.

**Select vs Combobox vs Radio Group**

| Options | Use |
|---|---|
| 2 to 5, all worth seeing at once | Radio Group (or Toggle Group if compact) |
| 6 to 15, short labels | Select |
| 15+, or the person types to find | Combobox |
| Several at once | Checkbox list in a Field.Set, or a multi Combobox with Badges |

**Checkbox**: independent yes/no, or several from a list ("Saya setuju dengan syarat"). States: unchecked, checked, indeterminate, each with Hover, Focus, Disabled, Error.

**Radio Group**: exactly one of a few visible options. Parts: Radio Group root, Item, each with a Field.Label. States: off, on, Hover, Focus, Disabled, Error. Choice-card layout: wrap each item in a bordered label (shipping "Reguler Rp 15.000 / Kilat Rp 30.000"), selected card gets a `$color-primary` border.

**Switch**: an instant on/off setting that applies without a save button ("Terima pesanan online"). States: off, on, Hover, Focus, Disabled.
- Pick Switch when the change applies at once; Checkbox when the form has a submit.

**Slider**: a value or range on a scale (price filter Rp 50.000 to Rp 500.000). States: Default, Hover, Focus (on the thumb), Disabled. Show the value beside it.

**Input OTP**: a one-time code. Parts: Group, Slot (one per digit), Separator. States: empty, active slot (caret, ring), filled, Error (all slots with `$color-destructive` border), Disabled.

**Calendar**: a month grid to pick a date. Caption layout: label, or dropdown month/year (use dropdown for birth dates). States per day: Default, Hover, Today, Selected, Outside month (muted), Unavailable (struck through or muted, for booked slots). **Range Calendar** picks a start and end, with in-between days on `$color-accent`.

**Date Picker**: not its own component: a Button or Input Group trigger ("26 Jul 2026") + Popover + Calendar. Draw it closed and open. Variants worth drawing: single date, range ("26 Jul – 2 Agu 2026"), date of birth (dropdown caption), with time.

**Label**: text tied to a control. Inside Field, use Field.Label.

## Navigation

**Tabs**: switch between panels of one page ("Produk | Ulasan | Info"). Parts: List, Trigger, Content. List variants: default (pill segment on `$color-muted`), line (underline). Orientation: horizontal, vertical. States: Trigger Default, Hover, Focus, Selected, Disabled.
- Pick over Toggle Group when each option has its own content panel; over Navigation Menu when it is inside a page, not between pages.

**Breadcrumb**: where you are in a hierarchy ("Beranda / Produk / Kopi Susu Gula Aren"). Parts: List, Item, Link, Page (current, not a link), Separator, Ellipsis (collapsed middle, can open a Dropdown Menu). Use on catalogs and admin pages three levels deep or more.

**Pagination**: move through pages of results. Parts: Content, Item, Link (active page gets the outline style), Previous, Next, Ellipsis. Draw the active page and an ellipsis. Pick over infinite scroll for tables and order lists people come back to.

**Navigation Menu**: a website's top nav with dropdown panels. Parts: List, Item, Trigger (with chevron), Content (the panel), Link, Indicator, Viewport. States: Trigger Default, Hover, Open, Focus. On mobile it becomes a menu Button that opens a Sheet.

**Menubar**: desktop-app menus (File, Edit). Rarely right for a small business site.

**Sidebar**: an app's main navigation rail (dashboard, admin). Parts: Provider, Root, Header (logo, workspace), Content, Group, Group Label, Group Action, Menu, Menu Item, Menu Button (isActive), Menu Action, Menu Badge (count), Menu Sub (nested), Footer (account), Trigger (collapse button), Rail, Inset (the page beside it).
- Root options: side left or right; variant sidebar, floating, inset; collapsible offcanvas, icon (shrinks to an icon rail), none.
- States: Menu Button Default, Hover, Active (current page), Focus; whole sidebar expanded, collapsed to icons, mobile (opens as a Sheet).
- Uses the `$color-sidebar-*` tokens. Pick for apps with 5+ sections; a site with 3 to 5 pages uses Navigation Menu, a phone-first app may use a bottom bar built from Toggle Group or Buttons (note it in the handoff).

Sources: shadcn-svelte docs, components and theming pages (MIT, Copyright (c) 2023 Hunter Johnston and CokaKoala); shadcn/ui skill SKILL.md and design-system-page.md (MIT, Copyright (c) 2023 shadcn).
