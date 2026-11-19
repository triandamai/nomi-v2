<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import DesignCanvas from '$lib/ds/design/DesignCanvas.svelte';
	import type { PageData } from './$types';

	// The Design Studio on its own page, full screen; a project workspace shows the same canvas in its Design view.
	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>{m.studio_title()} · {m.app_name()}</title></svelte:head>

<DesignCanvas sessionId={data.sessionId} directions={data.directions ?? null} previews={data.previews} feedback={data.feedback} approval={data.approval} wanted={data.wanted} />
