<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile — You & Settings, step 9 (Compare plans) → a plan in the sheet (bg, border,
	// radius 14, 14 in, 10 apart): its name at 17 with the price beside it, what it gives at 14, and
	// either YOUR PLAN in mono capitals or the one thing the person can do with it.
	type Props = { name: string; price: string; line: string; yours?: string; children?: Snippet };
	let { name, price, line, yours, children }: Props = $props();
</script>

<article aria-label={name} class="flex flex-col gap-2.5 rounded-xl bg-bg p-3.5 outline -outline-offset-1 outline-border">
	<div class="flex items-baseline gap-2">
		<h3 class="min-w-0 flex-1 truncate text-phone-title font-semibold text-text">{name}</h3>
		<span class="shrink-0 font-mono text-label text-text-2">{price}</span>
	</div>
	<p class="text-phone-sm text-text-2">{line}</p>
	{#if yours}
		<p class="font-mono text-label tracking-label text-text-3 uppercase">{yours}</p>
	{:else if children}
		<div class="flex flex-col gap-2">{@render children()}</div>
	{/if}
</article>
