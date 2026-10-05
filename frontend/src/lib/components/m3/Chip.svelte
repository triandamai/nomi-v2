<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	type Variant = 'assist' | 'filter' | 'suggestion';

	let {
		variant = 'assist',
		selected = false,
		leading,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		/** filter chips only — renders a check and the selected container. */
		selected?: boolean;
		leading?: Snippet;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes = $props();
</script>

<button
	type="button"
	class="m3-chip m3-chip--{variant} {extraClass}"
	class:m3-chip--selected={variant === 'filter' && selected}
	aria-pressed={variant === 'filter' ? selected : undefined}
	{...rest}
>
	{#if variant === 'filter' && selected}
		<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
	{:else if leading}
		{@render leading()}
	{/if}
	{@render children()}
</button>

<style>
	.m3-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 36px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1.5px solid var(--md-sys-color-outline-variant);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-color var(--nomi-motion-effects-fast),
			border-radius var(--nomi-motion-spatial-fast),
			padding var(--nomi-motion-spatial-fast);
	}
	.m3-chip:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-chip:active {
		border-radius: var(--md-sys-shape-corner-small);
	}
	.m3-chip:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	.m3-chip--suggestion {
		height: 40px;
		padding: 0 16px;
	}
	.m3-chip--filter {
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-chip--selected {
		padding-left: 10px;
		border-color: var(--md-sys-color-primary-container);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.m3-chip--selected:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-primary-container) 8%, var(--md-sys-color-primary-container));
	}
</style>
