---
name: pen-format
description: "Exactly how to write the OpenPencil scene graph the Design Studio draws: the document, every node type and property it understands, tokens and themes, components, the gotchas, and Nomi's rules for ids and size."
license: "Written for Nomi"
source: "@open-pencil/pen 0.16 and Nomi design_artifact.rs"
---
# The canvas format (.pen)

Exactly how to write the OpenPencil scene graph the Design Studio draws: the document, every node type and property it understands, tokens and themes, components, the gotchas, and Nomi's rules for ids and size.

Read this before your first save_preview in a chat. Write only what is listed here: a property the canvas doesn't know is ignored, so a guessed name (`background`, `bg`, `color` on a frame) draws nothing.

## The document

```json
{
  "version": "1.0",
  "themes": { "theme": ["light", "dark"] },
  "variables": {
    "color-background": { "type": "color", "value": [{ "value": "#F7F4EE" }, { "value": "#141311", "theme": { "theme": "dark" } }] },
    "space-4": { "type": "number", "value": 16 },
    "font-body": { "type": "string", "value": "Instrument Sans" }
  },
  "children": [ /* top-level frames: the design system, then each screen */ ]
}
```

- `children` holds the top-level frames. Each is one artboard on the canvas: a screen, or a part of the design system. Give each a `name` (its title in the Studio), a `width` (390 phone, 768 tablet, 1440 desktop) and an `x`/`y` so they don't overlap (leave 120 between them). Its height can be a number or `"fit_content"`.
- Several `.pen` files in one preview are merged into one canvas in path order: `children` are joined and `variables` combined. Keep `themes` and every variable in `canvas.pen`; put screens in more files (`screens-order.pen`, `screens-account.pen`) when one would pass the size limit.

## Nodes

Every node has `"type"` and `"id"`, and may have `"name"`, `x`, `y`, `width`, `height`, `opacity` (0–1), `rotation` (degrees), `enabled: false` (hidden), `clip: true` (crops children).

