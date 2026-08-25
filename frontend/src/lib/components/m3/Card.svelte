<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'outlined',
		class: extraClass = '',
		children
	}: {
		variant?: 'filled' | 'outlined' | 'elevated';
		class?: string;
		children: Snippet;
	} = $props();
</script>

<div class="m3-card m3-card--{variant} {extraClass}">
	{@render children()}
</div>

<style>
	.m3-card {
		border-radius: var(--md-sys-shape-corner-medium);
		overflow: hidden;
	}

	.m3-card--outlined {
		background: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
	}

	.m3-card--filled {
		background: var(--md-sys-color-surface-container-highest);
	}

	.m3-card--elevated {
		background: var(--md-sys-color-surface-container-low);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
</style>
