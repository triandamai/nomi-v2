<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { tick } from 'svelte';
	import AppearancePicker from '$lib/components/AppearancePicker.svelte';
	import DrawerCustomizer from '$lib/components/DrawerCustomizer.svelte';
	import HomeLayoutPicker from '$lib/components/HomeLayoutPicker.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import { DEFAULT_RECENTS_LIMIT, RECENTS_LIMITS, resolveRecents } from '$lib/drawerRecents';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let timezoneForm: HTMLFormElement | undefined = $state();
	let backdropForm: HTMLFormElement | undefined = $state();
	let backdropOn = $state(true);
	$effect.pre(() => {
		backdropOn = data.preferences.animated_backdrop === true;
	});
	// Recents in the drawer: the switch and the size save as soon as either changes.
	const RECENTS_LIMIT_OPTIONS = RECENTS_LIMITS.map((limit) => ({ value: String(limit), label: String(limit) }));
	let recentsForm: HTMLFormElement | undefined = $state();
	let recentsOn = $state(true);
	let recentsLimit = $state(String(DEFAULT_RECENTS_LIMIT));
	$effect.pre(() => {
		const recents = resolveRecents(data.preferences);
		recentsOn = recents.enabled;
		recentsLimit = String(recents.limit);
	});
	async function saveRecents() {
		// The hidden inputs follow the switch and the size on the next render; submit after it.
		await tick();
		recentsForm?.requestSubmit();
	}
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

		<section id="home" class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.prefs_home()}</h2>
			<p class="md-body-medium mt-1 mb-4" style="color: var(--md-sys-color-on-surface-variant)">{m.prefs_home_hint()}</p>
			<HomeLayoutPicker current={data.preferences.home_layout} />
			<form bind:this={backdropForm} method="POST" action="?/updateBackdrop" use:enhance={() => async ({ update }) => update({ reset: false })} class="backdrop-row">
				<label for="animated-backdrop" class="backdrop-row__text">
					<span class="backdrop-row__title">{m.prefs_backdrop()}</span>
					<span class="backdrop-row__hint">{m.prefs_backdrop_hint()}</span>
				</label>
				<Switch id="animated-backdrop" name="animated_backdrop" bind:checked={backdropOn} onchange={async () => {
						// The hidden input follows `checked` on the next render; submit after it.
						await tick();
						backdropForm?.requestSubmit();
					}} />
			</form>
		</section>

		<section id="drawer" class="mt-8">
			<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.drawer_title()}</h2>
			<p class="md-body-medium mt-1 mb-2" style="color: var(--md-sys-color-on-surface-variant)">{m.drawer_hint()}</p>
			<DrawerCustomizer savedPins={data.preferences.drawer_pins} />
			<form bind:this={recentsForm} method="POST" action="?/updateRecents" use:enhance={() => async ({ update }) => update({ reset: false })} class="backdrop-row recents-row">
				<div class="recents-row__top">
					<label for="drawer-recents" class="backdrop-row__text">
						<span class="backdrop-row__title">{m.recents_pref_title()}</span>
						<span class="backdrop-row__hint">{m.recents_pref_hint()}</span>
					</label>
					<Switch id="drawer-recents" name="drawer_recents" bind:checked={recentsOn} onchange={saveRecents} />
				</div>
				{#if recentsOn}
					<div class="recents-row__limit">
						<span class="backdrop-row__hint">{m.recents_pref_limit()}</span>
						<ButtonGroup options={RECENTS_LIMIT_OPTIONS} bind:value={recentsLimit} name="drawer_recents_limit" aria-label={m.recents_pref_limit()} onchange={saveRecents} />
					</div>
				{:else}
					<input type="hidden" name="drawer_recents_limit" value={recentsLimit} />
				{/if}
			</form>
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
	.backdrop-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		margin-top: 16px;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.backdrop-row__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		cursor: pointer;
	}
	.backdrop-row__title {
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.backdrop-row__hint {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.recents-row {
		flex-direction: column;
		align-items: stretch;
	}
	.recents-row__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}
	.recents-row__limit {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
	}
	.m3-timezone-select {
		height: 52px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-large);
		border: 1.5px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		max-width: 320px;
	}
</style>
