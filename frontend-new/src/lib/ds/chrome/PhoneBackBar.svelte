<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronLeft } from '@lucide/svelte';

	// Mobile / Back Bar (F4DAJ), as Flow / Mobile — You & Settings draws it on every pushed screen: 48
	// high, padded [0,8], the way back ("‹ You") at 16 with the 24 chevron, and the screen's own
	// action at the end where it has one. A link where back is a page, a button where it's a step
	// inside the page (a form screen going back to its list).
	type Props = { label: string; href?: string; onback?: () => void; trailing?: Snippet };
	let { label, href, onback, trailing }: Props = $props();
</script>

<div class="flex h-12 shrink-0 items-center gap-1 px-2">
	{#if href}
		<a {href} class="inline-flex h-11 items-center gap-0.5 rounded-full pr-2 pl-1 text-phone text-text outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus">
			<ChevronLeft size={24} strokeWidth={1.5} aria-hidden="true" />{label}
		</a>
	{:else}
		<button type="button" class="inline-flex h-11 cursor-pointer items-center gap-0.5 rounded-full pr-2 pl-1 text-phone text-text outline-offset-1 focus-visible:outline-2 focus-visible:outline-focus" onclick={onback}>
			<ChevronLeft size={24} strokeWidth={1.5} aria-hidden="true" />{label}
		</button>
	{/if}
	<span class="flex-1"></span>
	{@render trailing?.()}
</div>
