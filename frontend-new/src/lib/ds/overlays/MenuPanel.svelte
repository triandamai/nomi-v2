<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type MenuItem = { label: string; icon?: Component<LucideProps>; shortcut?: string; meta?: string; danger?: boolean; hover?: boolean; onselect?: () => void };
	export type MenuEntry = MenuItem | 'divider' | { group: string };
</script>

<script lang="ts">
	import MenuRow from './MenuRow.svelte';

	// Design System / Nomi → Overlays / Menu: the panel itself (surface, border-strong drawn inside,
	// the one shadow), its items 30 tall. Presentational: Menu floats it off a button and handles
	// the keyboard; the gallery shows it open. `hover` shows an item's hover without a pointer.
	// The pen's menu keeps its items at their own 220 inside the 236 panel; wide menus fill.
	type Props = {
		items: MenuEntry[];
		title?: string;
		footer?: string;
		wide?: boolean;
		id?: string;
		element?: HTMLElement;
		onpick?: (item: MenuItem) => void;
		[attribute: string]: unknown;
	};
	let { items, title, footer, wide = false, id, element = $bindable(), onpick, ...rest }: Props = $props();

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
		<hr class="border-border" />
	{:else}
		<span class={wide ? 'contents' : 'w-55'}><MenuRow item={entry} {wide} {onpick} /></span>
	{/if}
{/snippet}

<div
	bind:this={element}
	{id}
	role="menu"
	tabindex="-1"
	{...rest}
	class={['flex flex-col gap-0.5 rounded-md bg-surface p-1 text-text shadow-float outline -outline-offset-1 outline-border-strong', wide ? 'w-70' : 'w-59', rest.class as string | undefined]}
>
	{#if title}<p class="font-mono text-label tracking-label text-text-3 uppercase">{title}</p>{/if}
	{#each sections as section, at (at)}
		{#if section.label}
			<div role="group" aria-labelledby="{id}-group-{at}" class="flex flex-col gap-0.5">
				<p id="{id}-group-{at}" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{section.label}</p>
				{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
			</div>
		{:else}
			{#each section.entries as entry, index (index)}{@render row(entry)}{/each}
		{/if}
	{/each}
	{#if footer}
		<hr class="border-border" />
		<p class="px-2 py-1.5 font-mono text-label text-text-3">{footer}</p>
	{/if}
</div>
