<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'standard' | 'filled' | 'filled-tonal' | 'outlined';

	let {
		variant = 'standard',
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
	<a {href} class="m3-icon-btn m3-icon-btn--{variant} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button type="button" class="m3-icon-btn m3-icon-btn--{variant} {extraClass}" {...rest}>
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
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
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
		border: 1px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-icon-btn--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>
