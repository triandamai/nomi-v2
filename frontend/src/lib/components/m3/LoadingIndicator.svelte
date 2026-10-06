<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import MorphingShape from './MorphingShape.svelte';
	import type { GradientTone } from './shapes';

	// M3 Expressive loading indicator: a morphing shape (see MorphingShape) that also rotates.

	let {
		size = 48,
		tone = 'glow',
		contained = false,
		label = m.loading_working(),
	}: { size?: number; tone?: GradientTone; contained?: boolean; label?: string } = $props();
</script>

<span
	class="loading-indicator"
	class:loading-indicator--contained={contained}
	style="--size: {size}px"
	role="progressbar"
	aria-label={label}
>
	<MorphingShape size={contained ? Math.round(size * 0.62) : size} {tone} class="loading-indicator__shape" />
</span>

<style>
	.loading-indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex: none;
		width: var(--size);
		height: var(--size);
	}
	.loading-indicator :global(.loading-indicator__shape) {
		animation: loading-indicator-spin 2.6s linear infinite;
	}
	.loading-indicator--contained {
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-inverse-surface);
	}
	@keyframes loading-indicator-spin {
		to {
			rotate: 360deg;
		}
	}
</style>
