# Components: shadcn-svelte (and shadcn/ui in React)

Build a SvelteKit project's UI from **shadcn-svelte** (shadcn-svelte.com), the Svelte port of
shadcn/ui. It isn't a package you import from: each component is a few source files that live in
the project (`src/lib/components/ui/<name>/`), built on `bits-ui` (accessible headless parts),
`tailwind-variants` and Tailwind v4, and styled only through CSS variables. They are your code:
change them to fit the design. React projects use **shadcn/ui** the same way (see the end).
Vue and Astro projects have no kit here: build components by hand from the design tokens.

## Don't run its CLI in Nomi: write the files

The shadcn-svelte docs set a project up with `npx shadcn-svelte@latest init` and add components
with `npx shadcn-svelte@latest add button dialog` (React: `npx shadcn@latest init`,
`npx shadcn@latest add button`). Don't run them with `run_command`. Commands run in the preview's
runtime (a WebContainer in the person's browser, or the sandbox), and files a command writes there
don't come back into the project: they aren't in `list_files`, aren't published and are gone on
the next restart. The CLI also expects SvelteKit 2 (`svelte.config.js`, `$lib` aliases) and
overwrites the global CSS file. So add the packages to `package.json` and write every file
yourself with `write_files`, as below.

## Set it up once per project

1. `devDependencies` (the versions shadcn-svelte's registry pins), then `npm install` once:
   ```json
   "bits-ui": "^2.14.4",
   "@internationalized/date": "^3.10.0",
   "tailwind-variants": "^3.2.2",
   "cn": "^0.3.0",
   "tw-animate-css": "^1.4.0",
   "@lucide/svelte": "^0.561.0",
   "mode-watcher": "^1.1.0"
   ```
   Add `"svelte-sonner": "^1.2.0"` with Sonner. A component's registry entry lists what it needs.

2. `src/lib/utils.ts` (the registry's `utils`, as it is):
   ```ts
   export { cn } from 'cn';

   // eslint-disable-next-line @typescript-eslint/no-explicit-any
   export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T;
   // eslint-disable-next-line @typescript-eslint/no-explicit-any
   export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T;
   export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
   export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
   ```

3. The theme in `src/routes/layout.css`. The components read these variables, so **they are the
   design tokens**: fill them from the approved design (or the direction you picked from the design
   guide), never with shadcn's default neutral greys. Mind the names: shadcn's `--primary` is the
   brand/accent colour (main button, links, selection); its `--accent` is only the quiet hover and
   highlight background.
   ```css
   @import 'tailwindcss';
   @import 'tw-animate-css';

   @custom-variant dark (&:is(.dark *));

   :root {
     --radius: 0.5rem;
     --background: oklch(98.5% 0.008 85);  /* the page */
     --foreground: oklch(22% 0.02 85);     /* body text */
     --card: oklch(99% 0.006 85);
     --card-foreground: oklch(22% 0.02 85);
     --popover: oklch(99% 0.006 85);
     --popover-foreground: oklch(22% 0.02 85);
     --primary: oklch(55% 0.15 35);        /* the brand colour */
     --primary-foreground: oklch(98% 0.01 35);
     --secondary: oklch(95% 0.01 85);
     --secondary-foreground: oklch(22% 0.02 85);
     --muted: oklch(96% 0.01 85);
     --muted-foreground: oklch(52% 0.02 85);
     --accent: oklch(95% 0.012 85);        /* hover and highlight background */
     --accent-foreground: oklch(22% 0.02 85);
     --destructive: oklch(55% 0.19 27);
     --border: oklch(89% 0.012 85);
     --input: oklch(89% 0.012 85);
     --ring: oklch(55% 0.15 35);
   }

   .dark {
     /* the same names with the dark values */
   }

   @theme {
     --font-display: 'Fraunces', ui-serif, Georgia, serif;
     --font-sans: 'Inter Tight', ui-sans-serif, system-ui, sans-serif;
   }

   @theme inline {
     --color-background: var(--background);
     --color-foreground: var(--foreground);
     --color-card: var(--card);
     --color-card-foreground: var(--card-foreground);
     --color-popover: var(--popover);
     --color-popover-foreground: var(--popover-foreground);
     --color-primary: var(--primary);
     --color-primary-foreground: var(--primary-foreground);
     --color-secondary: var(--secondary);
     --color-secondary-foreground: var(--secondary-foreground);
     --color-muted: var(--muted);
     --color-muted-foreground: var(--muted-foreground);
     --color-accent: var(--accent);
     --color-accent-foreground: var(--accent-foreground);
     --color-destructive: var(--destructive);
     --color-border: var(--border);
     --color-input: var(--input);
     --color-ring: var(--ring);
     --radius-sm: calc(var(--radius) * 0.6);
     --radius-md: calc(var(--radius) * 0.8);
     --radius-lg: var(--radius);
     --radius-xl: calc(var(--radius) * 1.4);
   }

   @layer base {
     * { @apply border-border outline-ring/50; }
     body { @apply bg-background text-foreground font-sans antialiased; }
   }
   ```
   Add `--chart-1`…`--chart-5` with Chart and the `--sidebar-*` set with Sidebar (each mapped in
   `@theme inline` the same way). Any other token the design has (a success colour, a second
   surface) is named and mapped the same way.

4. `src/routes/+layout.svelte`: `<ModeWatcher />` puts the `.dark` class on when the system is dark
   (the components' `dark:` styles need it), and `<Toaster />` once with Sonner:
   ```svelte
   <script lang="ts">
   	import './layout.css';
   	import { ModeWatcher } from 'mode-watcher';
   	import { Toaster } from '#lib/components/ui/sonner/index.ts';
   	import type { LayoutProps } from './$types';

   	let { children }: LayoutProps = $props();
   </script>

   <ModeWatcher />
   <Toaster />
   {@render children()}
   ```

You don't need `components.json`: only the CLI reads it.

## Add a component

1. Get its files: `https://shadcn-svelte.com/registry/<name>.json` (`button.json`, `dialog.json`;
   `index.json` lists them all). Each entry has `files` (a `target` such as
   `dialog/dialog-content.svelte` and its `content`), `devDependencies`, and
   `registryDependencies`: other components it uses (Alert Dialog uses Button), added first. Read
   it with `read_web_page` if you have it. Otherwise, while the project is open, write this
   script, run `node scripts/registry.mjs dialog` (the list of files) and
   `node scripts/registry.mjs dialog dialog/dialog-content.svelte` (one file), and delete the
   script once the components are in:
   ```js
   const [name, target] = process.argv.slice(2);
   const item = await (await fetch(`https://shadcn-svelte.com/registry/${name}.json`)).json();
   if (target) console.log(item.files.find((f) => f.target === target).content);
   else console.log(JSON.stringify({ files: item.files.map((f) => f.target), devDependencies: item.devDependencies, registryDependencies: item.registryDependencies }));
   ```
   When neither works, write the files from what you know of shadcn-svelte in the same shape (a
   folder per component, an `index.ts` that exports its parts, `bits-ui` underneath,
   `tailwind-variants` for variants, `cn` for classes) and say so in your report.
2. Write each file to `src/lib/components/ui/<target>` with `write_files`, changing the registry's
   placeholders to SvelteKit 3 imports: `$UTILS$.js` → `#lib/utils.ts`, `$UI$/button/index.js` →
   `#lib/components/ui/button/index.ts`, `$HOOKS$/is-mobile.svelte.js` →
   `#lib/hooks/is-mobile.svelte.ts`. Relative imports (`./button.svelte`) stay as they are.
3. Add its `devDependencies` to `package.json`; run `npm install` once after adding them all.
4. Use it:
   ```svelte
   <script lang="ts">
   	import { Button, buttonVariants } from '#lib/components/ui/button/index.ts';
   	import * as Dialog from '#lib/components/ui/dialog/index.ts';
   	import { toast } from 'svelte-sonner';
   </script>

   <Dialog.Root>
   	<Dialog.Trigger class={buttonVariants({ variant: 'outline' })}>Edit</Dialog.Trigger>
   	<Dialog.Content>
   		<Dialog.Header><Dialog.Title>Edit book</Dialog.Title></Dialog.Header>
   		<Dialog.Footer><Button onclick={() => toast.success('Saved')}>Save</Button></Dialog.Footer>
   	</Dialog.Content>
   </Dialog.Root>
   ```
   Parts are `Name.Part` from a namespace import (or `CardHeader`, `DialogTitle` named exports).
   Props a component doesn't use go to its element, so `data-design-id` and `aria-*` pass through.

The registry's components: accordion, alert, alert-dialog, aspect-ratio, avatar, badge,
breadcrumb, button, button-group, calendar, card, carousel, chart, checkbox, collapsible, command,
context-menu, data-table, dialog, drawer, dropdown-menu, empty, field, form, hover-card, input,
input-group, input-otp, item, kbd, label, menubar, native-select, navigation-menu, pagination,
popover, progress, radio-group, range-calendar, resizable, scroll-area, select, separator, sheet,
sidebar, skeleton, slider, sonner, spinner, switch, table, tabs, textarea, toggle, toggle-group,
tooltip. A combobox is Popover + Command; a date picker is Popover + Calendar.

## From Sora's handoff to components

The handoff's Components list each part as `<data-design-id> (<name>): <what it is>. States: …`,
and Sora names them `<Area> / <Thing> / <State>` with shadcn's names: `Checkout / Button / Primary`,
`Settings / Dialog / Delete account`, `Nav / Sheet / Open`.

- **Thing** is the shadcn-svelte component (table below).
- **State** is a prop or a state of it: Default/Primary → `variant="default"`; Secondary →
  `variant="secondary"`; Outline → `variant="outline"`; Ghost → `variant="ghost"`;
  Destructive/Danger → `variant="destructive"`; Link → `variant="link"`; Small/Large/Icon →
  `size="sm" | "lg" | "icon"`; Disabled → `disabled`; Loading → `disabled` with a Spinner;
  Invalid/Error → `aria-invalid="true"` with the message under the field; Open → `bind:open`;
  Selected/Active → the component's value (`Tabs` `value`, `aria-current="page"` in a nav).
- **Area** is where it lives: the route, or a composed component in `src/lib/components/<area>/`
  built from the `ui/` parts.
- Keep each `data-design-id` on the element you build for it.

| Sora says | Build with |
|---|---|
| Button, icon button | `button` (`variant`, `size`) |
| Card | `card` (`Card.Root`, `Header`, `Title`, `Description`, `Action`, `Content`, `Footer`) |
| Dialog, modal / confirm | `dialog` / `alert-dialog` |
| Sheet, side panel / bottom sheet | `sheet` (`side="left"`…) / `drawer` |
| Input, text field, textarea | `input`, `textarea`, with `label` and `field` |
| Select, dropdown (a form value) | `select` (`native-select` when it must be the native one) |
| Menu, overflow (…) menu | `dropdown-menu` |
| Tabs, segmented control | `tabs` / `toggle-group` |
| Toast, Sonner, notification | `sonner`: `toast()`, `toast.success()`, `toast.error()` |
| Table, data grid | `table` (`data-table` for sorting and paging) |
| Badge, tag, chip | `badge` (`default`, `secondary`, `outline`, `destructive`) |
| Avatar | `avatar` (`Avatar.Root`, `Image`, `Fallback`) |
| Switch, checkbox, radio | `switch`, `checkbox`, `radio-group` |
| Tooltip, popover | `tooltip`, `popover` |
| Empty state, skeleton, spinner, progress | `empty`, `skeleton`, `spinner`, `progress` |
| App sidebar, breadcrumb, pagination | `sidebar`, `breadcrumb`, `pagination` |
| Search / command palette | `command` |

A part the kit doesn't have is built the same way in `src/lib/components/`: `tv()` for its
variants, `cn()` for classes, the theme's colour utilities only.

## The look is the design's, not shadcn's

The registry's default look (neutral greys, `rounded-md`, `shadow-xs`) is the stock shadcn look.
Set the variables from the design first; then, where Sora's design differs from a component
(radius, border, shadow, height, spacing), change that component's classes or add the variant the
design names. The approved design wins over the registry's defaults.

## Not these

- **rare-ui**: Sora may learn from it; you never copy its code. Its licence is MIT plus the
  Commons Clause plus a required visible attribution, which a person's app can't carry.
- **beUI**: reference only; never use beUI Pro or copy its code.
- No other component kit (DaisyUI, Flowbite, Skeleton, Material…). `bits-ui` is fine: shadcn-svelte
  is built on it.

## React projects: shadcn/ui

The same rules. Files come from `https://ui.shadcn.com/r/styles/new-york-v4/<name>.json` (the
same `files` and `content`), go in `src/components/ui/<name>.tsx`, and their imports become
relative with the extension (`@/components/ui/button` → `./button.tsx`; `cn` comes from the
`cn` package). Packages: `radix-ui`, `class-variance-authority`, `cn`, `lucide-react`,
`tw-animate-css`. The same variables and `@theme inline` go in `src/index.css`. Use it as
`import { Button } from './components/ui/button.tsx'` with `<Button variant="outline">`.
