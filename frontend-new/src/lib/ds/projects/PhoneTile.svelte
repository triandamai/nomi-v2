<script lang="ts">
	// Flow / Mobile Build → Monitor's tiles (M4GVXb, vhsAL): what's counted in mono capitals, the number
	// at 22, and what it counts under it, on surface-2, 12 round, 12 in, 4 apart. A number that wasn't
	// measured says so, never 0.
	type Props = { label: string; value?: string | null; note?: string; notMeasured: string };
	let { label, value, note, notMeasured }: Props = $props();
</script>

<div class="flex min-w-0 flex-1 flex-col gap-1 rounded-lg bg-surface-2 p-3">
	<span class="truncate font-mono text-label tracking-label text-text-3 uppercase">{label}</span>
	{#if value === null || value === undefined}
		<span class="text-phone-sm text-text-3">{notMeasured}</span>
	{:else}
		<span class="text-phone-display text-text tabular-nums">{value}</span>
	{/if}
	{#if note}<span class="truncate font-mono text-label text-text-3">{note}</span>{/if}
</div>
