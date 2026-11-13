<script lang="ts">
	import MenuRow from '../overlays/MenuRow.svelte';
	import type { MenuEntry, MenuItem } from '../overlays/MenuPanel.svelte';

	// Design System / Projects → Project / Card Menu (uA0lh): the panel a card's ⋯ opens. 248 wide,
	// 4 in, 2 between rows (surface, border-strong inside, the float shadow); the project's name
	// in mono capitals (6 / 8 / 4 in, unlike the standard menu's bare title), then the pen's Menu
	// Items (MenuRow, 30 tall) in groups, a 1px divider between them. Presentational:
	// ProjectCardMenu floats it and handles the keyboard; the gallery shows it open.
	type Props = {
		title: string;
		items: MenuEntry[];
		id?: string;
		element?: HTMLElement;
		onpick?: (item: MenuItem) => void;
		[attribute: string]: unknown;
	};
	let { title, items, id, element = $bindable(), onpick, ...rest }: Props = $props();

	const uid = $props.id();
</script>

<div
	bind:this={element}
	{id}
	role="menu"
	tabindex="-1"
	aria-labelledby="{uid}-title"
	{...rest}
	class={['flex w-62 flex-col gap-0.5 rounded-md bg-surface p-1 text-text shadow-float outline -outline-offset-1 outline-border-strong', rest.class as string | undefined]}
>
	<p id="{uid}-title" role="presentation" class="truncate px-2 pt-1.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{title}</p>
	{#each items as entry, index (index)}
		{#if entry === 'divider'}
			<hr class="border-border" />
		{:else if !('group' in entry)}
			<MenuRow item={entry} {onpick} />
		{/if}
	{/each}
</div>
