<script lang="ts">
	import { speechLanguage } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';

	// Dictation through the browser's own speech recognition. Renders nothing where the browser
	// has none (Firefox, for one), so there's never a button that can't work.

	let {
		ontranscript,
		disabled = false,
	}: {
		/** Called as speech comes in: the text so far for this listening session, and whether it's final. */
		ontranscript: (text: string, final: boolean) => void;
		disabled?: boolean;
	} = $props();

	type Recognition = {
		lang: string;
		continuous: boolean;
		interimResults: boolean;
		start: () => void;
		stop: () => void;
		onresult: ((event: { results: ArrayLike<ArrayLike<{ transcript: string }> & { isFinal: boolean }> }) => void) | null;
		onend: (() => void) | null;
		onerror: ((event: { error: string }) => void) | null;
	};

	let supported = $state(false);
	let listening = $state(false);
	let problem = $state<string | null>(null);
	let recognition: Recognition | null = null;

	onMount(() => {
		const w = window as unknown as { SpeechRecognition?: new () => Recognition; webkitSpeechRecognition?: new () => Recognition };
		const Ctor = w.SpeechRecognition ?? w.webkitSpeechRecognition;
		if (!Ctor) return;
		supported = true;
		recognition = new Ctor();
		recognition.lang = speechLanguage();
		recognition.continuous = true;
		recognition.interimResults = true;
		recognition.onresult = (event) => {
			let text = '';
			let final = true;
			for (let i = 0; i < event.results.length; i++) {
				text += event.results[i][0].transcript;
				if (!event.results[i].isFinal) final = false;
			}
			ontranscript(text.trim(), final);
		};
		recognition.onend = () => (listening = false);
		recognition.onerror = (event) => {
			listening = false;
			problem = event.error === 'not-allowed' ? m.mic_blocked() : null;
		};
		return () => recognition?.stop();
	});

	function toggle() {
		if (!recognition) return;
		problem = null;
		if (listening) {
			recognition.stop();
			listening = false;
		} else {
			recognition.start();
			listening = true;
		}
	}
</script>

{#if supported}
	<button
		type="button"
		class="mic"
		class:mic--on={listening}
		aria-pressed={listening}
		aria-label={listening ? m.mic_stop() : m.mic_dictate()}
		title={problem ?? (listening ? m.mic_listening() : m.mic_dictate())}
		{disabled}
		onclick={toggle}
	>
		<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
			<rect x="9" y="3" width="6" height="11" rx="3" />
			<path d="M5 11a7 7 0 0 0 14 0M12 18v3" />
		</svg>
	</button>
{/if}

<style>
	.mic {
		position: relative;
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.mic:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.mic:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	/* Listening: square-ish and filled, with a soft pulse. */
	.mic--on {
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		animation: mic-pulse 1.4s ease-in-out infinite;
	}
	@keyframes mic-pulse {
		50% {
			box-shadow: 0 0 0 6px color-mix(in srgb, var(--md-sys-color-error) 18%, transparent);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.mic--on {
			animation: none;
		}
	}
</style>
