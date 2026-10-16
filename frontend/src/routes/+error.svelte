<script lang="ts">
	// Nomi's own 404 and error pages: the crew acts it out, and there's always a way back.
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import StatusScene from '$lib/components/StatusScene.svelte';

	const notFound = $derived(page.status === 404);
</script>

<svelte:head><title>{notFound ? m.error_404_title() : m.error_500_title()} · Nomi</title></svelte:head>

<main class="status">
	<StatusScene variant={notFound ? 'lost' : 'broken'} />
	<p class="status__code">{page.status}</p>
	<h1 class="status__title">{notFound ? m.error_404_title() : m.error_500_title()}</h1>
	<p class="status__text">{notFound ? m.error_404_text() : m.error_500_text()}</p>
	{#if !notFound && page.error?.message && page.error.message !== 'Internal Error'}<p class="status__detail">{page.error.message}</p>{/if}
	<div class="status__actions">
		<Button href="/" variant="filled" size="m">{m.error_home()}</Button>
		{#if !notFound}<Button variant="text" size="m" onclick={() => location.reload()}>{m.error_retry()}</Button>{/if}
	</div>
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
	.status__code {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-weight: 600;
		letter-spacing: 0.08em;
	}
	.status__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.75rem, 5vw, 2.5rem);
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.status__text {
		max-width: 46ch;
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.status__detail {
		max-width: 60ch;
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.status__actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px;
		margin-top: 8px;
	}
</style>
