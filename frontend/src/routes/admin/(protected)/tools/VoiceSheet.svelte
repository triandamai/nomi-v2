<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Admin → Tools → speak: the ElevenLabs key, the voice the crew reads with and how it sounds.
	// Voices come from the key's account once a key is saved; "Test" reads a line with the saved
	// settings so the admin hears the result before anyone else does.
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { VoiceModel, VoiceOption, VoiceSettings } from './+page.server';

	let { open = $bindable(false), settings }: { open?: boolean; settings: VoiceSettings | undefined } = $props();

	const FORMAT_LABELS: Record<string, string> = {
		mp3_22050_32: 'MP3 22 kHz · 32 kbps',
		mp3_44100_64: 'MP3 44.1 kHz · 64 kbps',
		mp3_44100_96: 'MP3 44.1 kHz · 96 kbps',
		mp3_44100_128: 'MP3 44.1 kHz · 128 kbps',
		mp3_44100_192: 'MP3 44.1 kHz · 192 kbps',
	};

	let apiKey = $state('');
	let clearKey = $state(false);
	let voiceId = $state('');
	let voiceName = $state('');
	let modelId = $state('');
	let stability = $state(0.5);
	let similarity = $state(0.75);
	let style = $state(0);
	let speed = $state(1);
	let speakerBoost = $state(true);
	let outputFormat = $state('mp3_44100_128');
	let maxChars = $state('2500');

	let saving = $state(false);
	let saved = $state(false);
	let saveError = $state<string | null>(null);

	let voices = $state<VoiceOption[] | null>(null);
	let models = $state<VoiceModel[] | null>(null);
	let voicesLoading = $state(false);
	let voicesError = $state<string | null>(null);
	let voiceQuery = $state('');

	let testText = $state('');
	let testing = $state(false);
	let testError = $state<string | null>(null);
	let testUrl = $state<string | null>(null);

	let previewing = $state<string | null>(null);
	let previewAudio: HTMLAudioElement | null = null;

	const hasKey = $derived((settings?.key_source ?? 'missing') !== 'missing');
	const modelOptions = $derived((models ?? settings?.models ?? []).map((model) => ({ value: model.model_id, label: model.name })));
	const shownVoices = $derived.by(() => {
		const query = voiceQuery.trim().toLowerCase();
		const all = voices ?? [];
		if (!query) return all;
		return all.filter((v) => `${v.name} ${v.description ?? ''} ${v.category ?? ''}`.toLowerCase().includes(query));
	});

	// Fill the form from what's saved each time the sheet opens.
	let wasOpen = false;
	$effect(() => {
		if (open && !wasOpen && settings) {
			apiKey = '';
			clearKey = false;
			voiceId = settings.voice_id;
			voiceName = settings.voice_name ?? '';
			modelId = settings.model_id;
			stability = settings.stability;
			similarity = settings.similarity_boost;
			style = settings.style;
			speed = settings.speed;
			speakerBoost = settings.use_speaker_boost;
			outputFormat = settings.output_format;
			maxChars = String(settings.max_chars);
			saved = false;
			saveError = null;
			testError = null;
			if (hasKey && !voices) void loadVoices();
		}
		if (!open && wasOpen) stopPreview();
		wasOpen = open;
	});

	async function loadVoices() {
		voicesLoading = true;
		voicesError = null;
		try {
			const response = await fetch('/admin/tools/speak/voices');
			const body = await response.json();
			if (!response.ok) throw new Error(body?.error || m.tools_voice_list_failed());
			voices = body.voices as VoiceOption[];
			models = (body.models as VoiceModel[]).length ? (body.models as VoiceModel[]) : null;
		} catch (error) {
			voicesError = error instanceof Error && error.message ? error.message : m.tools_voice_list_failed();
		}
		voicesLoading = false;
	}

	function pick(voice: VoiceOption) {
		voiceId = voice.voice_id;
		voiceName = voice.name;
	}

	function stopPreview() {
		previewAudio?.pause();
		previewAudio = null;
		previewing = null;
	}

	function preview(voice: VoiceOption) {
		if (previewing === voice.voice_id) return stopPreview();
		stopPreview();
		if (!voice.preview_url) return;
		const audio = new Audio(voice.preview_url);
		audio.onended = () => stopPreview();
		previewAudio = audio;
		previewing = voice.voice_id;
		audio.play().catch(() => stopPreview());
	}

	async function runTest() {
		testing = true;
		testError = null;
		try {
			const response = await fetch('/admin/tools/speak/test', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ text: testText.trim() || null }),
			});
			if (!response.ok) {
				const body = await response.json().catch(() => null);
				throw new Error(body?.error || m.tools_voice_test_failed());
			}
			if (testUrl) URL.revokeObjectURL(testUrl);
			testUrl = URL.createObjectURL(await response.blob());
		} catch (error) {
			testError = error instanceof Error && error.message ? error.message : m.tools_voice_test_failed();
		}
		testing = false;
	}

	function keyPlaceholder() {
		if (settings?.key_source === 'saved') return m.tools_key_saved();
		if (settings?.key_source === 'env') return m.tools_key_env({ env: settings.env_var });
		return m.tools_key_missing();
	}
