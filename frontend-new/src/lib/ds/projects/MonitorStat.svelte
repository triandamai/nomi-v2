<script lang="ts">
	// Flow / Build — Workspace tools (U16wU) → Screen / Monitor (EQNtT) → Stat: one number of the
	// running app, [12,14] and 4 apart, outlined: its name in mono capitals, the value (15/500 mono)
	// and a note; "Not measured" in its place when Shipyard didn't measure it.
	type Props = { label: string; value?: string | null; note?: string; notMeasured?: string };
	let { label, value, note, notMeasured }: Props = $props();
</script>

<div data-part="stat" class="flex min-h-19.75 min-w-0 flex-1 flex-col gap-1 rounded-sm px-3.5 py-3 outline -outline-offset-1 outline-border">
	<span class="font-mono text-label tracking-label text-text-3 uppercase">{label}</span>
	{#if value}
		<span class="font-mono text-body font-medium text-text">{value}</span>
		{#if note}<span class="truncate font-mono text-label text-text-3">{note}</span>{/if}
	{:else}
		<span class="text-ui text-text-3">{notMeasured}</span>
	{/if}
</div>
