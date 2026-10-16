<script lang="ts">
	// Shown to everyone while an admin has maintenance mode on. It checks every few seconds and
	// takes people back to Nomi as soon as it's over.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import StatusScene from '$lib/components/StatusScene.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let state = $state(data.maintenance);
	$effect(() => {
		const timer = setInterval(async () => {
			const response = await fetch('/maintenance/status').catch(() => null);
			if (!response?.ok) return;
			state = await response.json();
			if (!state.enabled) location.href = '/';
		}, 8000);
		return () => clearInterval(timer);
	});

	const back = $derived(
		state.until ? new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(state.until)) : null,
	);
</script>

<svelte:head><title>{m.maintenance_title()} · Nomi</title></svelte:head>

<main class="status">
	<StatusScene variant="maintenance" />
	<h1 class="status__title">{m.maintenance_title()}</h1>
	<p class="status__text">{state.message || m.maintenance_default()}</p>
	<p class="status__meta">{back ? m.maintenance_back_at({ when: back }) : m.maintenance_back_soon()}</p>
	<p class="status__meta" role="status">{state.enabled ? m.maintenance_watching() : m.maintenance_over()}</p>
</main>

<style>
	.status {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		min-height: 100dvh;
		padding: 24px 16px;
		box-sizing: border-box;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		text-align: center;
	}
	.status__title {
		margin: 8px 0 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.75rem, 5vw, 2.5rem);
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.status__text {
		max-width: 46ch;
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1rem;
	}
	.status__meta {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
</style>
