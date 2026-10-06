<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { speechLanguage } from '$lib/i18n';
	import { onMount } from 'svelte';

	// Records a voice note: the browser transcribes speech as the user talks, and the transcript
	// becomes a "voice" attachment (the Files agent reads it and decides what to do). Shown inside
	// the composer while recording.

	let {
		onsave,
		oncancel,
	}: {
		onsave: (transcript: string, seconds: number) => void;
		oncancel: () => void;
	} = $props();

	type Recognition = {
		lang: string;
		continuous: boolean;
		interimResults: boolean;
		start: () => void;
		stop: () => void;
		abort: () => void;
		onresult: ((event: { results: ArrayLike<ArrayLike<{ transcript: string }> & { isFinal: boolean }> }) => void) | null;
		onend: (() => void) | null;
		onerror: ((event: { error: string }) => void) | null;
	};

	let transcript = $state('');
	let seconds = $state(0);
	let problem = $state<string | null>(null);
	let recognition: Recognition | null = null;
	let started = 0;
	let saving = false;

	onMount(() => {
		const w = window as unknown as { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition };
		const Ctor = w.SpeechRecognition ?? w.webkitSpeechRecognition;
		if (!Ctor) {
			problem = m.voice_unsupported();
			return;
		}
		recognition = new Ctor();
		recognition.lang = speechLanguage();
		recognition.continuous = true;
		recognition.interimResults = true;
		recognition.onresult = (event) => {
			let text = '';
			for (let i = 0; i < event.results.length; i++) text += event.results[i][0].transcript;
			transcript = text.trim();
		};
		recognition.onerror = (event) => {
			problem = event.error === 'not-allowed' ? m.mic_blocked() : event.error === 'no-speech' ? null : m.voice_stopped();
		};
		// Browsers end recognition after a pause; keep listening until the user stops.
		recognition.onend = () => {
			if (!saving && !problem) recognition?.start();
		};
		started = Date.now();
		recognition.start();
		const timer = setInterval(() => (seconds = Math.round((Date.now() - started) / 1000)), 250);
		return () => {
			clearInterval(timer);
			saving = true;
			recognition?.abort();
		};
	});

	function stop() {
		saving = true;
		recognition?.stop();
		if (transcript) onsave(transcript, seconds);
		else problem = m.voice_nothing();
	}

	function discard() {
		saving = true;
		recognition?.abort();
		oncancel();
	}

	const clock = $derived(`${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`);
</script>

<div class="voice" role="group" aria-label={m.voice_recording()}>
	<span class="voice__dot" aria-hidden="true"></span>
	<span class="voice__clock">{clock}</span>
	<span class="voice__text" aria-live="polite">
		{#if problem}
			<span class="voice__problem">{problem}</span>
		{:else if transcript}
			{transcript}
		{:else}
			<span class="voice__hint">{m.voice_listening()}</span>
		{/if}
	</span>
	<button type="button" class="voice__btn" onclick={discard}>{m.voice_discard()}</button>
	<button type="button" class="voice__btn voice__btn--primary" onclick={stop} disabled={!!problem && !transcript}>{m.common_done()}</button>
</div>

<style>
	.voice {
		flex-basis: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		padding: 4px 4px 4px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.voice__dot {
		flex: none;
		width: 10px;
		height: 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-error);
		animation: voice-pulse 1.2s ease-in-out infinite;
	}
	@keyframes voice-pulse {
		50% {
			scale: 1.5;
			opacity: 0.6;
		}
	}
	.voice__clock {
		flex: none;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.voice__text {
		flex: 1;
		min-width: 0;
		font-size: 0.9375rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.voice__hint,
	.voice__problem {
		opacity: 0.8;
	}
	.voice__btn {
		flex: none;
		height: 40px;
		padding: 0 14px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: inherit;
		font: inherit;
		font-size: 0.875rem;
		font-weight: 650;
		cursor: pointer;
	}
	.voice__btn--primary {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
	.voice__btn:disabled {
		opacity: 0.5;
		cursor: default;
	}
	@media (prefers-reduced-motion: reduce) {
		.voice__dot {
			animation: none;
		}
	}
</style>
