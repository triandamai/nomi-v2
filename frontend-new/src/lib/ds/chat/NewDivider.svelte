<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Chat responses → New Divider: where you left off, in mint, with how much came in since.
	let { count }: { count: number } = $props();

	const label = $derived(count === 1 ? m.new_divider_one() : m.new_divider({ count: String(count) }));
</script>

<div role="separator" aria-label={label} class="flex w-full items-center gap-2.5">
	<span class="h-px flex-1 bg-accent"></span>
	<span aria-hidden="true" class="font-mono text-label tracking-label text-accent">{label}</span>
	<span class="h-px flex-1 bg-accent"></span>
</div>
