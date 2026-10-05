<script lang="ts">
	import { onMount } from 'svelte';
	import { GRADIENT_STOPS, pointsToPath, shapePath, shapePoints, type GradientTone, type Point, type ShapeName } from './shapes';

	// A gradient shape that morphs through a sequence of M3 Expressive shapes on a spring.
	// Interpolated per animation frame in JS (every shape shares one point count — see shapes.ts)
	// rather than CSS `d` keyframes, which aren't animatable in every browser. Holds the first
	// shape still under prefers-reduced-motion.

	let {
		size = 48,
		tone = 'glow',
		sequence = ['cookie9', 'clover4', 'sunny8', 'flower5'],
		stepMs = 850,
		face = false,
		class: extraClass = '',
	}: {
		size?: number;
		tone?: GradientTone;
		sequence?: ShapeName[];
		/** Time spent on each shape, morph included. */
		stepMs?: number;
		/** Nomi's two-dot face. */
		face?: boolean;
		class?: string;
	} = $props();

	const uid = $props.id();
	// Until the first animation frame (and always under reduced motion) show the first shape.
	let animated = $state<string | null>(null);
	const d = $derived(animated ?? shapePath(sequence[0]));
	const stops = $derived(GRADIENT_STOPS[tone]);

	// Overshooting ease (same feel as --nomi-motion-spatial-default) — the "spring" in the morph.
	function spring(t: number): number {
		const c = 1.70158 * 1.2;
		return 1 + (c + 1) * Math.pow(t - 1, 3) + c * Math.pow(t - 1, 2);
	}

	function lerp(a: Point[], b: Point[], t: number): Point[] {
		return a.map(([ax, ay], i) => [ax + (b[i][0] - ax) * t, ay + (b[i][1] - ay) * t]);
	}

	onMount(() => {
		if (sequence.length < 2 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
		let frame = 0;
		const start = performance.now();
		const tick = (now: number) => {
			const elapsed = now - start;
			const step = Math.floor(elapsed / stepMs);
			const t = (elapsed % stepMs) / stepMs;
			const from = shapePoints(sequence[step % sequence.length]);
			const to = shapePoints(sequence[(step + 1) % sequence.length]);
			// The morph takes the first ~60% of each step; the shape then rests.
			animated = pointsToPath(lerp(from, to, spring(Math.min(1, t * 1.6))));
			frame = requestAnimationFrame(tick);
		};
		frame = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(frame);
	});
</script>

<svg width={size} height={size} viewBox="0 0 48 48" class={extraClass} aria-hidden="true">
	<defs>
		<linearGradient id="{uid}-g" x1="0" y1="0" x2="1" y2="1">
			{#each stops as stop, i (i)}
				<stop offset={i / (stops.length - 1)} stop-color={stop} />
			{/each}
		</linearGradient>
	</defs>
	<path {d} fill="url(#{uid}-g)" />
	{#if face}
		<circle cx="19" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
		<circle cx="29" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
	{/if}
</svg>
