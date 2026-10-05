<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'standard' | 'filled' | 'filled-tonal' | 'outlined';

	let {
		variant = 'standard',
		selected,
		href,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		/** Makes this a toggle: announces aria-pressed and morphs round → square when on. */
		selected?: boolean;
		href?: string;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes &
		HTMLAnchorAttributes = $props();
</script>

{#if href}
	<a {href} class="m3-icon-btn m3-icon-btn--{variant} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button
		type="button"
		class="m3-icon-btn m3-icon-btn--{variant} {extraClass}"
		class:m3-icon-btn--selected={selected}
		aria-pressed={selected}
		{...rest}
	>
		{@render children()}
	</button>
{/if}

<style>
	.m3-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		cursor: pointer;
		text-decoration: none;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast),
			color var(--nomi-motion-effects-fast);
	}

	.m3-icon-btn:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-medium);
	}

	/* Toggle "on": shape, not just color, says selected. */
	.m3-icon-btn.m3-icon-btn--selected {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		border-color: transparent;
	}

	.m3-icon-btn:disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-icon-btn--standard {
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-icon-btn--standard:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}

	.m3-icon-btn--filled {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-icon-btn--filled:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-icon-btn--filled-tonal {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.m3-icon-btn--filled-tonal:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-icon-btn--outlined {
		background: transparent;
		border: 1.5px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-icon-btn--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>
