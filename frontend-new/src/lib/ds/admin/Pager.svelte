<script lang="ts">
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import Button from '../actions/Button.svelte';

	// Admin → Pager: under a long list, where the person is in it ("21–40 of 134", in mono) and the
	// pages either side as links, so the address says which page this is and the back button works.
	// At either end that way is shown but can't be followed.
	type Props = {
		/** Names the navigation ("People pages"). */
		label: string;
		/** "21–40 of 134". */
		summary: string;
		previousLabel: string;
		nextLabel: string;
		previousHref?: string;
		nextHref?: string;
	};
	let { label, summary, previousLabel, nextLabel, previousHref, nextHref }: Props = $props();
</script>

<nav aria-label={label} class="flex w-full flex-wrap items-center justify-between gap-3">
	<span class="font-mono text-label text-text-3">{summary}</span>
	<span class="flex items-center gap-2">
		{#if previousHref}
			<Button label={previousLabel} variant="secondary" iconStart={ChevronLeft} href={previousHref} />
		{:else}
			<Button label={previousLabel} variant="secondary" iconStart={ChevronLeft} disabled />
		{/if}
		{#if nextHref}
			<Button label={nextLabel} variant="secondary" iconEnd={ChevronRight} href={nextHref} />
		{:else}
			<Button label={nextLabel} variant="secondary" iconEnd={ChevronRight} disabled />
		{/if}
	</span>
</nav>