| type | What it is | Its own properties |
|---|---|---|
| `"frame"` | A box that lays out its children. Most of a design is frames. | `layout`, `gap`, `padding`, `justifyContent`, `alignItems`, `children`, `fill`, `stroke`, `cornerRadius`, `effect`, `reusable` |
| `"rectangle"` | A plain shape (a divider, a photo placeholder's tone, a swatch) | `fill`, `stroke`, `cornerRadius`, `effect` |
| `"ellipse"` | A circle or oval (an avatar, a status dot) | `fill`, `stroke` |
| `"text"` | Text | `content`, `fontFamily`, `fontSize`, `fontWeight`, `lineHeight`, `letterSpacing`, `textAlign`, `textAlignVertical`, `textGrowth`, `fill` (its colour) |
| `"path"` | A vector drawn from an SVG path, scaled to its `width`/`height` (icons, logos, marks) | `geometry` (SVG path data), `fill`, `stroke` |
| `"ref"` | A placed copy (instance) of a reusable frame | `ref`, `descendants`, plus anything you override |

`icon_font` exists too, but the Studio doesn't load icon fonts, so it shows the icon's name as words. Draw icons as `path` nodes instead (below).

### Layout (frames)

- `layout`: `"vertical"` (a column), `"horizontal"` (a row) or `"none"` (children placed by their own `x`/`y`). **A frame without `layout` is a horizontal row.** Say `"vertical"` for every column and `"none"` only where you position by hand.
- `gap`: space between children, a number or a `$token`.
- `padding`: `16`, or `[16, 24]` (top and bottom, left and right), or `[16, 24, 20, 24]` (top, right, bottom, left). Numbers or `$tokens`.
- `justifyContent` (along the layout): `"start"`, `"center"`, `"end"`, `"space-between"`.
- `alignItems` (across it): `"start"`, `"center"`, `"end"`, `"stretch"`, `"baseline"`. Left out, children sit at the start, not the centre.

### Size

- A number is fixed pixels.
- `"fill_container"` takes the room left in a parent with layout (`layout: "none"` parents ignore it).
- `"fit_content"` hugs the children. A frame with layout and no `width`/`height` hugs anyway.
- `"fill_container(320)"` and `"fit_content(48)"` add a fallback size.
- A number token (`"$size-avatar"`) works too.

### Paint

- `fill`: `"#1F6F5C"`, `"#1F6F5C80"` (alpha), a `"$color-…"` token, or a list of these painted bottom to top. **Fills are solid colours only**: a gradient or an image can't be drawn on this canvas. Where your design wants one, use the solid colour that carries it and note the intent for Koda in a caption.
- `stroke`: `{ "align": "inside", "thickness": 1, "fill": "$color-border" }`. `align` is `"inside"`, `"center"` or `"outside"`. `thickness` can be per side: `{ "bottom": 1 }` for a divider line. A stroke without a `fill` draws nothing.
- `cornerRadius`: `12`, a `$token`, or `[12, 12, 0, 0]` (top-left, top-right, bottom-right, bottom-left).
- `effect`: shadows only. `{ "type": "shadow", "color": "#0000001F", "offset": { "x": 0, "y": 8 }, "blur": 24, "spread": 0 }`, with `"shadowType": "inner"` for an inner one. A list for several. Blur and glass don't exist here.

### Text

- `content` is the words. `fontFamily` is a family name or a `"$font-…"` token. `fontSize` in px.
- `fontWeight`: 100–900, or `"light"`, `"medium"`, `"semibold"`, `"bold"`.
- `lineHeight`: below 5 it is a multiple of the size (`1.5`), otherwise px (`24`).
- `letterSpacing` in px (`-0.4` for a large heading).
- `textAlign`: `"left"`, `"center"`, `"right"`, `"justified"`.
- Text is one line that grows sideways unless you set `"textGrowth": "fixed-width"` with a `width` (a number or `"fill_container"`): then it wraps. Set it on every paragraph.
- Colour is the text's `fill`.
- The Studio shows the fonts it has (Geist, Geist Mono, Fraunces and the system's). Name the real font you chose anyway: Koda loads it in the app.

### Icons and marks

Draw an icon as a `path`: `{ "type": "path", "id": "icon-cart", "width": 20, "height": 20, "geometry": "M3 3h2l2.4 12.2a2 2 0 0 0 2 1.6h7.7a2 2 0 0 0 2-1.6L21 7H6", "stroke": { "align": "center", "thickness": 1.75, "fill": "$color-foreground", "cap": "round", "join": "round" } }`. Outline icons are stroked with no `fill`. Keep one stroke width and one icon size per role.

### Photos and pictures

The canvas can't hold an image. Draw the picture's frame at its real size and ratio, fill it with a tone from the palette, and put a short caption inside naming the photo that goes there: `Photo · nasi goreng on a banana leaf, top-down, warm daylight`. The caption is a note for the person and Koda, not copy on the page.

## Tokens and themes

- `variables` holds the tokens: `"type": "color"` (hex), `"number"` (spacing, sizes, radii) or `"string"` (font families). Use them anywhere as `"$name"`: fills, strokes, `gap`, `padding`, `cornerRadius`, sizes, `fontFamily`.
- Name colours by their shadcn role with a `color-` prefix, so each maps to the CSS variable Koda sets: `color-background`, `color-foreground`, `color-card`, `color-primary`, `color-primary-foreground`, `color-muted-foreground`, `color-border`, `color-ring`, `color-destructive` (the full set is in components-kit). Then `space-1`…`space-12`, `radius-sm`…`radius-2xl`, `font-heading`, `font-body`.
- Give every top-level frame its own `fill`: one without is painted the Studio's default background.
- Themes: add `"themes": { "theme": ["light", "dark"] }`; the axis must be named `theme`. Give a colour its value per theme as a list: `[{ "value": "#FFFFFF" }, { "value": "#0E0E10", "theme": { "theme": "dark" } }]`. The first entry is light.
- Put `"theme": { "theme": "dark" }` or `"theme": { "theme": "light" }` on **every** top-level frame once the document has themes: a theme set on one frame carries on to the frames after it.

## Components and instances

- A frame with `"reusable": true` is a component. Put components in the design system frames, named `<Component> / <Variant> / <State>` (`Button / Primary / Disabled`).
- Place one with `{ "type": "ref", "id": "checkout-pay", "ref": "button-primary" }`, where `ref` is the component's `id`. It takes the component's size, layout, paint and children.
- Override what differs on the instance itself (`width`, `fill`…) and inside it with `descendants`, keyed by the child's id in the component: `"descendants": { "button-label": { "content": "Bayar Rp 249.000" }, "button-icon": { "enabled": false } }`. A descendant override can set `content`, `fill`, `enabled`, `x`, `y`, `width`, `height`, `rotation`, `name`, or `children` (replaces them).
- Change a component once and every instance follows: use refs for every repeated part.

## Nomi's rules (save_preview refuses a file that breaks them)

- Every node has an `id` of lowercase letters, digits and dashes (`order-summary`, `btn-pay-2`), unique within its top-level frame. The same element on several screens keeps the same id: that is how the person's feedback finds it. Keep an element's id for as long as it exists.
- `children` of the document can't be empty, and the JSON must parse.
- At most 300 elements across the preview, 200,000 bytes a file, 600,000 bytes together, 20 files. The entry is `canvas.pen`.
- The top-level frames are the pages the Studio lists: a frame's `name` is its title, and an optional `"path"` names the page.

## A small screen

```json
{ "type": "frame", "id": "menu-phone", "name": "Menu — Phone", "x": 0, "y": 0, "width": 390, "height": "fit_content",
  "layout": "vertical", "gap": 24, "padding": [24, 20], "fill": "$color-background", "theme": { "theme": "light" },
  "children": [
    { "type": "text", "id": "menu-title", "content": "Roti hari ini", "fontFamily": "$font-heading", "fontSize": 28,
      "fontWeight": 600, "lineHeight": 1.2, "letterSpacing": -0.4, "fill": "$color-foreground" },
    { "type": "frame", "id": "menu-item", "name": "Menu item", "width": "fill_container", "layout": "horizontal",
      "gap": 12, "alignItems": "center", "padding": 12, "fill": "$color-card", "cornerRadius": "$radius-md",
      "stroke": { "align": "inside", "thickness": 1, "fill": "$color-border" },
      "children": [
        { "type": "rectangle", "id": "menu-item-photo", "width": 64, "height": 64, "cornerRadius": 8, "fill": "#D9C7A7" },
        { "type": "frame", "id": "menu-item-text", "layout": "vertical", "gap": 4, "width": "fill_container", "children": [
          { "type": "text", "id": "menu-item-name", "content": "Roti sobek cokelat", "fontFamily": "$font-body", "fontSize": 16, "fontWeight": 600, "fill": "$color-foreground" },
          { "type": "text", "id": "menu-item-price", "content": "Rp 18.000", "fontFamily": "$font-body", "fontSize": 15, "fill": "$color-muted-foreground" } ] },
        { "type": "ref", "id": "menu-item-add", "ref": "button-icon-add" } ] } ] }
```
