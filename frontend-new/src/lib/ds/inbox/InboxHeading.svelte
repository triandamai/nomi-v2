<script lang="ts">
	// App / Inbox (BFoiZ) → Section: a feed section's name in mono capitals ("NEEDS YOU", "TODAY ·
	// WED 21 OCT"), stronger than the panel's labels, its count quieter beside it; 6 above the list.
	type Props = { label: string; count?: number | string };
	let { label, count }: Props = $props();
</script>

<h2 class="flex items-center gap-2 pb-1.5 font-mono text-label font-normal">
	<span class="tracking-label text-text-2 uppercase">{label}</span>
	{#if count !== undefined}<span class="text-text-3">{count}</span>{/if}
</h2>
