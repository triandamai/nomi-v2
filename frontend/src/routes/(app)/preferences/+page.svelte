<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const THEME_OPTIONS = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' },
	];

	// Swatch previews use each option's own light-scheme primary color directly, not the active
	// CSS variables — the point is to show what every option looks like, including the ones not
	// currently selected.
	const ACCENT_COLOR_OPTIONS = [
		{ value: 'green', label: 'Green', swatch: '#006e2a' },
		{ value: 'blue', label: 'Blue', swatch: '#0052dc' },
		{ value: 'purple', label: 'Purple', swatch: '#6920ff' },
		{ value: 'pink', label: 'Pink', swatch: '#ba005b' },
		{ value: 'orange', label: 'Orange', swatch: '#9f4200' },
		{ value: 'teal', label: 'Teal', swatch: '#006b5c' },
	];
</script>

<div class="h-full overflow-y-auto p-8">
	<div class="max-w-lg">
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Preferences</h1>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Theme</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Choose how Nomi looks. "System" follows your device's setting.
			</p>
			<form method="POST" action="?/updateTheme" use:enhance class="mt-3 flex gap-2">
				{#each THEME_OPTIONS as option (option.value)}
					<button
						type="submit"
						name="theme"
						value={option.value}
						class="m3-theme-option"
						class:m3-theme-option--active={data.preferences.theme === option.value}
					>
						{option.label}
					</button>
				{/each}
			</form>
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Color</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Pick an accent color for Nomi.
			</p>
			<form method="POST" action="?/updateAccentColor" use:enhance class="mt-3 flex flex-wrap gap-3">
				{#each ACCENT_COLOR_OPTIONS as option (option.value)}
					<button
						type="submit"
						name="accent_color"
						value={option.value}
						class="m3-color-option"
						class:m3-color-option--active={data.preferences.accent_color === option.value}
						aria-label={option.label}
						aria-pressed={data.preferences.accent_color === option.value}
					>
						<span class="m3-color-option__swatch" style="background: {option.swatch}"></span>
						<span class="md-label-medium">{option.label}</span>
					</button>
				{/each}
			</form>
		</section>
	</div>
</div>

<style>
	.m3-theme-option {
		padding: 8px 20px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
	}
	.m3-theme-option--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		border-color: transparent;
	}

	.m3-color-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 8px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 2px solid transparent;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.m3-color-option:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-color-option--active {
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-surface);
	}
	.m3-color-option__swatch {
		display: block;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-full);
		box-shadow: inset 0 0 0 1px color-mix(in srgb, black 15%, transparent);
	}
</style>
