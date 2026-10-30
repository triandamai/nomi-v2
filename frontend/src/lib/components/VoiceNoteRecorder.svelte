<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { speechLanguage } from '$lib/i18n';
	import { onMount } from 'svelte';
	import VoiceRecorder from '$lib/components/ds/chat/VoiceRecorder.svelte';

	// Records a voice note in place of the composer (drawn by the VoiceRecorder twin): the audio
	// itself, its input level as it comes in, and the browser's own live transcript where it can
	// make one. The page puts the transcript in the message to edit, or, with none, sends the
	// audio for the files model to transcribe.

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

	/** How many level bars the recorder shows, and how often a new one comes in. */
	const BARS = 48;
	const LEVEL_EVERY_MS = 100;
	/** The newest bars, still settling, drawn dimmer. */
	const FADING_BARS = 3;
	/** An RMS this loud fills a bar; speech rarely goes past it. */
	const FULL_LEVEL = 0.25;

	/** Opus in WebM (Chrome, Firefox) or Ogg, else AAC in MP4 (Safari). */
	function audioType(): string | undefined {
		return ['audio/webm;codecs=opus', 'audio/ogg;codecs=opus', 'audio/mp4', 'audio/webm'].find((t) => MediaRecorder.isTypeSupported(t));
	}

	let transcript = $state('');
	let seconds = $state(0);
	let problem = $state<string | null>(null);
	let ready = $state(false);
	let levels = $state<number[]>(Array(BARS).fill(0));
	let recognition: Recognition | null = null;
	let recorder: MediaRecorder | null = null;
	let stream: MediaStream | null = null;
	let audioContext: AudioContext | null = null;
	let chunks: Blob[] = [];
	let started = 0;
	let finished = false;

	const lang = speechLanguage();
	const language = (() => {
		try {
			return new Intl.DisplayNames([getLocale()], { type: 'language' }).of(lang) ?? lang;
		} catch {
			return lang;
		}
	})();

	function release() {
		stream?.getTracks().forEach((track) => track.stop());
		stream = null;
		void audioContext?.close();
		audioContext = null;
	}

	/** The microphone's loudness, a bar every LEVEL_EVERY_MS. */
	function listen(source: MediaStream): ReturnType<typeof setInterval> | undefined {
		if (typeof AudioContext === 'undefined') return undefined;
		audioContext = new AudioContext();
		const analyser = audioContext.createAnalyser();
		analyser.fftSize = 1024;
		audioContext.createMediaStreamSource(source).connect(analyser);
		const samples = new Float32Array(analyser.fftSize);
		return setInterval(() => {
			analyser.getFloatTimeDomainData(samples);
			let sum = 0;
			for (const sample of samples) sum += sample * sample;
			const rms = Math.sqrt(sum / samples.length);
			levels = [...levels.slice(1), Math.min(1, rms / FULL_LEVEL)];
		}, LEVEL_EVERY_MS);
	}

	onMount(() => {
		let timer: ReturnType<typeof setInterval> | undefined;
		let meter: ReturnType<typeof setInterval> | undefined;
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
			meter = listen(stream);

			// The live transcript is a bonus: recording works without it.
			const w = window as unknown as { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition };
			const Ctor = w.SpeechRecognition ?? w.webkitSpeechRecognition;
			if (Ctor) {
				recognition = new Ctor();
				recognition.lang = lang;
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
			clearInterval(meter);
			finished = true;
			recognition?.abort();
			if (recorder?.state === 'recording') recorder.stop();
			release();
		};
	});

	function stop() {
		if (!recorder || !ready || problem) return;
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

	const clock = $derived(`${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`);
</script>

<div class="voice">
	<VoiceRecorder elapsed={clock} {levels} fading={ready ? FADING_BARS : 0} {language} oncancel={discard} ondone={stop} />
	<p class="voice__text" aria-live="polite">
		{#if problem}
			<span class="voice__problem" role="alert">{problem}</span>
		{:else if transcript}
			{transcript}
		{:else}
			{m.voice_listening()}
		{/if}
	</p>
</div>

<style>
	.voice {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.voice__text {
		margin: 0;
		padding: 0 12px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.voice__problem {
		color: var(--md-sys-color-error);
	}
</style>
