<script lang="ts">
	// Flow / Mobile — You & Settings, steps 7 and 8 (Profile's Stats, Plan & usage's Usage): numbers
	// two to a row, 10 apart (surface, border, radius 14, 14 in): what it is at 13, the figure at 20
	// in mono, and its note in mono under it, wrapping rather than cut.
	type Tile = { label: string; value: string; note: string };
	type Props = { tiles: Tile[] };
	let { tiles }: Props = $props();
</script>

<dl class="grid grid-cols-2 gap-2.5">
	{#each tiles as tile (tile.label)}
		<div class="flex min-w-0 flex-col gap-1 rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
			<dt class="text-caption text-text-3">{tile.label}</dt>
			<dd class="truncate font-mono text-heading text-text">{tile.value}</dd>
			<dd class="font-mono text-label text-text-3">{tile.note}</dd>
		</div>
	{/each}
</dl>
