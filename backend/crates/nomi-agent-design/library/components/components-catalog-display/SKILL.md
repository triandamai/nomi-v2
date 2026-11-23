---
name: components-catalog-display
description: "The shadcn-svelte components for overlays, feedback, data display, layout and chat: what each is for, its parts, variants and states, and when to pick it over a neighbour; actions, inputs and navigation are in `components-catalog`."
license: "MIT, Copyright (c) 2023 shadcn; MIT, Copyright (c) 2023 Hunter Johnston, CokaKoala"
source: "https://github.com/shadcn-ui/ui (skills/shadcn), https://www.shadcn-svelte.com"
---
# Components Catalog: Overlays, Feedback, Data, Layout

The shadcn-svelte components for overlays, feedback, data display, layout and chat: what each is for, its parts, variants and states, and when to pick it over a neighbour; actions, inputs and navigation are in `components-catalog`.

## Overlays

Every Dialog, Alert Dialog, Sheet and Drawer has a Title, even if it is visually hidden: draw it, and note "title hidden" if the design hides it. Draw overlays open, over a dimmed copy of the screen (scrim rectangle, black about 50% alpha), with the trigger visible.

**Dialog**: a modal window for a short focused task (edit a product, add an address). Parts: Trigger, Content, Header, Title, Description, Footer (actions, right-aligned on desktop, stacked full-width on mobile), Close (X, can be hidden). Sizing by width only (small, default, large). Long content: scrolling body with a sticky footer.

**Alert Dialog**: a confirmation that interrupts and must be answered ("Hapus produk ini? Pesanan lama tetap tersimpan."). Parts: Trigger, Content, Header, Media (optional icon), Title, Description, Footer, Cancel, Action. Sizes: default, sm. Action can take the destructive variant. No close X, no click-outside dismiss.

**Sheet**: a panel that slides from an edge and holds a whole sub-task or secondary nav (cart, filters, mobile menu, order detail beside a list). Parts: Trigger, Content, Header, Title, Description, Footer, Close. Side: top, right, bottom, left.

**Drawer**: a bottom sheet with a drag handle, made for touch. Parts: Trigger, Content, Header, Title, Description, Footer, Close. Direction: top, right, bottom, left (bottom is the usual). Options: snap points (half and full height), non-modal, nested drawers.

**Dialog vs Sheet vs Drawer vs Alert Dialog**

| Need | Use |
|---|---|
| Short task, centred focus, desktop | Dialog |
| Same task on phone | Drawer (Dialog on desktop, Drawer on mobile is the standard pair) |
| Long or side-by-side task, keep page context, filters, cart, mobile nav | Sheet |
| Must confirm or cancel a risky action | Alert Dialog |
| Small extra info or a mini form tied to one control | Popover |

**Popover**: a small floating panel anchored to a trigger (a filter, a colour pick, the Date Picker calendar). Parts: Trigger, Content, Header, Title, Description, Close. Align: start, center, end. Closes on outside click.

