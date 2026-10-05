<script lang="ts">
	import { onMount } from 'svelte';
	import { GRADIENT_STOPS, pointsToPath, shapePath, shapePoints, type GradientTone, type ShapeName, type Point } from './shapes';

	// M3 Expressive loading indicator: a shape that morphs through the sequence below while
	// rotating. Interpolated per animation frame in JS (every shape shares one point count — see
	// shapes.ts) rather than via CSS `d` keyframes, which aren't animatable in every browser.

	let {
		size = 48,
		tone = 'glow',
		contained = false,
		label = 'Working',
	}: { size?: number; tone?: GradientTone; contained?: boolean; label?: string } = $props();

	const SEQUENCE: ShapeName[] = ['cookie9', 'clover4', 'sunny8', 'flower5'];
	const STEP_MS = 850;

	const uid = $props.id();
	let d = $state(shapePath(SEQUENCE[0]));
	const stops = $derived(GRADIENT_STOPS[tone]);

	// Overshooting ease (same curve as --nomi-motion-spatial-default) — the "spring" in the morph.
	function spring(t: number): number {
		const c = 1.70158 * 1.2;
		return 1 + (c + 1) * Math.pow(t - 1, 3) + c * Math.pow(t - 1, 2);
	}

	function lerp(a: Point[], b: Point[], t: number): Point[] {
		return a.map(([ax, ay], i) => [ax + (b[i][0] - ax) * t, ay + (b[i][1] - ay) * t]);
	}

	onMount(() => {
		if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		let frame = 0;
		const start = performance.now();
		const tick = (now: number) => {
			const elapsed = now - start;
			const step = Math.floor(elapsed / STEP_MS);
			const t = (elapsed % STEP_MS) / STEP_MS;
			const from = shapePoints(SEQUENCE[step % SEQUENCE.length]);
			const to = shapePoints(SEQUENCE[(step + 1) % SEQUENCE.length]);
			d = pointsToPath(lerp(from, to, spring(Math.min(1, t * 1.6))));
			frame = requestAnimationFrame(tick);
		};
		frame = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(frame);
	});
</script>

<span
	class="loading-indicator"
	class:loading-indicator--contained={contained}
	style="--size: {size}px"
	role="progressbar"
	aria-label={label}
>
	<svg viewBox="0 0 48 48" aria-hidden="true">
		<defs>
			<linearGradient id="{uid}-g" x1="0" y1="0" x2="1" y2="1">
				{#each stops as stop, i (i)}
					<stop offset={i / (stops.length - 1)} stop-color={stop} />
				{/each}
			</linearGradient>
		</defs>
		<path {d} fill="url(#{uid}-g)" />
	</svg>
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
	.loading-indicator svg {
		width: 100%;
		height: 100%;
		animation: loading-indicator-spin 2.6s linear infinite;
	}
	.loading-indicator--contained {
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-inverse-surface);
	}
	.loading-indicator--contained svg {
		width: 62%;
		height: 62%;
	}
	@keyframes loading-indicator-spin {
		to {
			rotate: 360deg;
		}
	}
</style>
