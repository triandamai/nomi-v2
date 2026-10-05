<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let timezoneForm: HTMLFormElement | undefined = $state();
	let timezoneInput: HTMLInputElement | undefined = $state();
	let detectedTimezone = $state('');
	// Guards the auto-save to exactly one attempt per page visit — an $effect (not onMount)
	// so it naturally waits for `timezoneForm`'s bind:this to actually be populated instead of
	// assuming mount-order timing, and re-fires if `data.preferences` updates before the guard
	// trips (e.g. a client-side navigation that reuses this component instance).
	let hasAttemptedAutoSave = false;

	// Sets the hidden input's value directly on the DOM element rather than only updating the
	// `detectedTimezone` $state that its `value` attribute binds to: Svelte flushes reactive DOM
	// updates asynchronously (next microtask), but requestSubmit() reads the input's current DOM
	// value synchronously — calling it right after a state assignment races that flush and can
	// submit the *previous* value. Setting `.value` imperatively first guarantees the submitted
	// value is correct regardless of Svelte's render scheduling.
	function submitTimezone(tz: string) {
		detectedTimezone = tz;
		if (timezoneInput) timezoneInput.value = tz;
		timezoneForm?.requestSubmit();
	}

	$effect(() => {
		if (hasAttemptedAutoSave || !timezoneForm) return;
		const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
		if (!data.preferences.has_stored_timezone && tz) {
			hasAttemptedAutoSave = true;
			submitTimezone(tz);
		}
	});

	const THEME_OPTIONS = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' },
	];

	// Swatch previews use each option's own light-scheme primary color directly, not the active
	// CSS variables — the point is to show what every option looks like, including the ones not
	// currently selected.
	const ACCENT_COLOR_OPTIONS = [
		{ value: 'green', label: 'Green', swatch: '#0b6b4a' },
		{ value: 'blue', label: 'Blue', swatch: '#0052dc' },
		{ value: 'purple', label: 'Purple', swatch: '#6920ff' },
		{ value: 'pink', label: 'Pink', swatch: '#ba005b' },
		{ value: 'orange', label: 'Orange', swatch: '#9f4200' },
		{ value: 'teal', label: 'Teal', swatch: '#006b5c' },
	];
</script>

<div class="h-full overflow-y-auto px-4 py-8 md:px-10">
	<div class="max-w-lg">
		<h1 class="md-display-small" style="color: var(--md-sys-color-on-surface)">Preferences</h1>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Theme</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Choose how Nomi looks. "System" follows your device's setting.
			</p>
			<form method="POST" action="?/updateTheme" use:enhance class="mt-3 flex gap-[2px]">
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

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Timezone</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				Used to schedule reminders at the right local time.
			</p>
			<form
				bind:this={timezoneForm}
				method="POST"
				action="?/updateTimezone"
				use:enhance
				class="mt-3"
			>
				<input bind:this={timezoneInput} type="hidden" name="timezone" value={detectedTimezone || data.preferences.timezone} />
				<select
					class="m3-timezone-select"
					value={data.preferences.timezone}
					onchange={(e) => submitTimezone(e.currentTarget.value)}
				>
					<!-- Intl.supportedValuesOf('timeZone') never includes the literal string "UTC" (browsers
					     canonicalize it away), but that's exactly what this app's backend defaults a fresh
					     user's timezone to — without this explicit option, that default (and any user whose
					     OS timezone genuinely is UTC) would never show as selected in this dropdown. -->
					<option value="UTC">UTC</option>
					{#each Intl.supportedValuesOf('timeZone') as tz (tz)}
						<option value={tz}>{tz}</option>
					{/each}
				</select>
			</form>
		</section>
	</div>
</div>

<style>
	/* Rendered as an M3 Expressive connected button group (see m3/ButtonGroup.svelte) — kept as
	   real submit buttons here so each choice still saves without JavaScript. */
	.m3-theme-option {
		height: 48px;
		padding: 0 22px;
		border: none;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.m3-theme-option:first-child {
		border-radius: var(--md-sys-shape-corner-full) var(--md-sys-shape-corner-small) var(--md-sys-shape-corner-small)
			var(--md-sys-shape-corner-full);
	}
	.m3-theme-option:last-child {
		border-radius: var(--md-sys-shape-corner-small) var(--md-sys-shape-corner-full) var(--md-sys-shape-corner-full)
			var(--md-sys-shape-corner-small);
	}
	.m3-theme-option.m3-theme-option--active {
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}

	.m3-color-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 8px;
		border-radius: var(--md-sys-shape-corner-large);
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
		width: 44px;
		height: 44px;
		border-radius: var(--md-sys-shape-corner-full);
		box-shadow: inset 0 0 0 1px color-mix(in srgb, black 15%, transparent);
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	/* Selected swatch morphs circle → rounded square: shape, not just a ring, marks the pick. */
	.m3-color-option--active .m3-color-option__swatch {
		border-radius: var(--md-sys-shape-corner-medium);
	}

	.m3-timezone-select {
		height: 52px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-large);
		border: 1.5px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		max-width: 320px;
	}
</style>
