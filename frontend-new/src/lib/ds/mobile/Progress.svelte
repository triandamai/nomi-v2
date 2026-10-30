<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Mobile / Progress: how far along a set of questions is, one segment each, the ones reached in
	// text colour.
	let { current, total }: { current: number; total: number } = $props();
</script>

<div role="progressbar" aria-label={m.mobile_progress({ current: String(current), total: String(total) })} aria-valuemin={1} aria-valuemax={total} aria-valuenow={current} class="flex w-full gap-1.5">
	{#each { length: total } as _, index (index)}
		<span data-segment class={['h-1 flex-1 rounded-xs', index < current ? 'bg-text' : 'bg-border-strong']}></span>
	{/each}
</div>
