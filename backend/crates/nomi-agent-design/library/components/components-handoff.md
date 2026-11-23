# Components Handoff

How a component on your canvas maps to the shadcn-svelte component Koda installs, and what you must annotate on each one (variant, size, states, token roles, motion intent) so Koda builds it without guessing.

## The mapping is the frame name

`<Component> / <Variant> / <State>` reads straight into code: the Component is the shadcn-svelte component (Koda adds it by its kebab-case name), the Variant is its variant value, the State is something Koda wires up, not a separate component.

| Canvas frame | Koda installs | Variant / size | Note |
|---|---|---|---|
| `Button / Outline / Sm / Hover` | button | variant outline, size sm | hover comes from the token |
| `Button / Default / Loading` | button + spinner | default, disabled while loading | no loading prop exists |
| `Input / Default / Error` | field + input | field invalid, control invalid | message in Field.Error |
| `Tabs / Line / Default` | tabs | list variant line | |
| `Alert / Destructive / Default` | alert | destructive | |
| `Sheet / Right / Open` | sheet | side right | |
| `Combobox / Default / Open` | popover + command + button | | composed, not one component |
| `Date Picker / Range / Open` | popover + range-calendar + button | | composed |
| `Sonner / Success` | sonner | toast type success | one Toaster in the layout |

Parts map the same way: a child frame named `Card.Footer` is the Card's footer part. Name parts with shadcn's part names; do not invent "Card bottom".

For React projects the names are the same in shadcn/ui; Koda handles the import style.

## What to annotate

Put a short note on each component on the design system frame, and on any screen instance that departs from it. Keep it to the facts below.

1. **Component and variant.** Already in the name. Add the size if not default.
2. **States shown.** List which states you drew. Anything missing is Koda's default; say so ("hover: default").
3. **Token roles.** Only when a part uses a non-obvious role: "price text `$color-foreground`, strike-through old price `$color-muted-foreground`". Custom tokens you added (`$color-warning`, `$color-success`) are listed once on the design system frame with light and dark values.
4. **Composition.** For composed pieces, name the parts in order: "Card: Header (Title, Action = Badge secondary), Content (Item list), Footer (Button default full width)".
5. **Responsive change.** When a piece changes on phone: "Dialog on desktop, Drawer on mobile", "Table becomes Item list under 640", "Navigation Menu becomes Sheet left".
6. **Motion intent.** One line, as words, since the canvas has no motion: what moves, why, and roughly how fast. Example: "Cart Sheet slides in from the right, about 200 ms, ease-out." "Toast slides up and fades." "Price counts up when the total changes." Mark anything optional as optional, and respect reduced motion.
7. **Content rules.** Truncation and limits: "product name max 2 lines, then ellipsis", "Rp format with dots: Rp 249.000".
8. **Behaviour that is not visible.** Disabled until valid, what Enter does, what the empty state links to.

## Things that have no shadcn component

Some patterns are compositions (Data Table, Date Picker, Combobox, a stepper, a bottom tab bar, a pricing table). Name them plainly, list the shadcn parts they are built from, and draw their states like any primitive. If an idea came from a reference library (beUI, rare-ui, Beautiful UI), describe the behaviour in your own words and never name a component to install from it (see `references-rules`).

## Checklist before approval

- Every reusable frame name maps to a shadcn-svelte component or a named composition.
- Each interactive component has its states drawn or marked default.
- Custom tokens are listed with both theme values.
- Motion and responsive changes are one-line notes.
- No reference library is asked to be installed or copied.

Sources: shadcn-svelte docs (MIT, Copyright (c) 2023 Hunter Johnston and CokaKoala); shadcn/ui skill SKILL.md and design-system.md (MIT, Copyright (c) 2023 shadcn).
