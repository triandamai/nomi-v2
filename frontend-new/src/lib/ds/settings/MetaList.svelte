<script lang="ts">
	// App / Settings / Plan & usage (C0og49) → Billing: facts in pairs under a mono label, each its
	// name and its value in mono, 32 tall over a stroke (the last without).
	type Props = { label: string; rows: { label: string; value: string }[] };
	let { label, rows }: Props = $props();
</script>

<div class="flex flex-col gap-2">
	<p class="font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
	<dl class="flex flex-col">
		{#each rows as row (row.label)}
			<div class="flex items-center justify-between gap-2 border-b border-border pt-2 pb-1.75 last:border-b-0 last:pb-2">
				<dt class="min-w-0 truncate text-caption text-text-2">{row.label}</dt>
				<dd class="shrink-0 font-mono text-label text-text">{row.value}</dd>
			</div>
		{/each}
	</dl>
</div>
