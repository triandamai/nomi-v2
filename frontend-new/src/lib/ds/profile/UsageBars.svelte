<script lang="ts">
	// App / Settings / Profile (U9Gnrg) → Models you use most: each model's name in mono, its
	// tokens and share at the end, and a 4px bar under them against the most used, 12 apart. With a
	// note (Plan & usage → By model) the value is the spend, stronger, and the note goes under the bar.
	export type UsageBar = { label: string; value: string; share: number; note?: string };
	type Props = { bars: UsageBar[]; empty: string };
	let { bars, empty }: Props = $props();
</script>

{#if bars.length}
	<ul class="flex flex-col gap-3">
		{#each bars as bar (bar.label)}
			<li class="flex flex-col gap-1.25">
				<span class="flex items-center justify-between gap-2 font-mono text-label">
					<span class="min-w-0 truncate text-text">{bar.label}</span>
					<span class={['shrink-0', bar.note ? 'text-text-2' : 'text-text-3']}>{bar.value}</span>
				</span>
				<span class="h-1 overflow-hidden rounded-cell bg-surface-2">
					<span class="block h-full rounded-cell bg-text-2" style="width: {Math.round(Math.min(1, Math.max(0, bar.share)) * 100)}%"></span>
				</span>
				{#if bar.note}<span class="truncate font-mono text-label text-text-3">{bar.note}</span>{/if}
			</li>
		{/each}
	</ul>
{:else}
	<p class="text-caption text-text-3">{empty}</p>
{/if}
