<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import AppearancePicker from '$lib/components/AppearancePicker.svelte';
	import DrawerCustomizer from '$lib/components/DrawerCustomizer.svelte';
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
</script>

<div class="h-full overflow-y-auto px-4 py-8 md:px-10">
	<div class="max-w-3xl">
		<h1 class="md-display-small" style="color: var(--md-sys-color-on-surface)">{m.prefs_title()}</h1>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}

		<section class="mt-6">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.prefs_appearance()}</h2>
			<p class="md-body-medium mt-1 mb-4" style="color: var(--md-sys-color-on-surface-variant)">
				{m.prefs_appearance_hint()}
			</p>
			<AppearancePicker preferences={data.preferences} />
		</section>

		<section id="drawer" class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.drawer_title()}</h2>
			<p class="md-body-medium mt-1 mb-2" style="color: var(--md-sys-color-on-surface-variant)">{m.drawer_hint()}</p>
			<DrawerCustomizer savedPins={data.preferences.drawer_pins} />
		</section>

		<section class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.prefs_timezone()}</h2>
			<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
				{m.prefs_timezone_hint()}
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