**Hover Card**: preview on hover (a customer's profile from their name). Desktop only; never hide needed info in it.

**Tooltip**: a short label for an icon-only button ("Salin tautan"). One line, no actions, never essential content (touch has no hover).

**Dropdown Menu**: a list of actions from a button (the "..." on an order row). Parts: Trigger, Content, Group, Label, Item, Shortcut, Separator, Sub / Sub Trigger / Sub Content, Checkbox Item, Radio Group / Radio Item. Item variants: default, destructive ("Hapus"). States per item: Default, Highlighted, Disabled, Checked.
- Pick for actions; Select for choosing a form value.

**Context Menu**: the same menu on right-click or long-press. Only as a shortcut; the actions must also exist somewhere visible.

**Command**: a searchable list of actions or items. Parts: Input, List, Empty, Group (with heading), Item, Separator, Shortcut. Inside a Dialog it is a command palette (Ctrl K) for admin tools; inside a Popover it is the Combobox list.

## Feedback

**Alert**: an inline message that stays on the page until the problem is gone ("Stok Kopi Susu tinggal 3"). Parts: icon, Title, Description, Action. Variants: default, destructive. Custom warning or success alerts need token pairs (note them).

**Sonner** (toast): a brief message after an action, bottom or top corner, auto-dismisses ("Pesanan #1042 tersimpan"). Types: default, success, info, warning, error, loading/promise (spinner that turns into success or error). Parts: icon, title, description, action button ("Urungkan"), close. Positions: top or bottom, left, center or right; pick one for the whole app (bottom-center on phones).

**Toast/Sonner vs Alert**: Sonner confirms something that just happened and goes away; Alert describes a state that persists or blocks. A failed save that needs fixing in the form is a Field error or an Alert, not only a toast.

**Progress**: a bar for known progress (upload 60%, profile 3 of 5 done). Show the number beside it.

**Spinner**: an unknown wait, small (inside a Button, Badge, Input Group, or centred in an Empty). Sizes by box: 12, 16, 24, 32.

**Skeleton**: grey placeholder blocks in the shape of content that is loading (card image, two text lines). Pick over a Spinner whenever the layout is known; draw one Skeleton version of each list or card screen.

**Empty**: the screen or panel with nothing in it yet. Parts: Header, Media (icon in a muted circle, or an avatar group), Title ("Belum ada pesanan"), Description ("Pesanan dari WhatsApp dan toko online muncul di sini."), Content (a Button "Bagikan link toko", or a search Input Group). Every list, table and search result needs one.

## Data display

**Card**: a contained group of related content (a product, a plan, a stat). Parts: Header (Title, Description, Action at top right), Content, Footer (actions). Size: default, sm. Image cards put the image above the Header. Use all the parts; do not dump everything into Content.

**Item**: a flexible row for lists (a customer, a file, a setting). Parts: Media (icon, avatar or image), Content (Title, Description), Actions, Header, Footer; group with Item Group and Item Separator. Variants: default, outline, muted. Sizes: default, sm, xs.
- Pick Item for lists of things with an action; Card for standalone blocks; Table for comparing many rows on the same columns.

**Table**: rows and columns (orders, stock). Parts: Header, Body, Row, Head, Cell, Footer, Caption. States per row: Default, Hover, Selected (checkbox column). **Data Table** is a Table plus sorting (header chevron), filtering (an Input above), column visibility (Dropdown Menu), pagination (below), row selection; it is a pattern, not a component. On phones, turn rows into Items or Cards.

**Avatar**: a person or brand image with a fallback of initials ("SR" for Siti Rahayu). Always draw the fallback state too. Avatar groups overlap with a "+3" count.

**Accordion**: stacked sections that expand (FAQ, shipping info). States per item: closed, open, Hover, Focus. One or many open at once.

**Collapsible**: a single show/hide region ("Lihat rincian biaya").

**Carousel**: horizontally swiped slides (product photos, testimonials). Parts: Content, Item, Previous, Next. Show partial next slide or dots so people know there is more.

**Chart**: area, bar, line, pie, radar and radial charts that use `$color-chart-1` to `$color-chart-5`, with Tooltip and Legend parts. Draw with paths and rectangles; label axes with real values (Rp 2,4 jt, Jan to Jun 2026).

**Aspect Ratio**: keeps images at a fixed ratio (1:1 product, 16:9 banner).

**Typography**: the type styles for headings, paragraphs, lists, quotes and inline code on content pages.

## Layout

**Separator**: a 1 px `$color-border` line between groups, horizontal or vertical.

**Scroll Area**: a region that scrolls inside a fixed box (a long list in a Popover or Sheet).

**Resizable**: panels split by a drag handle (list and detail side by side). Desktop tools only.

## Chat and AI

For assistant or customer-chat screens, shadcn-svelte has Message Scroller (scroll area that follows new messages), Message (a row with avatar and metadata), Bubble (the message surface), Attachment (a file chip with uploading and error states), Marker (a system note or date divider: "Hari ini"), Questionnaire, and Trace (assistant reasoning, tool calls and sources as lines). A separate AI Elements set adds Prompt Input, Reasoning, Tool, Sources, Inline Citation, Plan, Task, Queue, Suggestion, Confirmation and more. Use these names when you draw chat; see `references-patterns` for how to show their states.

Sources: shadcn-svelte docs, components pages (MIT, Copyright (c) 2023 Hunter Johnston and CokaKoala); shadcn/ui skill SKILL.md and design-system-page.md (MIT, Copyright (c) 2023 shadcn).
