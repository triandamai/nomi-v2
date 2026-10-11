<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Something the crew read out loud. The recording loads on the first play (as a blob, so
	// seeking works without range requests); the words it says sit under a disclosure.
	import { onDestroy, tick } from 'svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'voice' }> } = $props();

	let audio = $state<HTMLAudioElement | null>(null);
	let source = $state<string | null>(null);
	let loading = $state(false);
	let failed = $state(false);
	let playing = $state(false);
	let current = $state(0);
	let duration = $state(0);

	async function load() {
		loading = true;
		failed = false;
		try {
			const response = await fetch(block.url);
			if (!response.ok) throw new Error(String(response.status));
			source = URL.createObjectURL(await response.blob());
		} catch {
			failed = true;
		}
		loading = false;
	}

	async function toggle() {
		if (!source) {
			await load();
			if (!source) return;
			// Let the <audio> render with its new src before playing.
			await tick();
		}
		if (!audio) return;
		if (audio.paused) {
			await audio.play().catch(() => (failed = true));
		} else {
			audio.pause();
		}
	}

	function seek(event: Event) {
		if (audio) audio.currentTime = Number((event.currentTarget as HTMLInputElement).value);
	}

	function clock(seconds: number) {
		if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
		const whole = Math.floor(seconds);
		return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
	}

	onDestroy(() => {
		if (source) URL.revokeObjectURL(source);
	});
</script>

<figure class="voice">
	<div class="voice__player">
		<button
			type="button"
			class="voice__play"
			aria-label={playing ? m.voice_pause() : m.voice_play()}
			disabled={loading}
			onclick={toggle}
		>
			{#if loading}
				<svg class="voice__spin" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M21 12a9 9 0 1 1-6.2-8.6" /></svg>
			{:else if playing}
				<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="6" y="5" width="4" height="14" rx="1.2" /><rect x="14" y="5" width="4" height="14" rx="1.2" /></svg>
			{:else}
				<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M8 5.6v12.8a1 1 0 0 0 1.5.9l10.2-6.4a1 1 0 0 0 0-1.8L9.5 4.7A1 1 0 0 0 8 5.6Z" /></svg>
			{/if}
		</button>
		<input
			class="voice__seek"
			type="range"
			min="0"
			max={duration || 1}
			step="0.1"
			value={current}
			disabled={!source}
			aria-label={m.voice_position()}
			style="--progress: {duration ? (current / duration) * 100 : 0}%"
			oninput={seek}
		/>
		<span class="voice__time">{clock(current)}{duration ? ` / ${clock(duration)}` : ''}</span>
	</div>
	{#if failed}
		<p class="voice__error" role="alert">{m.voice_failed()}</p>
	{/if}
	<details class="voice__words">
		<summary>{block.voice ? m.voice_read_by({ voice: block.voice }) : m.voice_words()}</summary>
		<p>{block.text}</p>
	</details>
	{#if source}
		<audio
			bind:this={audio}
			src={source}
			preload="auto"
			onplay={() => (playing = true)}
			onpause={() => (playing = false)}
			onended={() => (playing = false)}
			ontimeupdate={() => (current = audio?.currentTime ?? 0)}
			onloadedmetadata={() => (duration = audio?.duration ?? 0)}
		></audio>
	{/if}
</figure>

<style>
	.voice {
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: min(100%, 380px);
		margin: 0;
		padding: 10px 14px 10px 10px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
	}
	.voice__player {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.voice__play {
		display: grid;
		flex: none;
		place-items: center;
		width: 44px;
		height: 44px;
		border: 0;
		border-radius: 50%;
		background: var(--nomi-gradient-action);
		color: var(--nomi-on-action);
		cursor: pointer;
		transition: filter 150ms ease-out;
	}
	.voice__play:hover:not(:disabled) {
		filter: brightness(1.06);
	}
	.voice__play:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.voice__play:disabled {
		cursor: progress;
	}
	.voice__spin {
		animation: spin 900ms linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.voice__spin {
			animation: none;
		}
	}
	.voice__seek {
		flex: 1;
		min-width: 0;
		height: 4px;
		margin: 0;
		border-radius: 2px;
		background: linear-gradient(
			to right,
			var(--md-sys-color-primary) var(--progress),
			var(--md-sys-color-outline-variant) var(--progress)
		);
		cursor: pointer;
		appearance: none;
	}
	.voice__seek:disabled {
		cursor: default;
	}
	.voice__seek::-webkit-slider-thumb {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--md-sys-color-primary);
		appearance: none;
	}
	.voice__seek::-moz-range-thumb {
		width: 14px;
		height: 14px;
		border: 0;
		border-radius: 50%;
		background: var(--md-sys-color-primary);
	}
	.voice__seek:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 4px;
	}
	.voice__time {
		flex: none;
		font-size: 0.8125rem;
		font-variant-numeric: tabular-nums;
		color: var(--md-sys-color-on-surface-variant);
	}
	.voice__error {
		margin: 0 0 0 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
	.voice__words {
		margin-left: 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.voice__words summary {
		width: fit-content;
		cursor: pointer;
	}
	.voice__words summary:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.voice__words p {
		margin: 6px 0 2px;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface);
		white-space: pre-wrap;
	}
</style>
