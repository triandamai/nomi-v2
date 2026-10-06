<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { MODES, THEMES } from '$lib/appearance';
	import type { AccentColor, Preferences, Theme } from '$lib/types';

	// Light/dark mode and the colour theme. Used on Preferences and in the admin console: both save
	// through the Preferences page's actions, so the choice follows the person everywhere. A pick
	// shows at once; the save follows.
	let { preferences }: { preferences: Pick<Preferences, 'theme' | 'accent_color'> } = $props();

	let mode = $state<Theme>('system');
	let theme = $state<AccentColor>('canopy');
	$effect.pre(() => {
		mode = preferences.theme;
		theme = preferences.accent_color;
	});

	function show(nextMode: Theme, nextTheme: AccentColor) {
		document.documentElement.dataset.theme = nextMode;
		document.documentElement.dataset.color = nextTheme;
	}

	const keep = () => async ({ update }: { update: (opts?: { reset?: boolean }) => Promise<void> }) => {
		await update({ reset: false });
	};
</script>

<div class="appearance">
	<section aria-labelledby="mode-title">
		<h3 id="mode-title" class="appearance__label">{m.appearance_mode()}</h3>
		<form method="POST" action="/preferences?/updateTheme" use:enhance={keep} class="modes">
			{#each MODES as option (option.value)}
				<button
					type="submit"
					name="theme"
					value={option.value}
					class="modes__item"
					class:modes__item--selected={mode === option.value}
					aria-pressed={mode === option.value}
					onclick={() => {
						mode = option.value;
						show(option.value, theme);
					}}
				>
					{option.label}
				</button>
			{/each}
		</form>
		<p class="appearance__hint">{m.appearance_mode_hint()}</p>
	</section>

	<section aria-labelledby="theme-title">
		<h3 id="theme-title" class="appearance__label">{m.appearance_theme()}</h3>
		<form method="POST" action="/preferences?/updateAccentColor" use:enhance={keep} class="themes">
			{#each THEMES as option (option.value)}
				{@const selected = theme === option.value}
				<button
					type="submit"
					name="accent_color"
					value={option.value}
					class="theme"
					class:theme--selected={selected}
					aria-pressed={selected}
					onclick={() => {
						theme = option.value;
						show(mode, option.value);
					}}
				>
					<span class="theme__preview" aria-hidden="true">
						{#each [option.light, option.dark] as swatch, i (i)}
							<span class="theme__half" style="background: {swatch.surface}">
								<span class="theme__card" style="background: {swatch.surfacecontainerlowest}">
									<span class="theme__line" style="background: {swatch.onsurface}"></span>
									<span class="theme__line theme__line--short" style="background: {swatch.onsurface}"></span>
									<span class="theme__row">
										<span class="theme__pill" style="background: {swatch.primary}"></span>
										<span class="theme__dot" style="background: {swatch.tertiary}"></span>
									</span>
								</span>
								<span class="theme__chip" style="background: {swatch.primarycontainer}"></span>
							</span>
						{/each}
						{#if selected}
							<span class="theme__check">
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 5 5 9-10" /></svg>
							</span>
						{/if}
					</span>
					<span class="theme__name">{option.label}</span>
					<span class="theme__description">{option.description}</span>
				</button>
			{/each}
		</form>
	</section>
</div>

<style>
	.appearance {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}
	.appearance__label {
		margin: 0 4px 10px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		font-weight: 400;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.appearance__hint {
		margin: 8px 4px 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	/* A connected button group, kept as submit buttons so a pick saves without JavaScript. */
	.modes {
		display: flex;
		gap: 2px;
		max-width: 360px;
	}
	.modes__item {
		flex: 1;
		height: 48px;
		padding: 0 18px;
		border: none;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast),
			color var(--nomi-motion-effects-fast);
	}
	.modes__item:first-child {
		border-radius: 24px 8px 8px 24px;
	}
	.modes__item:last-child {
		border-radius: 8px 24px 24px 8px;
	}
	.modes__item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, var(--md-sys-color-surface-container-high));
	}
	.modes .modes__item--selected {
		border-radius: 24px;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}

	.themes {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 168px), 1fr));
		gap: 12px;
	}
	.theme {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
		padding: 10px 10px 14px;
		border: 2px solid transparent;
		border-radius: 24px;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			border-color var(--nomi-motion-effects-fast);
	}
	.theme:hover {
		border-radius: var(--md-sys-shape-corner-large);
	}
	.theme--selected {
		border-color: var(--md-sys-color-primary);
	}
	.theme__preview {
		position: relative;
		display: flex;
		width: 100%;
		height: 92px;
		margin-bottom: 6px;
		border-radius: 16px;
		overflow: hidden;
	}
	.theme__half {
		position: relative;
		flex: 1;
		padding: 10px 8px;
	}
	.theme__card {
		display: flex;
		flex-direction: column;
		gap: 5px;
		padding: 8px;
		border-radius: 10px;
	}
	.theme__line {
		height: 4px;
		width: 80%;
		border-radius: 2px;
		opacity: 0.75;
	}
	.theme__line--short {
		width: 50%;
		opacity: 0.4;
	}
	.theme__row {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 2px;
	}
	.theme__pill {
		width: 28px;
		height: 10px;
		border-radius: 5px;
	}
	.theme__dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
	}
	.theme__chip {
		position: absolute;
		right: 8px;
		bottom: 8px;
		width: 18px;
		height: 8px;
		border-radius: 4px;
	}
	.theme__check {
		position: absolute;
		top: 6px;
		right: 6px;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.theme__name {
		padding: 0 4px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.theme__description {
		padding: 0 4px;
		font-size: 0.8125rem;
		line-height: 1.4;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
