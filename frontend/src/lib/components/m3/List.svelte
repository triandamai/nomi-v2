<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, class: extraClass = '' }: { children: Snippet; class?: string } = $props();
</script>

<div role="list" class="m3-list {extraClass}">
	{@render children()}
</div>

<style>
	.m3-list {
		display: flex;
		flex-direction: column;
	}
</style>
