<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'filled' | 'tonal' | 'outlined' | 'text' | 'elevated';

	let {
		variant = 'filled',
		href,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		href?: string;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes &
		HTMLAnchorAttributes = $props();
</script>

{#if href}
	<a {href} class="m3-button m3-button--{variant} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button class="m3-button m3-button--{variant} {extraClass}" {...rest}>
		{@render children()}
	</button>
{/if}

<style>
	.m3-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 40px;
		padding: 0 24px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		cursor: pointer;
		white-space: nowrap;
		text-decoration: none;
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		line-height: var(--md-sys-typescale-label-large-line-height);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		transition:
			background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
			box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-button:disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-button--filled {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-button--filled:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--tonal {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.m3-button--tonal:not(:disabled):hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--elevated {
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-primary);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--outlined {
		background: transparent;
		color: var(--md-sys-color-primary);
		border: 1px solid var(--md-sys-color-outline);
	}
	.m3-button--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}

	.m3-button--text {
		background: transparent;
		color: var(--md-sys-color-primary);
		padding: 0 12px;
	}
	.m3-button--text:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
</style>
