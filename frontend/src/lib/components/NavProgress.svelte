<script lang="ts">
	// A thin bar along the top while the next page loads, so a tap never looks ignored. It waits a
	// beat before showing (most moves are faster than that), then runs until the page arrives.
	import { navigating } from '$app/state';
	import { m } from '$lib/paraglide/messages';

	/** How long a navigation can take before the bar shows. */
	const SHOW_AFTER_MS = 120;

	let visible = $state(false);
	$effect(() => {
		if (!navigating.to) {
			visible = false;
			return;
		}
		const timer = setTimeout(() => (visible = true), SHOW_AFTER_MS);
		return () => clearTimeout(timer);
	});
</script>

{#if visible}
	<div class="nav-progress" role="progressbar" aria-label={m.nav_loading()}><span class="nav-progress__bar"></span></div>
{/if}

<style>
	.nav-progress {
		position: fixed;
		inset: 0 0 auto;
		z-index: 1000;
		height: 3px;
		overflow: hidden;
		pointer-events: none;
		background: color-mix(in srgb, var(--md-sys-color-primary) 18%, transparent);
	}
	.nav-progress__bar {
		position: absolute;
		inset: 0 auto 0 0;
		width: 40%;
		border-radius: 0 3px 3px 0;
		background: var(--md-sys-color-primary);
		animation: nav-progress 1.1s cubic-bezier(0.4, 0, 0.2, 1) infinite;
	}
	@keyframes nav-progress {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(250%);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.nav-progress__bar {
			width: 100%;
			animation: none;
			opacity: 0.7;
		}
	}
</style>
