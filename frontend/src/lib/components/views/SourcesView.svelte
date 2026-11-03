<!-- What an answer leaned on (Chat responses → Sources), numbered like the [1] citations in the
     reply. A web source without a detail shows its site. -->
<script lang="ts">
	import Sources from '$lib/components/ds/chat/Sources.svelte';
	import type { SourcesView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<SourcesView> = $props();

	const id = $props.id();

	function host(url: string | undefined): string | undefined {
		if (!url) return undefined;
		try {
			return new URL(url).hostname.replace(/^www\./, '');
		} catch {
			return undefined;
		}
	}

	const sources = $derived(
		view.items.map((item) => ({
			title: item.title,
			detail: item.detail ?? host(item.url),
			kind: item.kind ?? (item.url ? ('web' as const) : ('file' as const)),
			href: item.url,
		})),
	);
</script>

<Sources id="sources-{id}" {sources} />
