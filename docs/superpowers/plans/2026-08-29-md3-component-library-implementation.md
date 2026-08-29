# MD3 Expressive Component Library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the missing MD3 Expressive components (`IconButton`, `Menu`/`MenuItem`, `List`/`ListItem`, `Select`, `DataTable`) plus Expressive size variants for the existing `Button`, and refactor every page that currently hand-rolls one of these patterns to consume the new component instead.

**Architecture:** All new components live in `frontend/src/lib/components/m3/`, follow the existing components' conventions (Svelte 5 runes, `$props()`, spread `HTML*Attributes`, only `var(--md-sys-*)` tokens). `Menu` is the load-bearing primitive — built on the native `popover="auto"` attribute for correct light-dismiss (outside click, Escape) and top-layer rendering, which fixes a real behavioral gap every existing hand-rolled dropdown in this app has today. `Select` and every page's dropdown/menu usage are built on top of `Menu`.

**Tech Stack:** SvelteKit 2, Svelte 5 (runes), TypeScript, Tailwind (layout utilities only — all design tokens are CSS custom properties).

**Spec:** `docs/superpowers/specs/2026-08-29-md3-component-library-design.md`

## Global Constraints

- Every new component uses only `var(--md-sys-*)` tokens already defined in `frontend/src/lib/styles/material3.css` — no hardcoded hex/px values where a token exists (spec's own stated MD3 anti-pattern).
- `Menu` behavioral requirements are non-negotiable, not nice-to-haves (per direct instruction: "make sure the component behaviour works exactly like how material component should work"): opens on trigger click; closes on item selection, outside click, and Escape; ArrowUp/ArrowDown/Home/End roving-tabindex keyboard navigation between items; focus moves to the first item on open.
- `DataTable` never sorts data itself — it only tracks `sortKey`/`sortDirection` and calls `onSort`; every consumer owns sorting its own array via `$derived`.
- No new test framework — verification is `npm run check` (type correctness) plus manual behavioral verification for `Menu`'s keyboard/dismissal behavior specifically (called out per-task below), consistent with this codebase's established frontend convention.
- Page refactors must not change any existing server-side behavior (form action names, field names, `use:enhance` callback logic) — only the markup/component used to render the same interaction.

---

### Task 1: `Icon` additions, `IconButton`, and `Button` Expressive sizes

**Files:**
- Modify: `frontend/src/lib/components/m3/Icon.svelte`
- Create: `frontend/src/lib/components/m3/IconButton.svelte`
- Modify: `frontend/src/lib/components/m3/Button.svelte`

**Interfaces:**
- Produces (used by Tasks 6, 8): `IconButton` with props `{ variant?: 'standard'|'filled'|'filled-tonal'|'outlined'; href?: string; children: Snippet } & HTMLButtonAttributes & HTMLAnchorAttributes`.
- Produces (used by Task 5, and Icon's own consumers): `Icon` gains `'chevron-down'` and `'chevron-up'` to its `IconName` union.
- Produces: `Button` gains `size?: 'xs'|'s'|'m'|'l'|'xl'` (default `'s'`, matching current behavior exactly).

- [ ] **Step 1: Add the two new icon glyphs**

Edit `frontend/src/lib/components/m3/Icon.svelte` — add to the `IconName` union:

```ts
	export type IconName =
		| 'dashboard'
		| 'settings'
		| 'logout'
		| 'plus'
		| 'chat-bubble'
		| 'chevron-left'
		| 'chevron-right'
		| 'agents'
		| 'more'
		| 'chevron-down'
		| 'chevron-up';
```

Add two new branches to the SVG's `{#if}` chain, right before the closing `{/if}`:

```svelte
	{:else if name === 'chevron-down'}
		<polyline points="6 9 12 15 18 9" />
	{:else if name === 'chevron-up'}
		<polyline points="6 15 12 9 18 15" />
	{/if}
```

- [ ] **Step 2: Create `IconButton`**

Create `frontend/src/lib/components/m3/IconButton.svelte`:

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

{#if href}
	<a {href} class="m3-icon-btn m3-icon-btn--{variant} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button type="button" class="m3-icon-btn m3-icon-btn--{variant} {extraClass}" {...rest}>
		{@render children()}
	</button>
{/if}

<style>
	.m3-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		cursor: pointer;
		text-decoration: none;
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-icon-btn:disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-icon-btn--standard {
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-icon-btn--standard:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}

	.m3-icon-btn--filled {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-icon-btn--filled:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-icon-btn--filled-tonal {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.m3-icon-btn--filled-tonal:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-icon-btn--outlined {
		background: transparent;
		border: 1px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-icon-btn--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>
```

- [ ] **Step 3: Add `Button` size variants**

Replace the full contents of `frontend/src/lib/components/m3/Button.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'filled' | 'tonal' | 'outlined' | 'text' | 'elevated';
	type Size = 'xs' | 's' | 'm' | 'l' | 'xl';

	let {
		variant = 'filled',
		size = 's',
		href,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		size?: Size;
		href?: string;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes &
		HTMLAnchorAttributes = $props();
</script>

{#if href}
	<a {href} class="m3-button m3-button--{variant} m3-button--size-{size} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button class="m3-button m3-button--{variant} m3-button--size-{size} {extraClass}" {...rest}>
		{@render children()}
	</button>
{/if}

<style>
	.m3-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		cursor: pointer;
		white-space: nowrap;
		text-decoration: none;
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		line-height: var(--md-sys-typescale-label-large-line-height);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		transition:
			background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
			box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	/* Declared before the variant rules below so .m3-button--text's own padding override
	   (same specificity, later in source order) correctly wins for text buttons at every
	   size — text buttons keep tighter horizontal padding regardless of size, matching MD3. */
	.m3-button--size-xs {
		height: 32px;
		padding: 0 16px;
	}
	.m3-button--size-s {
		height: 40px;
		padding: 0 24px;
	}
	.m3-button--size-m {
		height: 48px;
		padding: 0 24px;
	}
	.m3-button--size-l {
		height: 56px;
		padding: 0 32px;
	}
	.m3-button--size-xl {
		height: 64px;
		padding: 0 36px;
	}

	.m3-button:disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-button--filled {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-button--filled:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--tonal {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.m3-button--tonal:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--elevated {
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-primary);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--outlined {
		background: transparent;
		color: var(--md-sys-color-primary);
		border: 1px solid var(--md-sys-color-outline);
	}
	.m3-button--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}

	.m3-button--text {
		background: transparent;
		color: var(--md-sys-color-primary);
		padding: 0 12px;
	}
	.m3-button--text:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
</style>
```

- [ ] **Step 4: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors. No existing `<Button>` call site needs updating — every one implicitly gets `size="s"` (40px), identical to today's fixed height.

- [ ] **Step 5: Self-review**

Confirm: every existing page still renders buttons at the same 40px height as before (grep for any `<Button` usage and confirm none accidentally changed size); the two new `Icon` glyphs render (no missing-glyph gap) at both the 20px default and other sizes used elsewhere.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/lib/components/m3/Icon.svelte frontend/src/lib/components/m3/IconButton.svelte \
        frontend/src/lib/components/m3/Button.svelte
git commit -m "feat: add IconButton, Button size variants, and two new icon glyphs"
```

---

### Task 2: `Menu` and `MenuItem`

**Files:**
- Create: `frontend/src/lib/components/m3/Menu.svelte`
- Create: `frontend/src/lib/components/m3/MenuItem.svelte`

**Interfaces:**
- Produces (used by Task 5 and Task 8): `Menu` with props `{ open?: boolean (bindable); trigger: Snippet<[{ toggle: () => void }]>; children: Snippet; class?: string }`.
- Produces (used by Task 5 and Task 8): `MenuItem` with props `{ selected?: boolean; children: Snippet; class?: string } & HTMLButtonAttributes`.

- [ ] **Step 1: Create `Menu`**

Create `frontend/src/lib/components/m3/Menu.svelte`:

```svelte
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
	<div
		bind:this={panelEl}
		popover="auto"
		role="menu"
		class="m3-menu-panel"
		ontoggle={handleToggle}
		onkeydown={handleKeydown}
	>
		{@render children()}
	</div>
</div>

<style>
	.m3-menu-anchor {
		position: relative;
		display: inline-block;
	}

	.m3-menu-panel {
		inset: auto;
		margin: 0;
		padding: 8px;
		min-width: 200px;
		max-width: 320px;
		max-height: 60vh;
		overflow-y: auto;
		border: none;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
</style>
```

Note: `[popover]`'s user-agent stylesheet already sets `display: none` when the popover isn't in the "showing" state — no extra CSS needed for that.

- [ ] **Step 2: Create `MenuItem`**

Create `frontend/src/lib/components/m3/MenuItem.svelte`:

```svelte
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

<button
	type="button"
	role="menuitem"
	tabindex="-1"
	class="m3-menu-item {extraClass}"
	class:m3-menu-item--selected={selected}
	{...rest}
>
	{@render children()}
</button>

<style>
	.m3-menu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		border: none;
		background: transparent;
		cursor: pointer;
		text-align: left;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		color: var(--md-sys-color-on-surface);
	}
	.m3-menu-item:hover,
	.m3-menu-item:focus-visible {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		outline: none;
	}
	.m3-menu-item--selected {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
</style>
```

`type="button"` is written before `{...rest}`, so a consumer can override it (e.g. `type="submit"` for a `MenuItem` inside a `use:enhance` form, exactly as Task 8 needs) — Svelte applies spread attributes in source order, so a `type` in `rest` wins over the literal one before it.

- [ ] **Step 3: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors (nothing consumes `Menu`/`MenuItem` yet, so this only checks the two new files compile).

- [ ] **Step 4: Manual behavioral verification** (no automated test — see Global Constraints)

Temporarily mount `Menu` with a couple of `MenuItem`s in any page (or use the dev server on an existing page after Task 5/8 wire it in — if verifying in isolation now, add a throwaway usage, check it, then remove it) and confirm in a real browser:
- Clicking the trigger opens the panel; clicking a `MenuItem` fires its `onclick`.
- Clicking anywhere outside the panel closes it.
- Pressing Escape while the panel is open closes it.
- Pressing ArrowDown/ArrowUp moves focus between items, wrapping at both ends; Home/End jump to the first/last item.
- Opening the menu moves focus to the first item automatically.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/components/m3/Menu.svelte frontend/src/lib/components/m3/MenuItem.svelte
git commit -m "feat: add the Menu component (popover-based) and MenuItem"
```

---

### Task 3: `List` and `ListItem`

**Files:**
- Create: `frontend/src/lib/components/m3/List.svelte`
- Create: `frontend/src/lib/components/m3/ListItem.svelte`

**Interfaces:**
- Produces (used by Task 8): `List` with props `{ children: Snippet; class?: string }`.
- Produces (used by Task 8): `ListItem` with props `{ headline: string; supportingText?: string; trailing?: Snippet; selected?: boolean; class?: string }`.

- [ ] **Step 1: Create `List`**

Create `frontend/src/lib/components/m3/List.svelte`:

```svelte
<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, class: extraClass = '' }: { children: Snippet; class?: string } = $props();
</script>

<div role="list" class="m3-list {extraClass}">
	{@render children()}
</div>

<style>
	.m3-list {
		display: flex;
		flex-direction: column;
	}
</style>
```

- [ ] **Step 2: Create `ListItem`**

Create `frontend/src/lib/components/m3/ListItem.svelte`:

```svelte
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
		<p class="md-body-large" style="margin: 0; color: var(--md-sys-color-on-surface)">{headline}</p>
		{#if supportingText}
			<p class="md-body-small" style="margin: 0; color: var(--md-sys-color-on-surface-variant)">{supportingText}</p>
		{/if}
	</div>
	{#if trailing}
		<div class="m3-list-item__trailing">{@render trailing()}</div>
	{/if}
</div>

<style>
	.m3-list-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-small);
	}
	.m3-list-item--selected {
		background: var(--md-sys-color-secondary-container);
	}
	.m3-list-item__text {
		min-width: 0;
		flex: 1;
	}
	.m3-list-item__text p {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.m3-list-item__trailing {
		flex-shrink: 0;
	}
</style>
```

- [ ] **Step 3: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/m3/List.svelte frontend/src/lib/components/m3/ListItem.svelte
git commit -m "feat: add the List and ListItem components"
```

---

### Task 4: `DataTable`

**Files:**
- Create: `frontend/src/lib/components/m3/DataTable.svelte`

**Interfaces:**
- Produces (used by Task 7): `DataTable` with props `{ columns: { key: string; label: string; sortable?: boolean }[]; sortKey?: string (bindable); sortDirection?: 'asc'|'desc' (bindable); onSort?: (key: string) => void; children: Snippet; class?: string }`. `children` renders the `<tr>` rows (into `<tbody>`) — `DataTable` never sorts data itself, only tracks which column/direction is active.

- [ ] **Step 1: Create `DataTable`**

Create `frontend/src/lib/components/m3/DataTable.svelte`:

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

<div class="m3-data-table-wrap {extraClass}">
	<table class="m3-data-table">
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
</div>

<style>
	.m3-data-table-wrap {
		overflow-x: auto;
	}
	.m3-data-table {
		width: 100%;
		border-collapse: collapse;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-data-table thead th {
		text-align: left;
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		white-space: nowrap;
	}
	.m3-data-table__sort {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		border: none;
		background: transparent;
		cursor: pointer;
		padding: 0;
		color: inherit;
		font: inherit;
	}
	/* tbody rows/cells come from the consumer's `children` snippet, not this component's own
	   template — Svelte's scoped-style hash only applies to markup this file renders directly,
	   so styling externally-provided content needs :global(). */
	.m3-data-table :global(tbody td) {
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
	}
	.m3-data-table :global(tbody tr:hover) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
	}
</style>
```

- [ ] **Step 2: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/m3/DataTable.svelte
git commit -m "feat: add the DataTable component"
```

---

### Task 5: `Select`

**Files:**
- Create: `frontend/src/lib/components/m3/Select.svelte`

**Interfaces:**
- Consumes: `Menu`, `MenuItem` (Task 2), `Icon`'s `chevron-down`/`chevron-up` (Task 1).
- Produces (used by Task 6, Task 8): `Select` with props `{ label: string; name: string; options: { value: string; label: string }[]; value?: string (bindable, defaults to options[0]?.value); class?: string }`. Renders a hidden `<input type={"hidden"}>` so it participates in a normal HTML form submission exactly like the native `<select>` it replaces.

- [ ] **Step 1: Create `Select`**

Create `frontend/src/lib/components/m3/Select.svelte`:

```svelte
<script lang="ts">
	import Menu from './Menu.svelte';
	import MenuItem from './MenuItem.svelte';
	import Icon from './Icon.svelte';

	let {
		label,
		name,
		options,
		value = $bindable(options[0]?.value ?? ''),
		class: extraClass = '',
	}: {
		label: string;
		name: string;
		options: { value: string; label: string }[];
		value?: string;
		class?: string;
	} = $props();

	// `options` must be destructured before `value` above so value's default expression can
	// reference it — matches native <select>'s own behavior of defaulting to the first
	// <option> when nothing is explicitly selected.

	let open = $state(false);
	const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? '');
</script>

<div class="m3-select {extraClass}">
	<span class="m3-select__label">{label}</span>
	<input type="hidden" {name} {value} />
	<Menu bind:open>
		{#snippet trigger({ toggle })}
			<button type="button" class="m3-select__trigger" onclick={toggle} aria-haspopup="listbox" aria-expanded={open}>
				<span>{selectedLabel}</span>
				<Icon name={open ? 'chevron-up' : 'chevron-down'} size={18} />
			</button>
		{/snippet}
		{#each options as option (option.value)}
			<MenuItem
				selected={option.value === value}
				onclick={() => {
					value = option.value;
					open = false;
				}}
			>
				{option.label}
			</MenuItem>
		{/each}
	</Menu>
</div>

<style>
	.m3-select {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.m3-select__label {
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
		letter-spacing: var(--md-sys-typescale-body-small-tracking);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-select__trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		box-sizing: border-box;
		height: 44px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		cursor: pointer;
	}
	.m3-select__trigger:focus-visible {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 15px;
	}
</style>
```

`required` is deliberately not a prop: a `type="hidden"` input is excluded from HTML constraint validation entirely (the `required` attribute has no effect on it), and defaulting `value` to the first option (matching native `<select>`) already guarantees a value is always present — an inert `required` prop would be dead API surface.

- [ ] **Step 2: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/m3/Select.svelte
git commit -m "feat: add the Select component (trigger + Menu, not a native select)"
```

---

### Task 6: Refactor — admin layout (`IconButton`) and LLM settings (`Select`)

**Files:**
- Modify: `frontend/src/routes/admin/(protected)/+layout.svelte`
- Modify: `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`

**Interfaces:**
- Consumes: `IconButton` (Task 1), `Select` (Task 5).

- [ ] **Step 1: Admin layout — replace every `.m3-icon-button` with `IconButton`**

Replace the full contents of `frontend/src/routes/admin/(protected)/+layout.svelte`:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import type { LayoutData } from './$types';

	let { children }: { data: LayoutData; children: Snippet } = $props();

	const STORAGE_KEY = 'nomi:admin-sidebar-collapsed';
	let collapsed = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
	}
</script>

<div class="flex h-screen" style="background: var(--md-sys-color-surface)">
	<aside
		class="flex flex-col p-4 transition-[width] duration-200"
		class:w-56={!collapsed}
		class:w-20={collapsed}
		class:items-center={collapsed}
		style="background: var(--md-sys-color-surface-container); border-right: 1px solid var(--md-sys-color-outline-variant)"
	>
		<div class="mb-4 flex w-full items-center" class:justify-center={collapsed} class:justify-between={!collapsed}>
			{#if !collapsed}
				<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface)">Admin</h2>
			{/if}
			<IconButton onclick={toggleCollapsed} aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
				<Icon name={collapsed ? 'chevron-right' : 'chevron-left'} />
			</IconButton>
		</div>

		<nav class="flex flex-col gap-1" class:items-center={collapsed}>
			{#if collapsed}
				<IconButton href="/admin" aria-label="Dashboard">
					<Icon name="dashboard" />
				</IconButton>
				<IconButton href="/admin/settings/llm" aria-label="LLM Settings">
					<Icon name="settings" />
				</IconButton>
				<IconButton href="/admin/agents" aria-label="Agents">
					<Icon name="agents" />
				</IconButton>
			{:else}
				<a href="/admin" class="m3-nav-link">Dashboard</a>
				<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
				<a href="/admin/agents" class="m3-nav-link">Agents</a>
			{/if}
		</nav>

		<form method="POST" action="/logout?redirect_to=/admin/login" class="mt-auto">
			{#if collapsed}
				<IconButton type="submit" style="color: var(--md-sys-color-outline)" aria-label="Log out">
					<Icon name="logout" />
				</IconButton>
			{:else}
				<button type="submit" class="m3-nav-link m3-nav-link--muted w-full text-left">Log out</button>
			{/if}
		</form>
	</aside>
	<main class="flex-1 overflow-y-auto p-8">
		{@render children()}
	</main>
</div>

<style>
	.m3-nav-link {
		display: block;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		border: none;
		background: transparent;
		cursor: pointer;
	}
	.m3-nav-link:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-nav-link--muted {
		color: var(--md-sys-color-outline);
	}
</style>
```

Note the `.m3-icon-button` CSS rule is gone entirely — superseded by `IconButton`'s own styles. `.m3-nav-link`/`.m3-nav-link--muted` are unchanged (unrelated to this refactor — the expanded-sidebar nav still uses plain text links, not icon buttons).

- [ ] **Step 2: LLM settings — replace both provider `<select>` instances with `Select`**

Replace the full contents of `frontend/src/routes/admin/(protected)/settings/llm/+page.svelte`:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	let showCreateForm = $state(false);
	let editingId = $state<string | null>(null);

	const PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">LLM models</h1>

{#if form?.error}
	<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
{/if}

<div class="mt-6 space-y-3">
	{#each data.models as model (model.id)}
		<Card variant="outlined" class="p-4">
			{#if editingId === model.id}
				<form
					method="POST"
					action="?/update"
					use:enhance={() => {
						return async ({ update }) => {
							await update();
							editingId = null;
						};
					}}
					class="space-y-2"
				>
					<input type="hidden" name="id" value={model.id} />
					<TextField id="label" name="label" label="Label" value={model.label} required />
					<Select label="Provider" name="provider" options={PROVIDER_OPTIONS} value={model.provider} />
					<TextField id="model_id" name="model_id" label="Model ID" value={model.model_id} />
					<TextField id="base_url" name="base_url" label="Base URL (optional)" value={model.base_url ?? ''} />
					<TextField
						id="api_key"
						name="api_key"
						type="password"
						label="API key"
						placeholder={`Leave blank to keep ${model.api_key_masked}`}
					/>
					<div class="flex gap-2 pt-2">
						<Button type="submit" variant="filled">Save</Button>
						<Button type="button" variant="outlined" onclick={() => (editingId = null)}>Cancel</Button>
					</div>
				</form>
			{:else}
				<div class="flex items-center justify-between">
					<div>
						<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">
							{model.label}
							{#if model.is_default}
								<span
									class="md-label-medium ml-2 rounded-full px-2 py-0.5"
									style="background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary)"
								>
									Default
								</span>
							{/if}
						</p>
						<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
							{model.provider} · {model.model_id} · {model.api_key_masked}
						</p>
					</div>
					<div class="flex items-center gap-2">
						<Button type="button" variant="text" onclick={() => (editingId = model.id)}>Edit</Button>
						{#if !model.is_default}
							<form method="POST" action="?/setDefault" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<Button type="submit" variant="text">Set default</Button>
							</form>
							<form method="POST" action="?/delete" use:enhance>
								<input type="hidden" name="id" value={model.id} />
								<Button type="submit" variant="text" style="color: var(--md-sys-color-error)">Delete</Button>
							</form>
						{/if}
					</div>
				</div>
			{/if}
		</Card>
	{/each}
</div>

<div class="mt-6">
	{#if showCreateForm}
		<form
			method="POST"
			action="?/create"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
					showCreateForm = false;
				};
			}}
			class="max-w-lg space-y-4 p-6"
			style="background: var(--md-sys-color-surface-container-low); border-radius: var(--md-sys-shape-corner-large)"
		>
			<TextField id="label" name="label" label="Label" required />
			<Select label="Provider" name="provider" options={PROVIDER_OPTIONS} />
			<TextField id="model_id" name="model_id" label="Model ID" />
			<TextField id="base_url" name="base_url" label="Base URL (optional)" />
			<TextField id="api_key" name="api_key" type="password" label="API key" />
			<Button type="submit" variant="filled" class="w-full">Add model</Button>
		</form>
	{:else}
		<Button type="button" variant="outlined" onclick={() => (showCreateForm = true)}>+ Add model</Button>
	{/if}
</div>
```

Note the `<style>` block with `.m3-select-field`/`.m3-select-field__label`/`.m3-select-field__select` is gone entirely — superseded by `Select`'s own styles. The create form's `Select` has no `value` prop, so it defaults to `PROVIDER_OPTIONS[0].value` (`'anthropic'`) — exactly matching the old native `<select>`'s behavior (no `<option selected>` anywhere meant the browser defaulted to the first option).

- [ ] **Step 3: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 4: Self-review**

Confirm: the sidebar's collapsed-state icons, expand/collapse toggle, and logout button all still work and are still 48×48px touch targets (previously 40×40px on `.m3-icon-button` — this is an intentional, spec-driven size increase to meet MD3's minimum touch target, not a regression); the LLM settings page's edit form still submits `provider` with the model's current value as the initial selection; the create form still defaults to Anthropic.

- [ ] **Step 5: Commit**

```bash
git add "frontend/src/routes/admin/(protected)/+layout.svelte" \
        "frontend/src/routes/admin/(protected)/settings/llm/+page.svelte"
git commit -m "refactor: use IconButton in the admin nav and Select in LLM settings"
```

---

### Task 7: Refactor — admin agents page (`DataTable`)

**Files:**
- Modify: `frontend/src/routes/admin/(protected)/agents/+page.svelte`

**Interfaces:**
- Consumes: `DataTable` (Task 4).

- [ ] **Step 1: Convert the grouped-Card layout to a flat, sortable `DataTable`**

Replace the full contents of `frontend/src/routes/admin/(protected)/agents/+page.svelte`:

```svelte
<script lang="ts">
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	type Row = {
		agent_session_id: string;
		user_label: string;
		agent_type: string;
		channel: string;
		started_at: string;
		last_activity_at: string;
	};

	const rows: Row[] = $derived(
		data.agents.users.flatMap((group) =>
			group.agents.map((agent) => ({
				agent_session_id: agent.agent_session_id,
				user_label: group.label,
				agent_type: agent.agent_type,
				channel: agent.channel,
				started_at: agent.started_at,
				last_activity_at: agent.last_activity_at,
			})),
		),
	);

	const columns = [
		{ key: 'user_label', label: 'User', sortable: true },
		{ key: 'agent_type', label: 'Agent Type', sortable: true },
		{ key: 'channel', label: 'Channel', sortable: true },
		{ key: 'started_at', label: 'Started', sortable: true },
		{ key: 'last_activity_at', label: 'Last Activity', sortable: true },
	];

	let sortKey = $state<string | undefined>('user_label');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	const sortedRows = $derived.by(() => {
		if (!sortKey) return rows;
		const key = sortKey as keyof Row;
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...rows].sort((a, b) => {
			const av = a[key];
			const bv = b[key];
			if (av < bv) return -1 * direction;
			if (av > bv) return 1 * direction;
			return 0;
		});
	});
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Running agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every currently active agent session.
</p>

<div class="mt-6">
	{#if sortedRows.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		<DataTable {columns} bind:sortKey bind:sortDirection>
			{#each sortedRows as row (row.agent_session_id)}
				<tr>
					<td>{row.user_label}</td>
					<td>{row.agent_type}</td>
					<td>{row.channel}</td>
					<td>{new Date(row.started_at).toLocaleString()}</td>
					<td>{new Date(row.last_activity_at).toLocaleString()}</td>
				</tr>
			{/each}
		</DataTable>
	{/if}
</div>
```

Per the spec's explicit scope decision: the per-user `Card` grouping is replaced by a flat table with `User` as an ordinary sortable column, sorted by `User` by default — this is a deliberate design choice (Material's `DataTable` spec has no "grouped rows" pattern), not an oversight. A person's agents are still trivially identifiable by sorting on the `User` column (the default sort).

- [ ] **Step 2: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 3: Self-review**

Confirm: the empty state ("No agents are currently running.") still renders when `data.agents.users` is empty; clicking a sortable column header toggles ascending/descending and updates the sort indicator icon; the table's rows still show every field the old Card-based layout showed (agent type, channel, started time, last activity — `last_activity_at` was not previously displayed at all in the Card layout, so this is a genuine improvement, not a behavior change to verify against old output).

- [ ] **Step 4: Commit**

```bash
git add "frontend/src/routes/admin/(protected)/agents/+page.svelte"
git commit -m "refactor: convert the admin agents page to a sortable DataTable"
```

---

### Task 8: Refactor — chat page (`Menu`, `MenuItem`, `List`, `ListItem`, `IconButton`, `Select`)

**Files:**
- Modify: `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`

**Interfaces:**
- Consumes: `Menu`, `MenuItem` (Task 2), `List`, `ListItem` (Task 3), `IconButton` (Task 1), `Select` (Task 5).

- [ ] **Step 1: Replace the hand-rolled `⋮` dropdown with `Menu`, and every list-of-rows inside it with the matching new component**

Replace the full contents of `frontend/src/routes/(app)/chat/[sessionId]/+page.svelte`:

```svelte
<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let menuOpen = $state(false);
	let menuView = $state<'root' | 'model' | 'personality'>('root');
	let showCustomForm = $state(false);
	let messagesContainer: HTMLDivElement | undefined = $state();
	let messageInput: HTMLInputElement | undefined = $state();

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	const CUSTOM_PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];

	const activeModelLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return 'Default model';
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? 'Default model';
		}
		return selection.label;
	});

	const currentPersonalityLabel = $derived.by(() => {
		const current = data.personality.versions.find((v) => v.is_current);
		return current?.description ?? 'Not set';
	});

	function closeMenu() {
		menuOpen = false;
	}

	// Always keep the latest message (and the "Typing…" indicator) in view — re-runs whenever
	// the message list changes or a reply starts streaming.
	$effect(() => {
		data.messages.length;
		pendingReply;
		messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight });
	});

	onMount(() => {
		messageInput?.focus();

		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${page.params.sessionId}/ws`);

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				connectionLost = false;
				if (hasConnectedBefore) {
					// Reconnected after a drop; neither leg replays missed events, so re-fetch to
					// reconcile anything that happened while disconnected.
					invalidateAll();
				}
				hasConnectedBefore = true;
			});

			socket.addEventListener('message', (event) => {
				let envelope: { kind: string };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				if (envelope.kind === 'Delta') {
					pendingReply = true;
				} else if (envelope.kind === 'TurnCompleted') {
					pendingReply = false;
					turnError = false;
					invalidateAll();
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				}
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					connectionLost = true;
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<div class="flex h-full flex-col" style="background: var(--md-sys-color-surface)">
	<div bind:this={messagesContainer} class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each data.messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if pendingReply}
			<div class="flex justify-start">
				<div
					class="md-body-large max-w-md px-4 py-2"
					style="background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface-variant); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)"
				>
					Typing…
				</div>
			</div>
		{/if}
		{#if turnError}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">
				Something went wrong — try sending again.
			</p>
		{/if}
		{#if form?.error}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		{#if connectionLost}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">
				Couldn't connect to this chat — try reloading the page.
			</p>
		{/if}
	</div>

	<div
		class="px-6 py-4"
		style="background: var(--md-sys-color-surface-container-low); border-top: 1px solid var(--md-sys-color-outline-variant)"
	>
		<div class="mb-2 flex justify-end">
			<Menu bind:open={menuOpen}>
				{#snippet trigger({ toggle })}
					<Button
						type="button"
						variant="outlined"
						onclick={() => {
							toggle();
							menuView = 'root';
						}}
						aria-label="Chat settings"
					>
						<Icon name="more" />
					</Button>
				{/snippet}
				<div class="w-80">
					{#if menuView === 'root'}
						<MenuItem onclick={() => (menuView = 'model')}>
							<div class="min-w-0 flex-1 text-left">
								<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">Model</p>
								<p class="md-body-small truncate" style="color: var(--md-sys-color-on-surface-variant)">
									{activeModelLabel}
								</p>
							</div>
							<Icon name="chevron-right" />
						</MenuItem>
						<MenuItem onclick={() => (menuView = 'personality')}>
							<div class="min-w-0 flex-1 text-left">
								<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">Personality</p>
								<p class="md-body-small truncate" style="color: var(--md-sys-color-on-surface-variant)">
									{currentPersonalityLabel}
								</p>
							</div>
							<Icon name="chevron-right" />
						</MenuItem>
					{:else}
						<div class="mb-2 flex items-center gap-1 px-1 pt-1">
							<IconButton onclick={() => (menuView = 'root')} aria-label="Back">
								<Icon name="chevron-left" size={18} />
							</IconButton>
							<p class="md-label-medium" style="color: var(--md-sys-color-on-surface-variant)">
								{menuView === 'model' ? 'Available models' : 'Personality history'}
							</p>
						</div>

						{#if menuView === 'model'}
							{#if form?.modelError}
								<p class="md-body-small mb-2 px-2" style="color: var(--md-sys-color-error)">{form.modelError}</p>
							{/if}
							{#each data.models.admin_models as model (model.id)}
								<form
									method="POST"
									action="?/selectAdminModel"
									use:enhance={() => {
										return async ({ update }) => {
											await update();
											closeMenu();
										};
									}}
								>
									<input type="hidden" name="admin_model_id" value={model.id} />
									<MenuItem
										type="submit"
										selected={data.models.selection?.kind === 'admin' &&
											data.models.selection.admin_model_id === model.id}
									>
										{model.label}
									</MenuItem>
								</form>
							{/each}

							<p class="md-label-medium mt-3 mb-1 px-2" style="color: var(--md-sys-color-on-surface-variant)">
								Your own key
							</p>
							{#if data.models.selection?.kind === 'custom'}
								<p class="md-body-medium px-2 py-1" style="font-weight: 600; color: var(--md-sys-color-on-surface)">
									{data.models.selection.label} ({data.models.selection.api_key_masked})
								</p>
							{/if}
							{#if showCustomForm}
								<form
									method="POST"
									action="?/selectCustomModel"
									use:enhance={() => {
										return async ({ update }) => {
											await update({ reset: true });
											showCustomForm = false;
										};
									}}
									class="mt-1 space-y-1 px-2"
								>
									<input name="label" type="text" placeholder="Label" required class="m3-picker-input" />
									<Select label="Provider" name="provider" options={CUSTOM_PROVIDER_OPTIONS} />
									<input name="model_id" type="text" placeholder="Model ID" class="m3-picker-input" />
									<input name="api_key" type="password" placeholder="API key" class="m3-picker-input" />
									<input name="base_url" type="text" placeholder="Base URL (optional)" class="m3-picker-input" />
									<Button type="submit" variant="filled" class="w-full">Save & validate</Button>
								</form>
							{:else}
								<MenuItem type="button" onclick={() => (showCustomForm = true)}>+ Use your own API key</MenuItem>
							{/if}
						{:else}
							{#if form?.personalityError}
								<p class="md-body-small mb-2 px-2" style="color: var(--md-sys-color-error)">{form.personalityError}</p>
							{/if}
							{#if data.personality.versions.length === 0}
								<p class="md-body-medium px-2 py-1" style="color: var(--md-sys-color-on-surface-variant)">
									You haven't set a personality yet — just ask nomi to change it.
								</p>
							{:else}
								<List>
									{#each data.personality.versions as version (version.version)}
										<ListItem
											headline={version.description}
											supportingText={`v${version.version} · ${new Date(version.created_at).toLocaleString()}`}
											selected={version.is_current}
										>
											{#snippet trailing()}
												{#if !version.is_current}
													<form
														method="POST"
														action="?/restorePersonality"
														use:enhance={() => {
															return async ({ update }) => {
																await update();
																closeMenu();
															};
														}}
													>
														<input type="hidden" name="version" value={version.version} />
														<Button type="submit" variant="text">Restore</Button>
													</form>
												{/if}
											{/snippet}
										</ListItem>
									{/each}
								</List>
							{/if}
						{/if}
					{/if}
				</div>
			</Menu>
		</div>
		<form
			method="POST"
			action="?/sendMessage"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
					messageInput?.focus();
				};
			}}
		>
			<div
				class="flex items-center gap-2 px-4 py-2"
				style="background: var(--md-sys-color-surface); border-radius: var(--md-sys-shape-corner-full); border: 1px solid var(--md-sys-color-outline)"
			>
				<input
					bind:this={messageInput}
					name="text"
					type="text"
					placeholder="Ask me anything..."
					required
					class="md-body-large flex-1 border-none bg-transparent outline-none"
					style="color: var(--md-sys-color-on-surface)"
				/>
				<Button type="submit" variant="filled">Send</Button>
			</div>
		</form>
	</div>
</div>

<style>
	.m3-picker-input {
		width: 100%;
		box-sizing: border-box;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		padding: 6px 8px;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-picker-input:focus {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 5px 7px;
	}
</style>
```

Note what's gone from the old `<style>` block: `.m3-picker-item`, `.m3-picker-item--selected`, `.m3-menu-row`, `.m3-personality-item`, `.m3-personality-item--current`, `.m3-icon-back` — all superseded by `Menu`/`MenuItem`/`List`/`ListItem`/`IconButton`'s own styles. `.m3-picker-input` **stays** — the custom-model form's plain `label`/`model_id`/`api_key`/`base_url` inputs still use it; they were never part of this refactor's scope (no `<label>` text, a different, valid compact-form design distinct from `TextField`, per the spec's Out of Scope).

- [ ] **Step 2: Verify**

```bash
cd frontend && npm run check
```

Expected: no errors.

- [ ] **Step 3: Manual behavioral verification** (this page has the most complex interaction of the whole plan)

In a real browser:
- The chat still auto-scrolls to the latest message and the input still auto-focuses (Menu/List/Select refactor must not have disturbed the existing scroll/focus `$effect`/`onMount` logic — re-verify both still work).
- Clicking the `⋮` button opens the menu at the root view (Model / Personality); clicking outside or pressing Escape closes it; clicking either root row drills into that sub-view; the back arrow returns to root.
- Selecting a model from the list closes the menu and the page reflects the new selection (`use:enhance`'s existing `update()` → `closeMenu()` flow, now via `MenuItem type="submit"` inside the existing form — confirm the form still actually submits, since a `<button role="menuitem">` losing its native form-submit affordance would be a silent regression here).
- The "+ Use your own API key" flow still opens the custom-model form, and its `Select` still defaults to Anthropic and updates the hidden `provider` field when changed.
- The personality history list still shows every version with the current one visually marked, and "Restore" still works and closes the menu.

- [ ] **Step 4: Commit**

```bash
git add "frontend/src/routes/(app)/chat/[sessionId]/+page.svelte"
git commit -m "refactor: consolidate the chat page's menu/list/select into the new m3 components"
```
