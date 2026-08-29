<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		selected = false,
		children,
		class: extraClass = '',
		...rest
	}: {
		selected?: boolean;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes = $props();
</script>

<button
	type="button"
	role="menuitem"
	tabindex="-1"
	class="m3-menu-item {extraClass}"
	class:m3-menu-item--selected={selected}
	{...rest}
>
	{@render children()}
</button>

<style>
	.m3-menu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		border: none;
		background: transparent;
		cursor: pointer;
		text-align: left;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		color: var(--md-sys-color-on-surface);
	}
	.m3-menu-item:hover,
	.m3-menu-item:focus-visible {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		outline: none;
	}
	.m3-menu-item--selected {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
</style>
