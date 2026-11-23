<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/ds/actions/Button.svelte';
	import StatusScene from './StatusScene.svelte';

	// Nomi's own 404 and error pages (the old app's, in v2's look): the crew acts it out, the status
	// in mono, what happened, and always a way back. A 404 leads home; anything else also offers
	// Try again (a full reload, which also picks up a new build). `fill` takes the whole screen (the
	// root page, outside the shell); inside the shell it centres in the page's own space.
	let { status, message = '', fill = false }: { status: number; message?: string; fill?: boolean } = $props();

	const notFound = $derived(status === 404);
	// SvelteKit's own placeholders, and the app's generic "Something went wrong", say nothing the
	// page doesn't already.
	const QUIET = ['Internal Error', 'Not Found', m.error_title()];
	const detail = $derived(!notFound && message && !QUIET.includes(message) ? message : '');
</script>

<main class={['flex w-full min-w-0 flex-col items-center justify-center gap-2.5 px-4 py-6 text-center', fill ? 'min-h-dvh bg-bg text-text' : 'min-h-full']} data-status-page={status}>
	<StatusScene variant={notFound ? 'lost' : 'broken'} />
	<p class="font-mono text-label font-medium text-accent">{status}</p>
	<h1 class="text-display font-medium text-text">{notFound ? m.error_404_title() : m.error_500_title()}</h1>
	<p class="max-w-120 text-body text-text-2">{notFound ? m.error_404_text() : m.error_500_text()}</p>
	{#if detail}<p class="max-w-140 font-mono text-label wrap-anywhere text-text-3">{detail}</p>{/if}
	<div class="flex flex-wrap justify-center gap-2 pt-2">
		<Button label={m.error_home()} href="/" />
		{#if !notFound}<Button label={m.error_retry()} variant="secondary" onclick={() => location.reload()} />{/if}
	</div>
</main>
