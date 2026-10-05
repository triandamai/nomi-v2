<script lang="ts">
	import { GRADIENT_STOPS, type GradientTone } from './shapes';

	// M3 Expressive wavy linear progress: the completed part is a sine wave in the agent's
	// gradient, the remainder a flat track, ending in a stop dot.

	let {
		value,
		tone = 'glow',
		label,
	}: {
		/** 0..1 */
		value: number;
		tone?: GradientTone;
		label: string;
	} = $props();

	const uid = $props.id();
	const WIDTH = 300;
	const START = 4;
	const END = 292;
	const WAVELENGTH = 15;

	const stops = $derived(GRADIENT_STOPS[tone]);
	const clamped = $derived(Math.max(0, Math.min(1, value)));
	const waveEnd = $derived(START + (END - START) * clamped);

	const wavePath = $derived.by(() => {
		const halfWaves = Math.floor((waveEnd - START) / WAVELENGTH);
		if (halfWaves < 1) return '';
		let d = `M${START} 10 q${WAVELENGTH / 2} -7 ${WAVELENGTH} 0`;
		for (let i = 1; i < halfWaves; i++) d += ` t${WAVELENGTH} 0`;
		return d;
	});
	const trackStart = $derived(START + Math.floor((waveEnd - START) / WAVELENGTH) * WAVELENGTH + 8);
</script>

<svg
	class="wavy-progress"
	viewBox="0 0 {WIDTH} 20"
	preserveAspectRatio="none"
	role="progressbar"
	aria-label={label}
	aria-valuemin={0}
	aria-valuemax={100}
	aria-valuenow={Math.round(clamped * 100)}
>
	<defs>
		<linearGradient id="{uid}-g" x1="0" y1="0" x2="1" y2="0">
			{#each stops as stop, i (i)}
				<stop offset={i / (stops.length - 1)} stop-color={stop} />
			{/each}
		</linearGradient>
	</defs>
	{#if wavePath}
		<path d={wavePath} fill="none" stroke="url(#{uid}-g)" stroke-width="5" stroke-linecap="round" vector-effect="non-scaling-stroke" />
	{/if}
	{#if trackStart < END}
		<path d="M{trackStart} 10 H{END}" class="wavy-progress__track" stroke-width="5" stroke-linecap="round" vector-effect="non-scaling-stroke" />
	{/if}
	<circle cx={END + 4} cy="10" r="2.5" class="wavy-progress__stop" />
</svg>

<style>
	.wavy-progress {
		display: block;
		width: 100%;
		height: 20px;
	}
	.wavy-progress__track {
		stroke: var(--md-sys-color-surface-container-highest);
	}
	.wavy-progress__stop {
		fill: var(--md-sys-color-primary);
	}
</style>
