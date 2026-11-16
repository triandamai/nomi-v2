<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Like & dislike → Long-press sheet: the quick actions at a sheet's top, in
	// equal tiles 8 apart ([0,16,8,16]): surface-2 with 14 corners, the 20 icon over its name at 13.
	// One that's on (Good, once liked) shows it in accent and says so (aria-pressed).
	type Tile = { label: string; icon: Component<LucideProps>; pressed?: boolean; onclick: () => void };
	type Props = { tiles: Tile[] };
	let { tiles }: Props = $props();
</script>

<div class="flex gap-2 px-4 pb-2">
	{#each tiles as tile (tile.label)}
		<button
			type="button"
			aria-pressed={tile.pressed === undefined ? undefined : tile.pressed}
			class={['flex h-16 min-w-0 flex-1 cursor-pointer flex-col items-center justify-center gap-1 rounded-xl bg-surface-2 text-ui outline-offset-2', tile.pressed ? 'text-accent' : 'text-text']}
			onclick={tile.onclick}
		>
			<Icon icon={tile.icon} size={20} />{tile.label}
		</button>
	{/each}
</div>
