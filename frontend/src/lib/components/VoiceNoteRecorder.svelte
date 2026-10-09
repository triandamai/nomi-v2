<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { speechLanguage } from '$lib/i18n';
	import { onMount } from 'svelte';

	// Records a voice note: the audio itself (uploaded and transcribed by the files model), plus
	// the browser's own live transcript where it can make one, shown while recording and kept as
	// a fallback. Shown inside the composer while recording.

	let {
		onsave,
		oncancel,
	}: {
		onsave: (audio: Blob, transcript: string, seconds: number) => void;
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

	/** Opus in WebM (Chrome, Firefox) or Ogg, else AAC in MP4 (Safari). */
	function audioType(): string | undefined {
		return ['audio/webm;codecs=opus', 'audio/ogg;codecs=opus', 'audio/mp4', 'audio/webm'].find((t) => MediaRecorder.isTypeSupported(t));
	}

	let transcript = $state('');
	let seconds = $state(0);
	let problem = $state<string | null>(null);
	let ready = $state(false);
	let recognition: Recognition | null = null;
	let recorder: MediaRecorder | null = null;
	let stream: MediaStream | null = null;
	let chunks: Blob[] = [];
	let started = 0;
	let finished = false;

	function release() {
		stream?.getTracks().forEach((track) => track.stop());
		stream = null;
	}

	onMount(() => {
		let timer: ReturnType<typeof setInterval> | undefined;
		(async () => {
			if (typeof MediaRecorder === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
				problem = m.voice_unsupported();
				return;
			}
			try {
				stream = await navigator.mediaDevices.getUserMedia({ audio: true });
			} catch {
				problem = m.mic_blocked();
				return;
			}
			if (finished) return release();
			recorder = new MediaRecorder(stream, { mimeType: audioType() });
			recorder.ondataavailable = (event) => {
				if (event.data.size > 0) chunks.push(event.data);
			};
			recorder.start(1000);
			started = Date.now();
			ready = true;
			timer = setInterval(() => (seconds = Math.round((Date.now() - started) / 1000)), 250);

			// The live transcript is a bonus: recording works without it.
			const w = window as unknown as { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition };
			const Ctor = w.SpeechRecognition ?? w.webkitSpeechRecognition;
			if (Ctor) {
				recognition = new Ctor();
				recognition.lang = speechLanguage();
				recognition.continuous = true;
				recognition.interimResults = true;
				recognition.onresult = (event) => {
					let text = '';
					for (let i = 0; i < event.results.length; i++) text += event.results[i][0].transcript;
					transcript = text.trim();
				};
				recognition.onerror = () => {};
				// Browsers end recognition after a pause; keep listening until the user stops.
				recognition.onend = () => {
					if (!finished) recognition?.start();
				};
				recognition.start();
			}
		})();
		return () => {
			clearInterval(timer);
			finished = true;
			recognition?.abort();
			if (recorder?.state === 'recording') recorder.stop();
			release();
		};
	});

	function stop() {
		if (!recorder) return;
		finished = true;
		recognition?.stop();
		const active = recorder;
		active.onstop = () => {
			release();
			const audio = new Blob(chunks, { type: active.mimeType || 'audio/webm' });
			if (audio.size === 0 || seconds < 1) {
				problem = m.voice_nothing();
				return;
			}
			onsave(audio, transcript, Math.max(1, seconds));
		};
		active.stop();
	}

	function discard() {
		finished = true;
		recognition?.abort();
		if (recorder?.state === 'recording') recorder.stop();
		release();
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
	<button type="button" class="voice__btn voice__btn--primary" onclick={stop} disabled={!ready || !!problem}>{m.common_done()}</button>
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
