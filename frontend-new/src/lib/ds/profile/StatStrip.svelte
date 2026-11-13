<script lang="ts">
	import { Flame } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Profile (U9Gnrg) → Stats Strip: the person's numbers side by side in one
	// outlined strip, 1px apart: the figure in big mono, what it is, and a mono note. The current
	// streak sits on the surface with the accent flame. Two to a row on narrow screens, the last
	// across both. `small` is Plan & usage's (C0og49 → Usage Stats): the name first at 12, the figure
	// at 15/500 mono, [14,16] in, four to a row.
	export type StatTile = { value: string; label: string; note: string; current?: boolean };
	type Props = { tiles: StatTile[]; small?: boolean };
	let { tiles, small = false }: Props = $props();
</script>

<dl class={['grid grid-cols-2 gap-px overflow-hidden rounded-md bg-border outline -outline-offset-1 outline-border', small ? 'md:grid-cols-4' : 'lg:grid-cols-5']}>
	{#each tiles as tile (tile.label)}
		<!-- The name comes first for screen readers ("Lifetime tokens, 186.4K"); the figure shows first. -->
		<div class={['flex flex-col gap-1', small ? 'px-4 py-3.5' : 'px-4.5 py-4 max-lg:last:col-span-2', tile.current ? 'bg-surface' : 'bg-bg']}>
			<dt class={[small ? 'text-caption' : 'order-2 text-ui', 'text-text-2']}>{tile.label}</dt>
			<dd class={['flex items-center gap-1.5', !small && 'order-1']}>
				<span class={['font-mono font-medium text-text', small ? 'text-body' : 'text-display tracking-stat']}>{tile.value}</span>
				{#if tile.current}<span class="inline-flex text-accent"><Icon icon={Flame} /></span>{/if}
			</dd>
			<dd class="order-3 font-mono text-label text-text-3">{tile.note}</dd>
		</div>
	{/each}
</dl>
