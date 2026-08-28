# MD3 Expressive Component Library

Date: 2026-08-29
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-08-28-admin-dashboard-and-agents-design.md` (the agents page this design converts to a `DataTable`), the M3 Expressive theming work already merged (`frontend/src/lib/styles/material3.css`).

## Purpose

The app currently has four reusable MD3 components (`Button`, `Card`, `TextField`, `Icon`) and a handful of interactive patterns — a dropdown menu, a labeled select, list rows, an icon-only button — hand-rolled inline, once per page, each with its own copy of near-identical CSS. This design builds the missing components as real, reusable `$lib/components/m3/*` files, matching MD3's actual behavioral spec (not just its visual appearance), and refactors every existing page that currently duplicates one of these patterns to consume the new component instead.

A concrete, current gap this closes: **none of the app's three existing hand-rolled dropdowns (chat page's model picker, chat page's personality panel, and — before this refactor — the same chat page's consolidated menu) close on outside click or Escape.** That's not how a Material menu behaves; the new `Menu` component fixes this for every consumer at once.

## 1. New Components

All live in `frontend/src/lib/components/m3/`, follow the existing components' conventions (Svelte 5 runes, `$props()`, spread `HTML*Attributes`, `class` passthrough, only `var(--md-sys-*)` tokens — never a raw hex/px value where a token exists).

### `IconButton.svelte`

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'standard' | 'filled' | 'filled-tonal' | 'outlined';

	let {
		variant = 'standard',
		href,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		href?: string;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes &
		HTMLAnchorAttributes = $props();
</script>
```

Renders `<a>` when `href` is set, `<button>` otherwise (same dual-mode pattern `Button.svelte` already uses). Fixed 48×48px — the MD3 minimum touch target — for every variant, simplifying the spec's separate "40dp visual / 48dp touch target" distinction into one size rather than reproducing the padding trick, since nothing in this app needs a visually-smaller icon button today. Four variants matching MD3's set: `standard` (transparent, `on-surface-variant`), `filled` (`primary`/`on-primary`), `filled-tonal` (`secondary-container`/`on-secondary-container`), `outlined` (transparent + `outline` border, `on-surface-variant`). `aria-label` is not enforced by a runtime check (Svelte can't easily assert a spread prop is present), but every call site in this design passes one — flagged in each task.

### `Icon.svelte` — two new glyphs

Add `'chevron-down'` and `'chevron-up'` to the existing hand-drawn set (`chevron-left`/`chevron-right` already exist) — needed by `Select`'s trigger affordance and `DataTable`'s sort indicator respectively. Same stroke-based convention as every existing icon.

### `Menu.svelte` + `MenuItem.svelte`

The core new interactive primitive — everything else that needs a dropdown (`Select`, and every page refactor below) is built on this.

```svelte
<!-- Menu.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		trigger,
		children,
		class: extraClass = '',
	}: {
		open?: boolean;
		trigger: Snippet<[{ toggle: () => void }]>;
		children: Snippet;
		class?: string;
	} = $props();

	let panelEl: HTMLDivElement | undefined = $state();
	let anchorEl: HTMLDivElement | undefined = $state();

	function toggle() {
		open = !open;
	}

	$effect(() => {
		if (!panelEl) return;
		if (open) {
			if (anchorEl) {
				const rect = anchorEl.getBoundingClientRect();
				panelEl.style.top = `${rect.bottom + 4}px`;
				panelEl.style.right = `${window.innerWidth - rect.right}px`;
			}
			if (!panelEl.matches(':popover-open')) panelEl.showPopover();
			const first = panelEl.querySelector<HTMLElement>('[role="menuitem"]');
			first?.focus();
		} else if (panelEl.matches(':popover-open')) {
			panelEl.hidePopover();
		}
	});

	function handleToggle(event: Event) {
		// Fires on every popover state change, including native light-dismiss (outside
		// click, Escape) — this is what keeps `open` in sync when the browser closes the
		// popover without Menu's own code asking it to.
		const toggleEvent = event as ToggleEvent;
		if (toggleEvent.newState === 'closed') open = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		const items = Array.from(panelEl?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []);
		if (items.length === 0) return;
		const currentIndex = items.indexOf(document.activeElement as HTMLElement);
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			items[(currentIndex + 1) % items.length]?.focus();
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			items[(currentIndex - 1 + items.length) % items.length]?.focus();
		} else if (event.key === 'Home') {
			event.preventDefault();
			items[0]?.focus();
		} else if (event.key === 'End') {
			event.preventDefault();
			items[items.length - 1]?.focus();
		}
	}
</script>

<div class="m3-menu-anchor {extraClass}" bind:this={anchorEl}>
	{@render trigger({ toggle })}
	<div bind:this={panelEl} popover="auto" role="menu" class="m3-menu-panel" ontoggle={handleToggle} onkeydown={handleKeydown}>
		{@render children()}
	</div>
</div>
```

Built on the native `popover="auto"` attribute (Baseline-available since 2024 — Chrome 114+, Firefox 125+, Safari 17+, all comfortably below this app's support bar) rather than a hand-rolled click-outside listener: `popover="auto"` gives correct light-dismiss (outside click, Escape) and top-layer rendering (no z-index or `overflow: hidden` clipping from an ancestor) for free, which is exactly the behavior this design set out to fix. Position is computed in JS against the trigger's `getBoundingClientRect()` on open (not CSS Anchor Positioning — that API's browser support is less uniform than `popover` itself, and this app's existing dropdowns already used the equivalent `position: absolute; right: 0; bottom: 100%` pattern, so a JS-computed `top`/`right` in viewport coordinates is a direct, compatible translation of that same idea for `position: fixed` top-layer content). Keyboard nav is a standard roving-tabindex menu: ArrowUp/Down moves focus between items, Home/End jump to the ends, Enter/Space (native `<button>` behavior, no extra code needed) activates the focused item.

```svelte
<!-- MenuItem.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		selected = false,
		children,
		class: extraClass = '',
		...rest
	}: {
		selected?: boolean;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes = $props();
</script>

<button type="button" role="menuitem" tabindex="-1" class="m3-menu-item {extraClass}" class:m3-menu-item--selected={selected} {...rest}>
	{@render children()}
</button>
```

`tabindex="-1"` on every item is deliberate — the roving-tabindex pattern means items are only reachable via `Menu`'s own ArrowUp/Down handling and the initial autofocus on open, never via normal Tab order (Tab should skip past a closed/background menu, matching real Material/ARIA menu behavior).

### `List.svelte` + `ListItem.svelte`

For rows that display a record with an optional side action (personality history, model list *before* they're picked, admin agents rows before the `DataTable` conversion below) — as opposed to `MenuItem`, which is for a single primary click-to-select-and-close action.

```svelte
<!-- List.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';
	let { children, class: extraClass = '' }: { children: Snippet; class?: string } = $props();
</script>

<div role="list" class="m3-list {extraClass}">
	{@render children()}
</div>
```

```svelte
<!-- ListItem.svelte -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		headline,
		supportingText,
		trailing,
		selected = false,
		class: extraClass = '',
	}: {
		headline: string;
		supportingText?: string;
		trailing?: Snippet;
		selected?: boolean;
		class?: string;
	} = $props();
</script>

<div role="listitem" class="m3-list-item {extraClass}" class:m3-list-item--selected={selected}>
	<div class="m3-list-item__text">
		<p class="md-body-large">{headline}</p>
		{#if supportingText}
			<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{supportingText}</p>
		{/if}
	</div>
	{#if trailing}
		<div class="m3-list-item__trailing">{@render trailing()}</div>
	{/if}
</div>
```

One-line vs. two-line is automatic: presence of `supportingText` adds the second line. `trailing` is a snippet slot (e.g. the personality panel's "Restore" `Button`), not a fixed prop shape — MD3's List spec allows arbitrary trailing content (icon, button, switch, text).

### `Select.svelte`

MD3's Select is **not** a styled native `<select>` — it's a custom trigger button plus an anchored menu list; no browser lets you restyle the native dropdown's own popup chrome to match. Built directly on `Menu` + `MenuItem`, so it gets the same correct keyboard/dismissal behavior for free, and a hidden `<input type="hidden">` so it still participates in a normal HTML form submission (every current consumer is a SvelteKit `use:enhance` form reading `FormData` server-side — this must keep working unchanged).

```svelte
<script lang="ts">
	import Menu from './Menu.svelte';
	import MenuItem from './MenuItem.svelte';
	import Icon from './Icon.svelte';

	let {
		label,
		name,
		value = $bindable(''),
		options,
		required = false,
		class: extraClass = '',
	}: {
		label: string;
		name: string;
		value?: string;
		options: { value: string; label: string }[];
		required?: boolean;
		class?: string;
	} = $props();

	let open = $state(false);
	const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? '');
</script>

<div class="m3-select {extraClass}">
	<span class="m3-select__label">{label}</span>
	<input type="hidden" {name} {value} {required} />
	<Menu bind:open>
		{#snippet trigger({ toggle })}
			<button type="button" class="m3-select__trigger" onclick={toggle} aria-haspopup="listbox" aria-expanded={open}>
				<span>{selectedLabel}</span>
				<Icon name={open ? 'chevron-up' : 'chevron-down'} size={18} />
			</button>
		{/snippet}
		{#each options as option (option.value)}
			<MenuItem selected={option.value === value} onclick={() => { value = option.value; open = false; }}>
				{option.label}
			</MenuItem>
		{/each}
	</Menu>
</div>
```

### `DataTable.svelte`

Proper `<table>` semantics — `<thead>`/`<tbody>`, `<th scope="col">`, `aria-sort` on sortable headers — with sorting kept deliberately dumb: `DataTable` only tracks *which* column and direction is active and calls back to the consumer; it never sorts the data itself. The consumer owns the array and re-derives a sorted view (`$derived`), keeping `DataTable` a pure rendering shell that works with any row shape.

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		columns,
		sortKey = $bindable<string | undefined>(undefined),
		sortDirection = $bindable<'asc' | 'desc'>('asc'),
		onSort,
		children,
		class: extraClass = '',
	}: {
		columns: { key: string; label: string; sortable?: boolean }[];
		sortKey?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		children: Snippet;
		class?: string;
	} = $props();

	function handleSort(column: { key: string; sortable?: boolean }) {
		if (!column.sortable) return;
		if (sortKey === column.key) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = column.key;
			sortDirection = 'asc';
		}
		onSort?.(column.key);
	}
</script>

<table class="m3-data-table {extraClass}">
	<thead>
		<tr>
			{#each columns as column (column.key)}
				<th
					scope="col"
					aria-sort={sortKey === column.key ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none'}
				>
					{#if column.sortable}
						<button type="button" class="m3-data-table__sort" onclick={() => handleSort(column)}>
							{column.label}
							{#if sortKey === column.key}
								<Icon name={sortDirection === 'asc' ? 'chevron-up' : 'chevron-down'} size={14} />
							{/if}
						</button>
					{:else}
						{column.label}
					{/if}
				</th>
			{/each}
		</tr>
	</thead>
	<tbody>
		{@render children()}
	</tbody>
</table>
```

Consumer renders `<tr>`/`<td>` rows into `children` — `DataTable` doesn't dictate per-cell rendering, since that already varies per consumer today (e.g. the agents page's formatted-date column).

### `Button.svelte` — Expressive size variants

Add a `size` prop (`'xs' | 's' | 'm' | 'l' | 'xl'`, default `'s'` — the current, only size) driving height and horizontal padding:

| Size | Height | Padding |
|---|---|---|
| `xs` | 32px | 0 16px |
| `s` (default, current) | 40px | 0 24px |
| `m` | 48px | 0 24px |
| `l` | 56px | 0 32px |
| `xl` | 64px | 0 36px |

No existing call site changes — every current `<Button>` usage is implicitly `size="s"`, unchanged.

## 2. Page Refactors (eliminate the duplication that motivated this design)

- **`frontend/src/routes/admin/(protected)/+layout.svelte`** — every `.m3-icon-button` (collapse toggle, all three nav links in collapsed state, logout) becomes `IconButton`. `.m3-icon-button` CSS rule deleted.
- **`frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`** — the hand-rolled `⋮` dropdown becomes one `Menu`: the root view's "Model"/"Personality" rows become `MenuItem`s (whole-row click, matches their current select-and-navigate behavior); the model list inside becomes `MenuItem`s (whole-row click selects and closes, matching current behavior exactly); the personality-history rows become `List`/`ListItem` with the "Restore" `Button` as `trailing` (not `MenuItem` — these rows aren't a single click-to-select action, they're a record with a secondary action button). `.m3-menu-row`/`.m3-picker-item`/`.m3-picker-item--selected`/`.m3-personality-item`/`.m3-personality-item--current`/`.m3-icon-back` CSS rules deleted (superseded by `Menu`/`MenuItem`/`List`/`ListItem`'s own styles). The "back" button inside the sub-views becomes an `IconButton`.
- **`frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`** — both provider `<select>` instances (edit form, create form) become `Select`. `.m3-select-field` CSS rules deleted.
- **`frontend/src/routes/admin/(protected)/agents/+page.svelte`** — converts from one `Card` per user (with agents listed as flex rows inside) to a single `DataTable`, columns `User | Agent Type | Channel | Started | Last Activity`, all sortable. **This is a deliberate scope decision, not an oversight**: the per-user `Card` grouping goes away in favor of a flat, sortable table with `User` as an ordinary (sortable) column — this is the correct, idiomatic way to use a `DataTable` (Material's spec has no "grouped rows" pattern), and a person's agents are still trivially identifiable by sorting on `User`. The empty state ("No agents are currently running") is preserved.

## 3. Testing

No new test framework — this codebase's frontend testing convention (established when markdown rendering was added) is `npm run check` plus `vitest` only where there's genuine non-visual logic worth protecting from regression, not component-library UI tests (no `@testing-library/svelte` dependency exists, and adding one is out of scope for this pass). The one piece of `Menu` that's pure, non-visual logic and worth a real test: the keyboard-navigation index math (`ArrowDown`/`ArrowUp`/`Home`/`End` wrapping correctly at the ends of the item list) — this can be tested directly against a rendered `Menu` via `@testing-library/svelte`... but since that dependency doesn't exist yet and adding a whole new testing layer is disproportionate to one component's keyboard math, this is called out explicitly as **manual verification only** (open each menu, confirm outside-click/Escape/arrow-key behavior in a real browser) rather than silently skipped.

Every other component and page refactor is verified via `npm run check` (type correctness) plus manual review of the diff (does the refactored page render the same information, does the new component's behavior match what's specified above) — consistent with how every other frontend change this session has been verified.

## Error Handling

| Condition | Behavior |
|---|---|
| `Menu` opened with zero `MenuItem`s inside | Panel still opens (empty popover); `handleKeydown`'s `items.length === 0` guard no-ops rather than throwing. |
| `Select` with `value` not matching any `options` entry | `selectedLabel` derives to `''` (trigger shows blank) rather than throwing — matches how the existing native `<select>` behaves with no matching `<option selected>`. |
| `DataTable` with zero rows | Not handled inside `DataTable` itself (it only renders whatever `children` produces) — each consumer keeps its own existing empty-state message above/around the table, same as today. |
| Browser without `popover` support | Out of scope — this app's support bar already assumes evergreen browsers (the whole app is client-rendered SvelteKit with no noscript fallback story). |

## Out of Scope

- `Dialog`, `Switch`, `Checkbox`, `Radio`, `Snackbar` — no current page needs them; adding them now would be speculative.
- `DataTable` pagination, filtering, or row selection — the current agents page has neither the data volume nor the requirement for any of these.
- Rewriting `Card` or `TextField` — audited against the MD3 spec during design; no gaps found worth closing.
- A shared design-system Storybook/gallery page — nice-to-have, not requested, not needed for this pass's actual consumers.