</script>

{#snippet slider(id: string, label: string, hint: string, min: number, max: number, step: number, value: number, set: (v: number) => void, shown: string)}
	<div class="slider">
		<div class="slider__head">
			<label for={id}>{label}</label>
			<output for={id}>{shown}</output>
		</div>
		<input
			{id}
			name={id}
			type="range"
			{min}
			{max}
			{step}
			{value}
			style="--fill: {((value - min) / (max - min)) * 100}%"
			oninput={(event) => set(Number(event.currentTarget.value))}
		/>
		<p class="slider__hint">{hint}</p>
	</div>
{/snippet}

<BottomSheet bind:open>
	<h2 class="sheet-title">{m.tools_voice_title()}</h2>
	<p class="lede">{m.tools_voice_lede()}</p>
	{#if saveError}
		<p class="md-body-medium mt-2" role="alert" style="color: var(--md-sys-color-error)">{saveError}</p>
	{/if}
	<form
		method="POST"
		action="?/saveVoice"
		class="mt-4 flex flex-col gap-4"
		use:enhance={() => {
			saving = true;
			saved = false;
			saveError = null;
			const keyChanged = apiKey.trim().length > 0 || clearKey;
			return async ({ result }) => {
				saving = false;
				if (result.type === 'success') {
					await invalidateAll();
					apiKey = '';
					clearKey = false;
					saved = true;
					if (keyChanged) {
						voices = null;
						models = null;
						if (settings?.key_source !== 'missing') void loadVoices();
					}
				} else if (result.type === 'failure') {
					saveError = (result.data?.error as string) || m.err_save_tool();
				}
			};
		}}
	>
		<TextField
			id="voice_api_key"
			name="api_key"
			type="password"
			autocomplete="off"
			label={m.tools_voice_key()}
			bind:value={apiKey}
			placeholder={keyPlaceholder()}
			supportingText={m.tools_voice_key_hint()}
		/>
		{#if settings?.key_source === 'saved'}
			<label class="check">
				<input type="checkbox" bind:checked={clearKey} />
				<span>{m.tools_clear_key()}</span>
			</label>
		{/if}
		<input type="hidden" name="clear_api_key" value={String(clearKey)} />

		<fieldset class="voices">
			<legend class="field-label">{m.tools_voice_voice()}</legend>
			<input type="hidden" name="voice_id" value={voiceId} />
			<input type="hidden" name="voice_name" value={voiceName} />
			{#if !hasKey}
				<p class="hint">{m.tools_voice_needs_key_first()}</p>
			{:else if voicesLoading}
				<p class="hint">{m.tools_voice_loading()}</p>
			{:else if voicesError}
				<p class="hint hint--error" role="alert">{voicesError}</p>
				<Button type="button" variant="text" size="s" onclick={loadVoices}>{m.tools_voice_retry()}</Button>
			{:else if voices}
				{#if voices.length > 8}
					<TextField id="voice_search" label={m.tools_voice_search()} bind:value={voiceQuery} />
				{/if}
				<ul class="voice-list">
					{#each shownVoices as voice (voice.voice_id)}
						{@const selected = voice.voice_id === voiceId}
						<li class="voice-row" class:voice-row--selected={selected}>
							<label class="voice-row__pick">
								<input type="radio" name="voice_choice" value={voice.voice_id} checked={selected} onchange={() => pick(voice)} />
								<span class="voice-row__text">
									<span class="voice-row__name">{voice.name}</span>
									{#if voice.description || voice.category}
										<span class="voice-row__meta">{voice.description ?? voice.category}</span>
									{/if}
								</span>
							</label>
							{#if voice.preview_url}
								<button
									type="button"
									class="voice-row__preview"
									aria-label={previewing === voice.voice_id ? m.voice_pause() : m.tools_voice_preview({ voice: voice.name })}
									onclick={() => preview(voice)}
								>
									{#if previewing === voice.voice_id}
										<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1.2" /><rect x="14" y="5" width="4" height="14" rx="1.2" /></svg>
									{:else}
										<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5.6v12.8a1 1 0 0 0 1.5.9l10.2-6.4a1 1 0 0 0 0-1.8L9.5 4.7A1 1 0 0 0 8 5.6Z" /></svg>
									{/if}
								</button>
							{/if}
						</li>
					{:else}
						<li class="hint">{m.list_no_match({ query: voiceQuery })}</li>
					{/each}
				</ul>
			{/if}
			{#if !voices}
				<p class="hint">{m.tools_voice_current({ voice: voiceName || voiceId })}</p>
			{/if}
		</fieldset>

		<Select label={m.tools_voice_model()} name="model_id" bind:value={modelId} options={modelOptions} />

		{@render slider('stability', m.tools_voice_stability(), m.tools_voice_stability_hint(), 0, 1, 0.05, stability, (v) => (stability = v), stability.toFixed(2))}
		{@render slider('similarity_boost', m.tools_voice_similarity(), m.tools_voice_similarity_hint(), 0, 1, 0.05, similarity, (v) => (similarity = v), similarity.toFixed(2))}
		{@render slider('style', m.tools_voice_style(), m.tools_voice_style_hint(), 0, 1, 0.05, style, (v) => (style = v), style.toFixed(2))}
		{@render slider('speed', m.tools_voice_speed(), m.tools_voice_speed_hint(), 0.7, 1.2, 0.05, speed, (v) => (speed = v), `${speed.toFixed(2)}×`)}

		<div class="switch-row">
			<span class="switch-row__text">
				<span>{m.tools_voice_boost()}</span>
				<span class="hint">{m.tools_voice_boost_hint()}</span>
			</span>
			<Switch checked={speakerBoost} aria-label={m.tools_voice_boost()} onchange={(value) => (speakerBoost = value)} />
			<input type="hidden" name="use_speaker_boost" value={String(speakerBoost)} />
		</div>

		<Select
			label={m.tools_voice_format()}
			name="output_format"
			bind:value={outputFormat}
			options={(settings?.output_formats ?? []).map((f) => ({ value: f, label: FORMAT_LABELS[f] ?? f }))}
		/>
		<TextField
			id="voice_max_chars"
			name="max_chars"
			type="number"
			min="100"
			max={settings?.max_chars_limit ?? 5000}
			step="100"
			label={m.tools_voice_max_chars()}
			supportingText={m.tools_voice_max_chars_hint({ max: String(settings?.max_chars_limit ?? 5000) })}
			bind:value={maxChars}
		/>

		<div class="flex flex-wrap gap-2 pt-2">
			<Button type="submit" variant="filled" disabled={saving}>{m.common_save()}</Button>
			<Button type="button" variant="outlined" onclick={() => (open = false)}>{m.common_cancel()}</Button>
			{#if saved}<span class="saved" role="status">{m.tools_saved()}</span>{/if}
		</div>
	</form>

	<div class="test">
		<TextField id="voice_test_text" label={m.tools_voice_test_text()} bind:value={testText} supportingText={m.tools_voice_test_hint()} />
		<Button type="button" variant="tonal" class="w-fit" disabled={testing || !hasKey} onclick={runTest}>
			{testing ? m.tools_voice_testing() : m.tools_voice_test()}
		</Button>
		{#if testError}
			<p class="test__error" role="alert">{testError}</p>
		{/if}
		{#if testUrl}
			<!-- svelte-ignore a11y_media_has_caption -->
			<audio class="test__audio" controls autoplay src={testUrl}></audio>
		{/if}
	</div>
</BottomSheet>

<style>
	.sheet-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.lede {
		margin: 6px 0 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.hint--error {
		color: var(--md-sys-color-error);
	}
	.check {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.field-label {
		margin-bottom: 6px;
		padding: 0;
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.voices {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: 0;
		min-width: 0;
	}
	.voice-list {
		display: flex;
		flex-direction: column;
		max-height: 280px;
		margin: 0;
		padding: 0;
		overflow-y: auto;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		list-style: none;
	}
	.voice-row {
		display: flex;
		align-items: center;
		gap: 4px;
		padding-right: 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.voice-row:last-child {
		border-bottom: 0;
	}
	.voice-row--selected {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.voice-row__pick {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 12px;
		min-width: 0;
		min-height: 48px;
		padding: 8px 12px;
		cursor: pointer;
	}
	.voice-row__pick:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.voice-row__pick input {
		flex: none;
		accent-color: var(--md-sys-color-primary);
	}
	.voice-row__pick:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.voice-row__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.voice-row__name {
		font-weight: 600;
	}
	.voice-row__meta {
		overflow: hidden;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.voice-row__preview {
		display: grid;
		flex: none;
		place-items: center;
		width: 44px;
		height: 44px;
		border: 0;
		border-radius: 50%;
		background: transparent;
		color: var(--md-sys-color-primary);
		cursor: pointer;
	}
	.voice-row__preview:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 10%, transparent);
	}
	.voice-row__preview:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.slider {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.slider__head {
		display: flex;
		justify-content: space-between;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}
	.slider__head output {
		font-variant-numeric: tabular-nums;
		color: var(--md-sys-color-on-surface-variant);
	}
	.slider input {
		width: 100%;
		height: 4px;
		margin: 10px 0;
		border-radius: 2px;
		background: linear-gradient(
			to right,
			var(--md-sys-color-primary) var(--fill),
			var(--md-sys-color-surface-container-highest) var(--fill)
		);
		cursor: pointer;
		appearance: none;
	}
	.slider input::-webkit-slider-thumb {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--md-sys-color-primary);
		appearance: none;
	}
	.slider input::-moz-range-thumb {
		width: 20px;
		height: 20px;
		border: 0;
		border-radius: 50%;
		background: var(--md-sys-color-primary);
	}
	.slider input:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 6px;
	}
	.slider__hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.switch-row {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	.switch-row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}
	.saved {
		align-self: center;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-primary);
	}
	.test {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: 24px;
		padding-top: 20px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.test__error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.test__audio {
		width: 100%;
	}
</style>
