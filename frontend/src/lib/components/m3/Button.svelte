<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';

	type Variant = 'filled' | 'gradient' | 'tonal' | 'outlined' | 'text' | 'elevated';
	type Size = 'xs' | 's' | 'm' | 'l' | 'xl';

	let {
		variant = 'filled',
		size = 's',
		href,
		children,
		class: extraClass = '',
		...rest
	}: {
		variant?: Variant;
		size?: Size;
		href?: string;
		children: Snippet;
		class?: string;
	} & HTMLButtonAttributes &
		HTMLAnchorAttributes = $props();
</script>

{#if href}
	<a {href} class="m3-button m3-button--{variant} m3-button--size-{size} {extraClass}" {...rest}>
		{@render children()}
	</a>
{:else}
	<button class="m3-button m3-button--{variant} m3-button--size-{size} {extraClass}" {...rest}>
		{@render children()}
	</button>
{/if}

<style>
	/* M3 Expressive button: fully round at rest, corners tighten on press (shape morph on a
	   spring), so a press reads through shape — not just a color change. Each size rests at
	   half its height rather than corner-full, so the morph animates (see material3.css). */
	.m3-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		cursor: pointer;
		white-space: nowrap;
		text-decoration: none;
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: 600;
		font-size: var(--md-sys-typescale-label-large-size);
		line-height: var(--md-sys-typescale-label-large-line-height);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast),
			box-shadow var(--nomi-motion-effects-fast),
			filter var(--nomi-motion-effects-fast);
	}

	/* Declared before the variant rules below so .m3-button--text's own padding override
	   (same specificity, later in source order) correctly wins for text buttons at every
	   size — text buttons keep tighter horizontal padding regardless of size, matching MD3. */
	.m3-button--size-xs {
		height: 32px;
		border-radius: 16px;
		padding: 0 14px;
	}
	.m3-button--size-s {
		height: 40px;
		border-radius: 20px;
		padding: 0 18px;
	}
	.m3-button--size-s:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-small);
	}
	.m3-button--size-m {
		height: 56px;
		border-radius: 28px;
		padding: 0 24px;
		font-size: 1rem;
	}
	.m3-button--size-m:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.m3-button--size-l {
		height: 96px;
		border-radius: 48px;
		padding: 0 40px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.625rem;
		font-weight: 650;
	}
	.m3-button--size-l:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-extra-large);
	}
	.m3-button--size-xl {
		height: 136px;
		border-radius: 68px;
		padding: 0 56px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 2.25rem;
		font-weight: 700;
	}
	.m3-button--size-xl:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-extra-extra-large);
	}
	.m3-button--size-xs:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-small);
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

	/* The hero action. Use at most one per screen — gradient is a material, not a fill style. */
	.m3-button--gradient {
		background: var(--nomi-gradient-glow);
		color: var(--nomi-on-gradient-glow);
	}
	.m3-button--gradient:not(:disabled):hover {
		filter: saturate(1.15) brightness(1.03);
		box-shadow: 0 6px 16px -6px color-mix(in srgb, var(--md-sys-color-primary) 55%, transparent);
	}

	.m3-button--tonal {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.m3-button--tonal:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-secondary-container) 8%, var(--md-sys-color-secondary-container));
	}

	.m3-button--elevated {
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-primary);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-button--outlined {
		background: transparent;
		color: var(--md-sys-color-on-surface);
		border: 1.5px solid var(--md-sys-color-outline);
	}
	.m3-button--outlined:not(:disabled):hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
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
