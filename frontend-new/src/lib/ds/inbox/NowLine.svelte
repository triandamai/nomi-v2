<script lang="ts">
	// App / Inbox (BFoiZ) → Now Line: where now falls among today's entries, the time in the time
	// column, a 1px rule and "now", all in the accent on its quiet tint.
	type Props = { time: string; label: string };
	let { time, label }: Props = $props();
</script>

<div role="separator" aria-label="{label} {time}" class="flex items-center gap-3 rounded-xs bg-accent-bg p-1">
	<span class="w-14 shrink-0 font-mono text-label text-accent">{time}</span>
	<span class="h-px min-w-0 flex-1 bg-accent"></span>
	<span class="font-mono text-label text-accent">{label}</span>
</div>
