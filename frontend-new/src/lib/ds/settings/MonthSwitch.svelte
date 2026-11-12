<script lang="ts">
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Plan & usage (C0og49) → Month Switch: which month is shown, between the one
	// before and the one after, in an outlined 32px group. There's no month after this one, so
	// its arrow waits (no link) on the current month.
	type Props = { month: string; previous: { href: string; label: string }; next?: { href: string; label: string }; nextLabel: string };
	let { month, previous, next, nextLabel }: Props = $props();
</script>

<div class="flex h-8 shrink-0 items-center gap-0.5 rounded-sm border border-border-strong px-px">
	<a href={previous.href} aria-label={previous.label} data-sveltekit-noscroll class="inline-flex size-6.5 items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text"><Icon icon={ChevronLeft} /></a>
	<span class="px-2.5 text-ui font-medium text-text" aria-live="polite">{month}</span>
	{#if next}
		<a href={next.href} aria-label={next.label} data-sveltekit-noscroll class="inline-flex size-6.5 items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text"><Icon icon={ChevronRight} /></a>
	{:else}
		<span role="link" aria-disabled="true" aria-label={nextLabel} class="inline-flex size-6.5 items-center justify-center text-border-strong"><Icon icon={ChevronRight} /></span>
	{/if}
</div>
