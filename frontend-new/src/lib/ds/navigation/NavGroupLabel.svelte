<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * Names a group of sidebar rows (Talk, Build, Life). Written in sentence case; shown in capitals.
	 * Anything beside it (the workspace chip by Build) sits at the row's end, centred, in the same
	 * 12 / 10 / 4 padding.
	 */
	let { label, children }: { label: string; children?: Snippet } = $props();
</script>

{#if children}
	<div class="flex items-center gap-1.5 px-2.5 pt-3 pb-1">
		<p class="min-w-0 flex-1 font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
		{@render children()}
	</div>
{:else}
	<p class="px-2.5 pt-3 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
{/if}
