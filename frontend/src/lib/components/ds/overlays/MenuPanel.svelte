<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type MenuItem = { label: string; icon?: Component<LucideProps>; shortcut?: string; meta?: string; danger?: boolean; hover?: boolean; onselect?: () => void };
	export type MenuEntry = MenuItem | 'divider' | { group: string };
</script>
<script lang="ts">
	import MenuRow from './MenuRow.svelte';

	// The M3 menu panel itself. Presentational: Menu floats it off a button and handles the
	// keyboard; the gallery shows it open. `hover` shows an item's hover without a pointer.
	type Props = {
		items: MenuEntry[];
		title?: string;
		footer?: string;
		wide?: boolean;
		/** The chat menu is 260 wide; other wide menus 280. */
		width?: 260;
		id?: string;
		element?: HTMLElement;
		onpick?: (item: MenuItem) => void;
		[attribute: string]: unknown;
	};
	let { items, title, footer, wide = false, width, id, element = $bindable(), onpick, ...rest }: Props = $props();

	const sections = $derived.by(() => {
		const out: { label?: string; entries: (MenuItem | 'divider')[] }[] = [{ entries: [] }];
		for (const entry of items) {
			if (typeof entry === 'object' && 'group' in entry) out.push({ label: entry.group, entries: [] });
			else out[out.length - 1].entries.push(entry);
		}
		return out.filter((section) => section.label || section.entries.length > 0);
	});
</script>

{#snippet row(entry: MenuItem | 'divider')}
	{#if entry === 'divider'}
		<hr class="divider" />
	{:else}
		<MenuRow item={entry} {wide} {onpick} />
	{/if}
{/snippet}

<div bind:this={element} {id} role="menu" tabindex="-1" {...rest} class="menu {width === 260 ? 'menu--260' : wide ? 'menu--wide' : ''} {rest.class ?? ''}">
	{#if title}<p class="nomi-meta heading">{title}</p>{/if}
	{#each sections as section, at (at)}
		{#if section.label}
			<div role="group" aria-labelledby="{id}-group-{at}" class="group">
				<p id="{id}-group-{at}" role="presentation" class="nomi-meta heading">{section.label}</p>
				{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
			</div>
		{:else}
			{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
		{/if}
	{/each}
	{#if footer}
		<hr class="divider" />
		<p class="footer">{footer}</p>
	{/if}
</div>

<style>
	.menu {
		display: flex;
		width: 236px;
		flex-direction: column;
		padding: 8px 0;
		border: 0;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
	.menu--260 {
		width: 260px;
	}
	.menu--wide {
		width: 280px;
	}
	.group {
		display: flex;
		flex-direction: column;
	}
	.heading {
		margin: 0;
		padding: 8px 16px 4px;
	}
	.divider {
		margin: 8px 0;
		border: 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.footer {
		margin: 0;
		padding: 8px 16px;
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
