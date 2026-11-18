<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import DesignStudio from '$lib/components/design/DesignStudio.svelte';
	import type { PageData } from './$types';

	// The Design Studio on its own page, full screen; a project page shows the same Studio in its Design tab.
	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>{m.studio_title()}</title></svelte:head>

<DesignStudio sessionId={data.sessionId} previews={data.previews} feedback={data.feedback} approval={data.approval} wanted={data.wanted} />
