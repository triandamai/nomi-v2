<script lang="ts">
	import AgentShape from './AgentShape.svelte';
	import Checkbox from './Checkbox.svelte';
	import {
		GRADIENT_STOPS,
		GRADIENT_TONES,
		MOTION_LABELS,
		SHAPE_LABELS,
		SHAPE_MOTIONS,
		SHAPE_NAMES,
		type GradientTone,
		type ShapeMotion,
		type ShapeName,
	} from './shapes';

	// Picks an agent's look: a shape, a gradient and how it moves while working. Native radio
	// groups underneath, so it submits with a plain form (names below) and works by keyboard.

	let {
		shape = $bindable('cookie9'),
		tone = $bindable('glow'),
		motion = $bindable('spin'),
		name = 'Agent',
	}: {
		shape?: ShapeName;
		tone?: GradientTone;
		motion?: ShapeMotion;
		/** Shown under the preview. */
		name?: string;
	} = $props();

	const uid = $props.id();
	let previewWorking = $state(true);

	function swatch(t: GradientTone): string {
		const stops = GRADIENT_STOPS[t];
		return `linear-gradient(135deg, ${stops.join(', ')})`;
	}
</script>

<div class="picker">
	<div class="picker__preview">
		<AgentShape {shape} {tone} {motion} size={96} working={previewWorking} label="{name}'s look" />
		<span class="picker__name">{name || 'Agent'}</span>
		<Checkbox bind:checked={previewWorking} label="Show working" class="picker__toggle" />
	</div>

	<div class="picker__controls">
		<fieldset class="picker__group">
			<legend class="nomi-meta">Shape</legend>
			<div class="picker__shapes">
				{#each SHAPE_NAMES as option (option)}
					<label class="picker__shape" title={SHAPE_LABELS[option]}>
						<input type="radio" name="shape" value={option} bind:group={shape} class="sr-only" />
						<AgentShape shape={option} {tone} size={36} />
						<span class="picker__shape-label">{SHAPE_LABELS[option]}</span>
					</label>
				{/each}
			</div>
		</fieldset>

		<fieldset class="picker__group">
			<legend class="nomi-meta">Gradient</legend>
			<div class="picker__tones">
				{#each GRADIENT_TONES as option (option)}
					<label class="picker__tone" title={option}>
						<input type="radio" name="tone" value={option} bind:group={tone} class="sr-only" />
						<span class="picker__swatch" style="background: {swatch(option)}"></span>
						<span class="sr-only">{option}</span>
					</label>
				{/each}
			</div>
		</fieldset>

		<fieldset class="picker__group">
			<legend class="nomi-meta">Motion while working</legend>
			<div class="picker__motions" id="{uid}-motions">
				{#each SHAPE_MOTIONS as option (option)}
					<label class="picker__motion">
						<input type="radio" name="motion" value={option} bind:group={motion} class="sr-only" />
						<AgentShape {shape} {tone} motion={option} size={22} working />
						<span>{MOTION_LABELS[option]}</span>
					</label>
				{/each}
			</div>
		</fieldset>
	</div>
</div>

<style>
	.picker {
		display: grid;
		grid-template-columns: minmax(0, 180px) minmax(0, 1fr);
		gap: 20px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	@media (max-width: 640px) {
		.picker {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.picker__preview {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		align-self: start;
		padding: 24px 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
	}
	.picker__name {
		font-family: var(--md-ref-typeface-brand);
		font-weight: 700;
		font-size: 1.125rem;
		text-align: center;
		overflow-wrap: anywhere;
	}
	.picker :global(.picker__toggle) {
		font-size: 0.8125rem;
	}
	.picker__controls {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	.picker__group {
		margin: 0;
		padding: 0;
		border: none;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.picker__shapes {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
		gap: 6px;
	}
	.picker__shape {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 8px 4px 6px;
		border-radius: var(--md-sys-shape-corner-large);
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.picker__shape:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.picker__shape:has(input:checked) {
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.picker__shape:has(input:focus-visible),
	.picker__tone:has(input:focus-visible),
	.picker__motion:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.picker__shape-label {
		font-size: 0.6875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.picker__shape:has(input:checked) .picker__shape-label {
		color: inherit;
	}
	.picker__tones {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.picker__tone {
		display: inline-flex;
		padding: 3px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 2px solid transparent;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.picker__tone:has(input:checked) {
		border-color: var(--md-sys-color-on-surface);
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.picker__swatch {
		width: 32px;
		height: 32px;
		border-radius: inherit;
	}
	.picker__motions {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.picker__motion {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 40px;
		padding: 0 14px 0 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.picker__motion:has(input:checked) {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
</style>
