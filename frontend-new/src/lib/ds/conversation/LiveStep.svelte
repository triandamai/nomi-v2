<script lang="ts">
	import { LoaderCircle, Square } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// The step an agent is on right now, as it really is ("Writing src/routes/menu/+page.svelte"),
	// with the time so far: no progress bar when nobody knows how long it takes.
	type Props = { step: string; elapsed?: string; stopLabel?: string; onstop?: () => void };
	let { step, elapsed, stopLabel, onstop }: Props = $props();
</script>

<div class="flex w-full items-center gap-2 rounded-sm border border-border px-3 py-2">
	<span class="inline-flex animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>
	<span role="status" class="min-w-0 flex-1 truncate font-mono text-label text-text">{step}</span>
	{#if elapsed}<span class="font-mono text-label text-text-3">{elapsed}</span>{/if}
	{#if stopLabel && onstop}<Button label={stopLabel} variant="ghost" iconStart={Square} onclick={onstop} />{/if}
</div>
