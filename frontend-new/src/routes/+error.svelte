<script lang="ts">
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import StatusPage from '$lib/errors/StatusPage.svelte';

	// Anything outside the app's shell that can't be shown: an address that's nowhere (404), or a
	// page or the shell itself that failed (500). The crew acts it out; there's always a way back.
	const notFound = $derived(page.status === 404);
</script>

<svelte:head><title>{notFound ? m.error_404_title() : m.error_500_title()} · {m.app_name()}</title></svelte:head>

<StatusPage status={page.status} message={page.error?.message} fill />
