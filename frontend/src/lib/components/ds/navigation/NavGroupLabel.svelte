<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * Names a group of drawer rows (Talk, Build, Life), in the mono meta face. Anything beside it
	 * (the workspace chip by Build) sits at the row's end.
	 */
	let { label, children }: { label: string; children?: Snippet } = $props();
</script>

{#if children}
	<div class="group">
		<p class="nomi-meta label">{label}</p>
		{@render children()}
	</div>
{:else}
	<p class="nomi-meta label solo">{label}</p>
{/if}

<style>
	.group {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 16px 16px 8px;
	}
	.label {
		min-width: 0;
		flex: 1;
		margin: 0;
	}
	.solo {
		padding: 16px 16px 8px;
	}
</style>
