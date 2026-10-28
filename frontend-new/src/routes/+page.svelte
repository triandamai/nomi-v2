<script lang="ts">
	import { m } from '$lib/paraglide/messages';
</script>

<svelte:head><title>{m.app_name()}</title></svelte:head>

<main>
	<h1>{m.app_name()}</h1>
	<p>{m.app_in_progress()}</p>
</main>
