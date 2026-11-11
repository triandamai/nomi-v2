<script lang="ts">
	import SourceTile from './SourceTile.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Inbox (BFoiZ) → Coming Up: the days ahead at a glance, its name and count across the
	// top, then each day (its name 48 wide, today's in the text colour) with its entries: the
	// crew member's 26px tile, the title and the time in mono. 10 between the heading and each day.
	export type AgendaItem = { agent: CrewId; title: string; time: string };
	export type AgendaDay = { label: string; today?: boolean; items: AgendaItem[] };
	type Props = { label: string; count: number; days: AgendaDay[]; empty: string };
	let { label, count, days, empty }: Props = $props();

	const id = $props.id();
</script>

<section aria-labelledby={id} class="flex flex-col gap-2.5">
	<h2 {id} class="flex items-center justify-between font-mono text-label font-normal text-text-3">
		<span class="tracking-label uppercase">{label}</span>
		<span>{count}</span>
	</h2>
	{#each days as day (day.label)}
		<div class="flex items-start gap-3">
			<span class={['w-12 shrink-0 font-mono text-label', day.today ? 'text-text' : 'text-text-3']}>{day.label}</span>
			<ul class="flex min-w-0 flex-1 flex-col gap-1.5">
				{#each day.items as item, index (index)}
					<li class="flex items-center gap-2">
						<SourceTile agent={item.agent} size={26} />
						<span class="min-w-0 flex-1 truncate text-caption text-text">{item.title}</span>
						<span class="font-mono text-label text-text-3">{item.time}</span>
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<p class="text-caption text-text-3">{empty}</p>
	{/each}
</section>
